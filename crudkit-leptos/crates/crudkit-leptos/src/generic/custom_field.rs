use crate::ReactiveValue;
use crudkit_core::Value;
use crudkit_web::generic::prelude::*;
use leptonic::prelude::ViewCallback;
use leptos::prelude::*;
use std::fmt::Debug;
use std::sync::Arc;
use std::{collections::HashMap, error::Error};

#[derive(Clone)]
pub struct CustomField<T: CrudDataTrait + 'static> {
    pub renderer: ViewCallback<(
        StoredValue<HashMap<T::Field, ReactiveValue>>, // signals
        FieldMode,                                     // field_mode
        FieldOptions,                                  // field_options
        ReactiveValue,                                 // value
        Callback<Result<Value, Arc<dyn Error>>>,       // value_changed
    )>,
}

impl<T: CrudDataTrait> Debug for CustomField<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CustomField").finish()
    }
}

impl<T: CrudDataTrait> PartialEq for CustomField<T> {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}

pub type CustomFields<T> = HashMap<<T as CrudDataTrait>::Field, CustomField<T>>;

pub type CustomCreateFields<T> = HashMap<
    <<T as CrudMainTrait>::CreateModel as CrudDataTrait>::Field,
    CustomField<<T as CrudMainTrait>::CreateModel>,
>;

pub type CustomUpdateFields<T> = HashMap<
    <<T as CrudMainTrait>::UpdateModel as CrudDataTrait>::Field,
    CustomField<<T as CrudMainTrait>::UpdateModel>,
>;

pub type CustomReadFields<T> = HashMap<
    <<T as CrudMainTrait>::ReadModel as CrudDataTrait>::Field,
    CustomField<<T as CrudMainTrait>::ReadModel>,
>;
