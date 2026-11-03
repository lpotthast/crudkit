//! A resource whose accounts live in memory, and clients that record the collaboration messages
//! they receive. The tests of the CRUD operations share them.

// Every test crate uses only a part of this module.
#![allow(dead_code)]

use std::convert::Infallible;
use std::fmt::{self, Debug, Formatter};
use std::future::ready;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::{Extension, Router};
use crudkit_core::Order;
use crudkit_core::Value;
use crudkit_core::condition::{
    Condition, ConditionClause, ConditionClauseValue, ConditionElement, Operator,
};
use crudkit_rs::crudkit_wire_format::v1::CollabMessageV1;
use crudkit_rs::data::{ConditionValueConverter, FieldLookup};
use crudkit_rs::prelude::*;
use crudkit_rs::validate::GlobalValidationState;
use crudkit_rs_macros::CkField;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use tower_service::Service;
use utoipa::ToSchema;

/// A value that must never reach the logs.
pub const SECRET: &str = "correct-horse-battery-staple";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema, CkId, CkField)]
pub struct Account {
    pub id: i64,
    pub name: String,
    pub secret: String,
}

#[derive(Clone, Debug, Deserialize, ToSchema, CkField)]
pub struct NewAccount {
    pub name: String,
    pub secret: String,
}

impl crudkit_rs::data::CreateModel for NewAccount {}

#[derive(Clone, Debug, Deserialize, ToSchema, CkField)]
pub struct AccountUpdate {
    pub name: String,
    pub secret: String,
}

/// The account `id`, whose secret is [`SECRET`].
pub fn account(id: i64) -> Account {
    Account {
        id,
        name: format!("Account {id}"),
        secret: SECRET.to_owned(),
    }
}

/// The condition `field = value`.
pub fn equals(field: &str, value: ConditionClauseValue) -> Condition {
    Condition::All(vec![ConditionElement::Clause(ConditionClause {
        column_name: field.to_owned(),
        operator: Operator::Equal,
        value,
    })])
}

/// The accounts, readable as `P` decides.
pub struct Accounts<P>(PhantomData<P>);

impl<P> Debug for Accounts<P> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("Accounts")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AccountsType;

impl ResourceType for AccountsType {
    fn name(&self) -> &'static str {
        "accounts"
    }
}

#[derive(Debug, CkResourceContext)]
pub struct AccountsContext;

/// The authentication of a client.
#[derive(Clone, Debug)]
pub struct Token;

impl RequiresAuth for Token {}

impl<P: CrudAuthPolicy> CrudResource for Accounts<P> {
    type ReadModel = Account;
    type ReadModelId = AccountId;
    type ReadModelField = AccountField;

    type CreateModel = NewAccount;
    type CreateModelField = NewAccountField;

    type UpdateModel = AccountUpdate;
    type UpdateModelField = AccountUpdateField;

    type Model = Account;
    type Id = AccountId;
    type ModelField = AccountField;

    type Repository = MemoryRepository;
    type ValidationResultRepository = NoopValidationResultRepository;
    type CollaborationService = Clients;
    type Context = AccountsContext;
    type HookData = ();
    type Lifetime = crudkit_rs::lifetime::NoopLifetimeHooks;
    type Auth = Token;
    type AuthPolicy = P;
    type ResourceType = AccountsType;
    const TYPE: AccountsType = AccountsType;
}

/// The routes of the accounts that only authenticated clients may read.
#[expect(
    clippy::diverging_sub_expression,
    reason = "The routes that `impl_add_crud_routes!` generates diverge in sub-expressions."
)]
pub mod routes {
    crudkit_rs::impl_add_crud_routes!(
        crate::support::Accounts<crudkit_rs::auth::RestrictedAuthPolicy>,
        accounts
    );
}

/// Posts `body` to the generated route `operation` of the accounts as an authenticated client, and
/// returns the status and the body of the response.
pub async fn post(
    context: &Arc<CrudContext<Accounts<RestrictedAuthPolicy>>>,
    operation: &str,
    body: &impl Serialize,
) -> (StatusCode, serde_json::Value) {
    let mut router = routes::axum_accounts_crud_routes::add_crud_routes("", Router::new())
        .layer(Extension(context.clone()))
        .layer(Extension(Token));
    let body = serde_json::to_vec(body).expect("the request should serialize");
    let request = Request::post(format!("/accounts/crud/v1/{operation}"))
        .header("content-type", "application/json")
        .body(Body::from(body))
        .expect("the request should be valid");
    std::future::poll_fn(|cx| <Router as Service<Request<Body>>>::poll_ready(&mut router, cx))
        .await
        .expect("the router should be ready");
    let response = router
        .call(request)
        .await
        .expect("the router should answer");
    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("the response body should be readable");
    let body = serde_json::from_slice(&body).expect("the response body should be JSON");
    (status, body)
}

/// The context of the accounts in `repository`, whose changes `clients` receive.
pub fn context<P: CrudAuthPolicy>(
    repository: MemoryRepository,
    clients: Arc<Clients>,
) -> Arc<CrudContext<Accounts<P>>> {
    Arc::new(CrudContext {
        res_context: Arc::new(AccountsContext),
        repository: Arc::new(repository),
        validators: Vec::new(),
        resource_validators: Vec::new(),
        validation_result_repository: Arc::new(NoopValidationResultRepository),
        collab_service: clients,
        global_validation_state: Arc::new(GlobalValidationState::new()),
    })
}

/// Stores accounts in memory. It matches conditions of equality clauses only.
#[derive(Default)]
pub struct MemoryRepository {
    accounts: Mutex<Vec<Account>>,
    calls: AtomicUsize,
}

/// A failure of the [`MemoryRepository`]. Like the error of a database, its `Debug` text names the
/// values that caused it, and its `Display` text does not.
#[derive(Debug)]
pub enum MemoryRepositoryError {
    /// The repository cannot evaluate a condition.
    Condition(String),
    /// A write would break a constraint. The detail names the values, as PostgreSQL's does.
    Constraint { detail: String },
}

impl fmt::Display for MemoryRepositoryError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Condition(reason) => write!(f, "unsupported condition: {reason}"),
            Self::Constraint { .. } => f.write_str("constraint violated"),
        }
    }
}

impl RepositoryError for MemoryRepositoryError {}

impl MemoryRepository {
    pub fn with(accounts: Vec<Account>) -> Self {
        Self {
            accounts: Mutex::new(accounts),
            calls: AtomicUsize::new(0),
        }
    }

    pub fn accounts(&self) -> Vec<Account> {
        self.lock().clone()
    }

    /// The number of requests that reached the repository.
    pub fn calls(&self) -> usize {
        self.calls.load(Ordering::SeqCst)
    }

    fn lock(&self) -> MutexGuard<'_, Vec<Account>> {
        self.accounts
            .lock()
            .expect("accounts should not be poisoned")
    }

    /// Fails like a unique constraint of a database if an account other than `id` has `name`.
    fn check_unique_name(
        accounts: &[Account],
        id: i64,
        name: &str,
    ) -> Result<(), MemoryRepositoryError> {
        if accounts
            .iter()
            .any(|account| account.id != id && account.name == name)
        {
            return Err(MemoryRepositoryError::Constraint {
                detail: format!("Key (name)=({name}) already exists."),
            });
        }
        Ok(())
    }

    /// Counts a request and returns the accounts that match `condition`, at most `limit`.
    fn matching(
        &self,
        limit: Option<u64>,
        condition: Option<&Condition>,
    ) -> Result<Vec<Account>, MemoryRepositoryError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        let limit = limit.map_or(usize::MAX, |limit| {
            usize::try_from(limit).expect("limit should fit into usize")
        });
        let mut matching = Vec::new();
        for account in self.lock().iter() {
            if matching.len() == limit {
                break;
            }
            if condition.map_or(Ok(true), |condition| matches(account, condition))? {
                matching.push(account.clone());
            }
        }
        Ok(matching)
    }
}

/// Whether `account` matches `condition`. Values are converted for their field, as a storage
/// adapter does.
fn matches(account: &Account, condition: &Condition) -> Result<bool, MemoryRepositoryError> {
    let (elements, all) = match condition {
        Condition::All(elements) => (elements, true),
        Condition::Any(elements) => (elements, false),
    };
    for element in elements {
        let matched = match element {
            ConditionElement::Clause(clause) => clause_matches(account, clause)?,
            ConditionElement::Condition(condition) => matches(account, condition)?,
        };
        if matched != all {
            return Ok(matched);
        }
    }
    Ok(all)
}

fn clause_matches(
    account: &Account,
    clause: &ConditionClause,
) -> Result<bool, MemoryRepositoryError> {
    let field = AccountField::from_name(&clause.column_name).ok_or_else(|| {
        MemoryRepositoryError::Condition(format!("unknown column '{}'", clause.column_name))
    })?;
    if clause.operator != Operator::Equal {
        return Err(MemoryRepositoryError::Condition(format!(
            "unsupported operator {:?}",
            clause.operator
        )));
    }
    let value = field
        .convert_condition_value(clause.value.clone())
        .map_err(MemoryRepositoryError::Condition)?;
    Ok(match (field, value) {
        (AccountField::Id, Value::I64(id)) => account.id == id,
        (AccountField::Name, Value::String(name)) => account.name == name,
        (AccountField::Secret, Value::String(secret)) => account.secret == secret,
        (field, _) => {
            return Err(MemoryRepositoryError::Condition(format!(
                "unexpected value for {field:?}"
            )));
        }
    })
}

impl<P: CrudAuthPolicy> Repository<Accounts<P>> for MemoryRepository {
    type Error = MemoryRepositoryError;

    fn insert(
        &self,
        create_model: NewAccount,
    ) -> impl Future<Output = Result<Account, Self::Error>> + Send {
        self.calls.fetch_add(1, Ordering::SeqCst);
        let mut accounts = self.lock();
        let id = accounts.iter().map(|account| account.id).max().unwrap_or(0) + 1;
        if let Err(err) = Self::check_unique_name(&accounts, id, &create_model.name) {
            return ready(Err(err));
        }
        let account = Account {
            id,
            name: create_model.name,
            secret: create_model.secret,
        };
        accounts.push(account.clone());
        ready(Ok(account))
    }

    fn count(
        &self,
        _limit: Option<u64>,
        _skip: Option<u64>,
        _order_by: Option<IndexMap<AccountField, Order>>,
        condition: Option<&Condition>,
    ) -> impl Future<Output = Result<u64, Self::Error>> + Send {
        ready(
            self.matching(None, condition)
                .map(|matching| u64::try_from(matching.len()).expect("count should fit into u64")),
        )
    }

    fn fetch_one(
        &self,
        _limit: Option<u64>,
        _skip: Option<u64>,
        _order_by: Option<IndexMap<AccountField, Order>>,
        condition: Option<&Condition>,
    ) -> impl Future<Output = Result<Option<Account>, Self::Error>> + Send {
        ready(
            self.matching(Some(1), condition)
                .map(|mut matching| matching.pop()),
        )
    }

    fn fetch_many(
        &self,
        limit: Option<u64>,
        _skip: Option<u64>,
        _order_by: Option<IndexMap<AccountField, Order>>,
        condition: Option<&Condition>,
    ) -> impl Future<Output = Result<Vec<Account>, Self::Error>> + Send {
        ready(self.matching(limit, condition))
    }

    fn read_one(
        &self,
        _limit: Option<u64>,
        _skip: Option<u64>,
        _order_by: Option<IndexMap<AccountField, Order>>,
        condition: Option<&Condition>,
    ) -> impl Future<Output = Result<Option<Account>, Self::Error>> + Send {
        ready(
            self.matching(Some(1), condition)
                .map(|mut matching| matching.pop()),
        )
    }

    fn read_many(
        &self,
        limit: Option<u64>,
        _skip: Option<u64>,
        _order_by: Option<IndexMap<AccountField, Order>>,
        condition: Option<&Condition>,
    ) -> impl Future<Output = Result<Vec<Account>, Self::Error>> + Send {
        ready(self.matching(limit, condition))
    }

    fn update(
        &self,
        existing: Account,
        update_model: AccountUpdate,
    ) -> impl Future<Output = Result<Account, Self::Error>> + Send {
        self.calls.fetch_add(1, Ordering::SeqCst);
        let mut accounts = self.lock();
        if let Err(err) = Self::check_unique_name(&accounts, existing.id, &update_model.name) {
            return ready(Err(err));
        }
        let updated = Account {
            id: existing.id,
            name: update_model.name,
            secret: update_model.secret,
        };
        for account in accounts.iter_mut() {
            if account.id == existing.id {
                *account = updated.clone();
            }
        }
        ready(Ok(updated))
    }

    fn delete(
        &self,
        model: Account,
    ) -> impl Future<Output = Result<DeleteResult, Self::Error>> + Send {
        self.calls.fetch_add(1, Ordering::SeqCst);
        let mut accounts = self.lock();
        let before = accounts.len();
        accounts.retain(|account| account.id != model.id);
        ready(Ok(DeleteResult {
            entities_affected: u64::try_from(before - accounts.len())
                .expect("count should fit into u64"),
        }))
    }
}

/// The clients connected for collaboration messages, with the messages that each received.
#[derive(Default)]
pub struct Clients {
    clients: Mutex<Vec<Client>>,
}

struct Client {
    authenticated: bool,
    received: Vec<CollabMessageV1>,
}

/// Identifies a client of [`Clients`].
#[derive(Debug, Clone, Copy)]
pub struct ClientId(usize);

impl Clients {
    /// Connects a client that authenticated, or one that did not.
    pub fn connect(&self, authenticated: bool) -> ClientId {
        let mut clients = self.lock();
        clients.push(Client {
            authenticated,
            received: Vec::new(),
        });
        ClientId(clients.len() - 1)
    }

    pub fn received(&self, client: ClientId) -> Vec<CollabMessageV1> {
        self.lock()[client.0].received.clone()
    }

    fn lock(&self) -> MutexGuard<'_, Vec<Client>> {
        self.clients.lock().expect("clients should not be poisoned")
    }
}

impl CollaborationService for Clients {
    type Error = Infallible;

    fn broadcast(
        &self,
        message: CollabMessageV1,
        audience: Audience,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send {
        for client in self.lock().iter_mut() {
            if audience.admits(client.authenticated) {
                client.received.push(message.clone());
            }
        }
        ready(Ok(()))
    }
}
