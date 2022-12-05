use std::fmt::{Debug, Display};
use std::hash::Hash;

use crate::{
    prelude::{
        Condition, ConditionClause, ConditionClauseValue, ConditionElement, IntoAllEqualCondition,
        Operator,
    },
    IdValue,
};
use dyn_clone::DynClone;
use serde::{Deserialize, Serialize};

//#[typetag::serde(tag = "type")]
pub trait IdFieldValue: Debug {
    fn into_condition_clause_value(&self) -> ConditionClauseValue;
}

pub trait IntoSerializableValue {
    type SerializableValue;

    fn into_serializable_value(&self) -> Self::SerializableValue;
}

pub trait IdField: Debug + Display {
    type Value: IdFieldValue + IntoSerializableValue;

    fn name(&self) -> &'static str;
    fn into_value(&self) -> Self::Value;
}

//#[typetag::serde(tag = "type")]
pub trait DynIdField: Debug + DynClone {
    fn dyn_name(&self) -> &'static str;
    fn into_dyn_value(&self) -> Box<dyn IdFieldValue>;
}
dyn_clone::clone_trait_object!(DynIdField);

/// Structs marked with this trait might be used as IDs in the crud system.
///
/// Id's might be used as keys in data structures, as they are guaranteed to be Eq, Ord and Hash!
///
/// You might want to generate a type-erased `SerializableId` using `into_serializable_id`.
pub trait Id: Debug + Display + DynClone + PartialEq + Eq + Hash + PartialOrd + Ord {
    /// This might be an enum, providing all possible fields.
    type Field: IdField;
    type FieldIter: Iterator<Item = Self::Field>;

    fn fields_iter(&self) -> Self::FieldIter;
    fn fields(&self) -> Vec<Box<dyn DynIdField>>;

    fn into_serializable_id(&self) -> SerializableId;
}

impl<I> IntoAllEqualCondition for I
where
    I: Id,
{
    fn to_all_equal_condition(&self) -> Condition {
        Condition::All(
            self.fields_iter()
                .map(|field| {
                    ConditionElement::Clause(ConditionClause {
                        column_name: String::from(field.name()),
                        operator: Operator::Equal,
                        value: field.into_value().into_condition_clause_value(),
                    })
                })
                .collect::<Vec<_>>(),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)] // TODO: Serde passthrough?
pub struct SerializableId(pub Vec<(String, IdValue)>);

impl IntoAllEqualCondition for SerializableId {
    fn to_all_equal_condition(&self) -> Condition {
        Condition::All(
            self.0
                .iter()
                .map(|(field_name, serializable_value)| {
                    ConditionElement::Clause(ConditionClause {
                        column_name: String::from(field_name),
                        operator: Operator::Equal,
                        value: serializable_value.clone().into(),
                    })
                })
                .collect::<Vec<_>>(),
        )
    }
}
