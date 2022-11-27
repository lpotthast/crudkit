use std::fmt::{Debug, Display};

use crate::prelude::{
    Condition, ConditionClause, ConditionClauseValue, ConditionElement, IntoAllEqualCondition,
    Operator,
};
use dyn_clone::DynClone;

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
pub trait Id: Debug + Display + DynClone {
    /// This might be an enum, providing all possible fields.
    type Field: IdField;
    type FieldIter: Iterator<Item = Self::Field>;

    fn fields_iter(&self) -> Self::FieldIter;
    fn fields(&self) -> Vec<Box<dyn DynIdField>>;
}

impl<I> IntoAllEqualCondition for I
where
    I: Id,
{
    fn into_all_equal_condition(self) -> Condition {
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
