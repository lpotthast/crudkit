use crate::model::{AnyCreateField, AnyUpdateField, CreateField, UpdateField};
use crate::{FieldOptions, Label, TabId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Layout {
    Columns1,
    Columns2,
    Columns3,
    Columns4,
}

impl Default for Layout {
    fn default() -> Self {
        Self::Columns2
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Elem<F> {
    Enclosing(Enclosing<F>),
    Field((F, FieldOptions)),
    Separator,
}

impl Elem<AnyCreateField> {
    pub fn create_field(field: impl CreateField, options: FieldOptions) -> Self {
        Self::Field((field.into(), options))
    }
}

impl Elem<AnyUpdateField> {
    pub fn field(field: impl UpdateField, options: FieldOptions) -> Self {
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
