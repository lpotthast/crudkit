//! Native test support for CrudKit's hooks: a single-threaded executor and an in-memory server.
//!
//! The dev-dependency on `reactive_graph` enables Leptos effects, which Leptos otherwise only runs
//! in client builds.

// With `ssr`, only the markup tests run; the helpers of the hook tests are unused then.
#![cfg_attr(feature = "ssr", allow(dead_code))]

pub(crate) mod items;

use crate::instance::CrudNavigationScope;
use crate::instance::manager_context;
use any_spawner::{CustomExecutor, Executor, PinnedFuture, PinnedLocalFuture};
use crudkit_web::http::ReqwestExecutor;
use futures::executor::{LocalPool, LocalSpawner};
use futures::task::LocalSpawnExt;
use leptos::prelude::*;
use leptos::reactive::owner::Owner;
use std::cell::RefCell;
use std::fmt;
use std::sync::{Arc, Mutex, PoisonError};

thread_local! {
    static LOCAL_POOL: RefCell<LocalPool> = RefCell::new(LocalPool::new());
    // Spawning must not borrow the pool, as running tasks spawn further tasks.
    static SPAWNER: LocalSpawner = LOCAL_POOL.with(|pool| pool.borrow().spawner());
}

/// Runs every task on the polling test thread.
///
/// Leptos spawns `LocalResource` and local action futures through `Executor::spawn`, so a
/// multi-threaded pool would poll non-`Send` futures from foreign threads.
struct SingleThreadedExecutor;

impl CustomExecutor for SingleThreadedExecutor {
    fn spawn(&self, fut: PinnedFuture<()>) {
        self.spawn_local(fut);
    }

    fn spawn_local(&self, fut: PinnedLocalFuture<()>) {
        SPAWNER.with(|spawner| {
            spawner
                .spawn_local(fut)
                .expect("the local pool accepts tasks");
        });
    }

    fn poll_local(&self) {
        LOCAL_POOL.with(|pool| {
            // A nested poll happens while a task runs; the outer poll continues the work.
            if let Ok(mut pool) = pool.try_borrow_mut() {
                pool.run_until_stalled();
            }
        });
    }
}

/// Installs the single-threaded test executor. Safe to call from every test.
///
/// [`with_crud_manager`] calls it, so tests using that do not need to.
pub(crate) fn init_executor() {
    _ = Executor::init_custom_executor(SingleThreadedExecutor);
}

/// Runs pending tasks, effects, and resources of the current thread until they stall, e.g. after
/// triggering a request.
pub(crate) fn settle() {
    for _ in 0..10 {
        Executor::poll_local();
    }
}

/// Runs `test` in a fresh reactive owner below a CrudKit manager, as if rendered inside a
/// [`crate::instance::CrudInstanceMgr`].
///
/// Inside, mount instances with [`crate::instance::provide_crud_instance`] and use CrudKit's
/// hooks. Provide a [`crate::hooks::notify::CrudNotifier`] first to observe notifications.
pub(crate) fn with_crud_manager(test: impl FnOnce()) {
    init_executor();
    let owner = Owner::new();
    owner.with(|| {
        provide_context(manager_context(CrudNavigationScope::new()));
        test();
    });
}

/// A request received by a [`CrudTestServer`].
#[derive(Debug, Clone)]
pub(crate) struct CrudTestRequest {
    /// The CrudKit operation, i.e. the last path segment, e.g. `read-many` or `update-one`.
    pub(crate) operation: String,
    /// The JSON body, or `null` without a body.
    pub(crate) body: serde_json::Value,
}

/// The answer of a [`CrudTestServer`] to one request.
#[derive(Debug, Clone)]
pub(crate) struct CrudTestResponse {
    /// The HTTP status code.
    status: u16,
    /// The JSON body.
    body: serde_json::Value,
}

impl CrudTestResponse {
    /// A successful response with `body`.
    #[must_use]
    pub(crate) fn json(body: serde_json::Value) -> Self {
        Self { status: 200, body }
    }

    /// A response with `status`, e.g. 403 or 422, and `body`. `status` must be a valid HTTP
    /// status code; the request panics otherwise.
    #[must_use]
    pub(crate) fn status(status: u16, body: serde_json::Value) -> Self {
        Self { status, body }
    }
}

type Handler = dyn Fn(&CrudTestRequest) -> CrudTestResponse + Send + Sync;

/// An in-memory CrudKit API answering every request with a handler, and recording the requests.
///
/// Pass [`Self::executor`] as the `reqwest_executor` of a [`crate::prelude::CrudInstanceConfig`].
pub(crate) struct CrudTestServer {
    handler: Box<Handler>,
    requests: Mutex<Vec<CrudTestRequest>>,
}

impl fmt::Debug for CrudTestServer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CrudTestServer")
            .field("requests", &self.requests)
            .finish_non_exhaustive()
    }
}

impl CrudTestServer {
    /// Creates a server answering requests with `handler`.
    pub(crate) fn new(
        handler: impl Fn(&CrudTestRequest) -> CrudTestResponse + Send + Sync + 'static,
    ) -> Arc<Self> {
        Arc::new(Self {
            handler: Box::new(handler),
            requests: Mutex::default(),
        })
    }

    /// Returns an executor sending requests to this server.
    #[must_use]
    pub(crate) fn executor(self: &Arc<Self>) -> Arc<dyn ReqwestExecutor> {
        Arc::new(CrudTestExecutor(Arc::clone(self)))
    }

    /// Returns the bodies of all requests of `operation` received so far, oldest first.
    #[must_use]
    pub(crate) fn requests_to(&self, operation: &str) -> Vec<serde_json::Value> {
        self.requests
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .iter()
            .filter(|request| request.operation == operation)
            .map(|request| request.body.clone())
            .collect()
    }
}

#[derive(Debug)]
struct CrudTestExecutor(Arc<CrudTestServer>);

#[async_trait::async_trait]
impl ReqwestExecutor for CrudTestExecutor {
    async fn request(
        &self,
        method: http::Method,
        url: reqwest::Url,
        with: Arc<dyn Fn(reqwest::RequestBuilder) -> reqwest::RequestBuilder + Send + Sync>,
    ) -> Result<reqwest::Response, reqwest::Error> {
        let request = with(reqwest::Client::new().request(method, url.clone())).build()?;
        let body = request
            .body()
            .and_then(reqwest::Body::as_bytes)
            .and_then(|bytes| serde_json::from_slice(bytes).ok())
            .unwrap_or(serde_json::Value::Null);
        let request = CrudTestRequest {
            operation: url
                .path_segments()
                .and_then(|mut segments| segments.next_back())
                .unwrap_or_default()
                .to_owned(),
            body,
        };
        let response = (self.0.handler)(&request);
        self.0
            .requests
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(request);
        Ok(reqwest::Response::from(
            http::Response::builder()
                .status(response.status)
                .header("Content-Type", "application/json")
                .body(serde_json::to_vec(&response.body).unwrap_or_default())
                .expect("a status code and a JSON header form a valid response"),
        ))
    }
}
