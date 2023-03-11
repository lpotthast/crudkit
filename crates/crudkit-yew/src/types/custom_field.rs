use std::{sync::Arc, collections::HashMap};
use std::fmt::Debug;

use yew::Html;

use crate::{FieldMode, CrudDataTrait, CrudMainTrait};


#[derive(Clone)]
pub struct CustomField<T: CrudDataTrait> {
    pub renderer: Box<Arc<dyn Fn(&T, FieldMode) -> Html>>,
}

impl<T: CrudDataTrait> Debug for CustomField<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f
            .debug_struct("CustomField")
            .finish()
    }
}

impl<T: CrudDataTrait> PartialEq for CustomField<T> {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}
impl<T: CrudDataTrait> CustomField<T> {
    pub fn render(&self, entity: &T, field_mode: FieldMode) -> yew::Html {
        (self.renderer)(entity, field_mode)
    }
}

pub type CustomFields<T> = HashMap<<T as CrudDataTrait>::Field, CustomField<T>>;
pub type CustomCreateFields<T> = HashMap<<<T as CrudMainTrait>::CreateModel as CrudDataTrait>::Field, CustomField<<T as CrudMainTrait>::CreateModel>>;
pub type CustomUpdateFields<T> = HashMap<<<T as CrudMainTrait>::UpdateModel as CrudDataTrait>::Field, CustomField<<T as CrudMainTrait>::UpdateModel>>;
pub type CustomReadFields<T> = HashMap<<<T as CrudMainTrait>::ReadModel as CrudDataTrait>::Field, CustomField<<T as CrudMainTrait>::ReadModel>>;
