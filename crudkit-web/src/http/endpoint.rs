use crudkit_core::condition::{Condition, merge_conditions};
use crudkit_wire_format::WireFormatVersion;
use serde::{Serialize, de::DeserializeOwned};
use std::fmt::Debug;
use std::sync::Arc;

use super::error::RequestError;
use super::executor::ReqwestExecutor;
use super::request;

/// An operation of the generated CRUD routes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CrudOperation {
    ReadCount,
    ReadOne,
    ReadMany,
    CreateOne,
    UpdateOne,
    DeleteById,
    DeleteMany,
}

impl CrudOperation {
    /// The last path segment of the operation's route.
    fn path_segment(self) -> &'static str {
        match self {
            Self::ReadCount => "read-count",
            Self::ReadOne => "read-one",
            Self::ReadMany => "read-many",
            Self::CreateOne => "create-one",
            Self::UpdateOne => "update-one",
            Self::DeleteById => "delete-by-id",
            Self::DeleteMany => "delete-many",
        }
    }
}

/// The generated CRUD routes of one resource, together with the client scope applied to them.
#[derive(Debug, Clone)]
pub(crate) struct CrudEndpoint {
    api_base_url: String,
    executor: Arc<dyn ReqwestExecutor>,
    resource_name: String,
    base_condition: Option<Condition>,
}

impl CrudEndpoint {
    pub(crate) fn new(
        api_base_url: String,
        executor: Arc<dyn ReqwestExecutor>,
        resource_name: String,
    ) -> Self {
        Self {
            api_base_url,
            executor,
            resource_name,
            base_condition: None,
        }
    }

    pub(crate) fn set_base_condition(&mut self, condition: Option<Condition>) {
        self.base_condition = condition;
    }

    /// Narrows `condition` to the base condition.
    pub(crate) fn scoped(&self, condition: Option<Condition>) -> Option<Condition> {
        merge_conditions(self.base_condition.clone(), condition)
    }

    /// Posts `body` to the version 1 route of `operation`.
    pub(crate) async fn post<B, T>(
        &self,
        operation: CrudOperation,
        body: B,
    ) -> Result<T, RequestError>
    where
        T: DeserializeOwned + Debug,
        B: Serialize + Debug + Send + Sync + 'static,
    {
        let url = format!(
            "{}/{}/crud/{}/{}",
            self.api_base_url,
            self.resource_name,
            WireFormatVersion::V1.path_segment(),
            operation.path_segment()
        );
        request::post(url, self.executor.as_ref(), body).await
    }
}
