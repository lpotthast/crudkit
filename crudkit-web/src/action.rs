//! Payloads of resource and entity actions.

use dyn_clone::DynClone;
use dyn_eq::DynEq;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::ops::Deref;

/// Marker trait for data that can be used as the payload of CRUD actions.
///
/// Use the `CkActionPayload` derive macro to implement this and [`ErasedActionPayload`] for your type.
pub trait ActionPayload:
    PartialEq + Clone + Debug + Serialize + DeserializeOwned + Send + Sync
{
}

/// The payload of actions that do not need one.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub struct EmptyActionPayload {}

impl ActionPayload for EmptyActionPayload {}

/// Object-safe form of [`ActionPayload`], used where payloads of different types are handled alike.
pub trait ErasedActionPayload:
    Debug + DynClone + DynEq + downcast_rs::Downcast + Send + Sync
{
}
dyn_eq::eq_trait_object!(ErasedActionPayload);
dyn_clone::clone_trait_object!(ErasedActionPayload);
downcast_rs::impl_downcast!(ErasedActionPayload);

/// Any type-erased action payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DynActionPayload {
    inner: Box<dyn ErasedActionPayload>,
}

impl<T: ErasedActionPayload> From<T> for DynActionPayload {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

impl DynActionPayload {
    pub fn new<Concrete: ErasedActionPayload>(concrete: Concrete) -> Self {
        Self {
            inner: Box::new(concrete),
        }
    }

    /// Take the payload as its concrete type.
    ///
    /// # Panics
    ///
    /// Panics if the payload is not of type `Concrete`.
    #[must_use]
    pub fn downcast<Concrete: ErasedActionPayload>(self) -> Concrete {
        *self.inner.downcast::<Concrete>().expect("correct")
    }

    /// Borrow the payload as its concrete type.
    ///
    /// # Panics
    ///
    /// Panics if the payload is not of type `Concrete`.
    #[must_use]
    pub fn downcast_ref<Concrete: ErasedActionPayload>(&self) -> &Concrete {
        self.inner.downcast_ref::<Concrete>().expect("correct")
    }

    /// Mutably borrow the payload as its concrete type.
    ///
    /// # Panics
    ///
    /// Panics if the payload is not of type `Concrete`.
    pub fn downcast_mut<Concrete: ErasedActionPayload>(&mut self) -> &mut Concrete {
        self.inner.downcast_mut::<Concrete>().expect("correct")
    }
}

impl Deref for DynActionPayload {
    type Target = dyn ErasedActionPayload;

    fn deref(&self) -> &Self::Target {
        self.inner.as_ref()
    }
}
