use crate::model::{DynCreateField, DynUpdateField, ErasedCreateField, ErasedUpdateField};
use crate::{FieldOptions, Label, TabId};
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Layout {
    Columns1,
    #[default]
    Columns2,
    Columns3,
    Columns4,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Elem<F> {
    Enclosing(Enclosing<F>),
    Field((F, FieldOptions)),
    Separator,
}

impl Elem<DynCreateField> {
    pub fn create_field(field: impl ErasedCreateField, options: FieldOptions) -> Self {
        Self::Field((field.into(), options))
    }
}

impl Elem<DynUpdateField> {
    pub fn field(field: impl ErasedUpdateField, options: FieldOptions) -> Self {
        Self::Field((field.into(), options))
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Enclosing<F> {
    None(Group<F>),
    Tabs(Vec<Tab<F>>),
    Card(Group<F>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tab<F> {
    /// A unique identifier for this tab.
    pub id: TabId,
    pub label: Label,
    pub group: Group<F>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Group<F> {
    pub layout: Layout,
    pub children: Vec<Elem<F>>,
}
