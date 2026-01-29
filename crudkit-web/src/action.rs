use dyn_clone::DynClone;
use dyn_eq::DynEq;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::ops::Deref;

/// Marker trait for specifying data which can be used as payload for CRUD actions.
/// Use the `CrudActionPayload` derive macro from derive-crud-action-payload to implement this for your type.
pub trait CrudActionPayload:
    PartialEq + Clone + Debug + Serialize + DeserializeOwned + Send + Sync
{
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub struct EmptyActionPayload {}

impl CrudActionPayload for EmptyActionPayload {}

pub trait ActionPayload: Debug + DynClone + DynEq + downcast_rs::Downcast + Send + Sync {}
dyn_eq::eq_trait_object!(ActionPayload);
dyn_clone::clone_trait_object!(ActionPayload);
downcast_rs::impl_downcast!(ActionPayload);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnyActionPayload {
    inner: Box<dyn ActionPayload>,
}

impl AnyActionPayload {
    pub fn from<Concrete: ActionPayload>(concrete: Concrete) -> Self {
        Self {
            inner: Box::new(concrete),
        }
    }

    pub fn downcast<Concrete: ActionPayload>(self) -> Concrete {
        *self.inner.downcast::<Concrete>().expect("correct")
    }

    pub fn downcast_ref<Concrete: ActionPayload>(&self) -> &Concrete {
        self.inner.downcast_ref::<Concrete>().expect("correct")
    }

    pub fn downcast_mut<Concrete: ActionPayload>(&mut self) -> &mut Concrete {
        self.inner.downcast_mut::<Concrete>().expect("correct")
    }
}

impl Deref for AnyActionPayload {
    type Target = dyn ActionPayload;

    fn deref(&self) -> &Self::Target {
        self.inner.as_ref()
    }
}
