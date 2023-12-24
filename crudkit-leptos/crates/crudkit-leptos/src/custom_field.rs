use std::{collections::HashMap, error::Error};
use std::fmt::Debug;
use std::rc::Rc;

use crudkit_web::Value;
use leptos::{Callback, StoredValue};

use crate::{CrudDataTrait, CrudMainTrait, FieldMode, FieldOptions, ReactiveValue};

/// O: Output of the renderer.
#[derive(Clone)]
pub struct CustomField<T: CrudDataTrait + 'static, O> {
    pub renderer: Rc<dyn Fn(StoredValue<HashMap<T::Field, ReactiveValue>>, FieldMode, FieldOptions, ReactiveValue, Callback<Result<Value, Box<dyn Error>>>) -> O>,
}

impl<T: CrudDataTrait, O> Debug for CustomField<T, O> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CustomField").finish()
    }
}

impl<T: CrudDataTrait, O> PartialEq for CustomField<T, O> {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}

impl<T: CrudDataTrait, O> CustomField<T, O> {
    pub fn render(&self, signals: StoredValue<HashMap<T::Field, ReactiveValue>>, field_mode: FieldMode, field_options: FieldOptions, value: ReactiveValue, value_changed: Callback<Result<Value, Box<dyn Error>>>) -> O {
        (self.renderer)(signals, field_mode, field_options, value, value_changed)
    }
}

pub type CustomFields<T, O> = HashMap<<T as CrudDataTrait>::Field, CustomField<T, O>>;

pub type CustomCreateFields<T, O> = HashMap<
    <<T as CrudMainTrait>::CreateModel as CrudDataTrait>::Field,
    CustomField<<T as CrudMainTrait>::CreateModel, O>,
>;

pub type CustomUpdateFields<T, O> = HashMap<
    <<T as CrudMainTrait>::UpdateModel as CrudDataTrait>::Field,
    CustomField<<T as CrudMainTrait>::UpdateModel, O>,
>;

pub type CustomReadFields<T, O> = HashMap<
    <<T as CrudMainTrait>::ReadModel as CrudDataTrait>::Field,
    CustomField<<T as CrudMainTrait>::ReadModel, O>,
>;
