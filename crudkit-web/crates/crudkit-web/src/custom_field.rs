use std::fmt::Debug;
use std::{collections::HashMap, sync::Arc};

use crate::{CrudDataTrait, CrudMainTrait, FieldMode};

/// O: Output of the renderer.
#[derive(Clone)]
pub struct CustomField<T: CrudDataTrait, O> {
    pub renderer: Box<Arc<dyn Fn(&T, FieldMode) -> O>>,
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
    pub fn render(&self, entity: &T, field_mode: FieldMode) -> O {
        (self.renderer)(entity, field_mode)
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
