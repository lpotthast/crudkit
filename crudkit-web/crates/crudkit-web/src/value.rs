use crate::CrudSelectableTrait;
use crudkit_condition::ConditionClauseValue;
use crudkit_id::IdValue;
use crudkit_shared::TimeDuration;
use std::fmt::Display;
use thiserror::Error;
use time::format_description::well_known::Rfc3339;
use tracing::warn;

/// The value of a field.
/// All variants should be stateless / copy-replaceable.
// TODO: DEFERRED: Implement Serialize and Deserialize with typetag when wasm is supported in typetag. Comment in "typetag" occurrences.
#[derive(Debug, Clone)]
pub enum Value {
    I32(i32),
    U32(u32),
    I64(i64),
    U64(u64),
    I128(i128),
    U128(u128),
    OptionalI32(Option<i32>),
    OptionalI64(Option<i64>),
    OptionalU32(Option<u32>),
    OptionalU64(Option<u64>),
    OptionalI128(Option<i128>),
    OptionalU128(Option<u128>),
    F32(f32),
    F64(f64),
    Bool(bool),
    String(String),
    OptionalString(Option<String>),
    // Ecosystem support.
    // -- serde
    Json(serde_json::Value),
    OptionalJson(Option<serde_json::Value>),
    // -- uuid
    Uuid(uuid::Uuid),
    OptionalUuid(Option<uuid::Uuid>),
    // -- time
    PrimitiveDateTime(time::PrimitiveDateTime),
    OptionalPrimitiveDateTime(Option<time::PrimitiveDateTime>),
    OffsetDateTime(time::OffsetDateTime),
    OptionalOffsetDateTime(Option<time::OffsetDateTime>),
    Duration(TimeDuration),
    OptionalDuration(Option<TimeDuration>),

    // Specialized bool-case, render as a green check mark if false and an orange exclamation mark if true.
    ValidationStatus(bool),

    OneToOneRelation(Option<u32>),
    Reference(Vec<Box<dyn crudkit_id::IdField>>), // TODO: This variant should probably be named "Reference". Can it carry a "SerializableId" (as it is of known size)?
    Custom(()),
    Select(Box<dyn CrudSelectableTrait>),
    Multiselect(Vec<Box<dyn CrudSelectableTrait>>),
    OptionalSelect(Option<Box<dyn CrudSelectableTrait>>),
    OptionalMultiselect(Option<Vec<Box<dyn CrudSelectableTrait>>>),
    //Select(Box<dyn CrudSelectableSource<Selectable = dyn CrudSelectableTrait>>),
}

#[derive(Debug, Clone)]
pub struct JsonValue {
    value: serde_json::Value,
    string_representation: String,
}

impl JsonValue {
    pub fn new(value: serde_json::Value) -> Self {
        let string_representation = serde_json::to_string(&value).unwrap();
        Self {
            value,
            string_representation,
        }
    }

    pub fn set_value(&mut self, value: serde_json::Value) {
        self.value = value;
        self.string_representation = serde_json::to_string(&self.value).unwrap();
    }

    pub fn get_value(&self) -> &serde_json::Value {
        &self.value
    }

    pub fn get_string_representation(&self) -> &str {
        self.string_representation.as_str()
    }
}

impl From<JsonValue> for serde_json::Value {
    fn from(value: JsonValue) -> Self {
        value.value
    }
}

impl From<JsonValue> for String {
    fn from(value: JsonValue) -> Self {
        value.string_representation
    }
}

impl From<crudkit_shared::Value> for Value {
    fn from(value: crudkit_shared::Value) -> Self {
        match value {
            crudkit_shared::Value::String(value) => Value::String(value),
            crudkit_shared::Value::Json(value) => Value::Json(value),
            crudkit_shared::Value::Uuid(value) => Value::Uuid(value),
            crudkit_shared::Value::I32(value) => Value::I32(value),
            crudkit_shared::Value::I64(value) => Value::I64(value),
            crudkit_shared::Value::U8Vec(_values) => todo!("support vector types"),
            crudkit_shared::Value::I32Vec(_values) => todo!("support vector types"),
            crudkit_shared::Value::I64Vec(_values) => todo!("support vector types"),
            crudkit_shared::Value::U32(value) => Value::U32(value),
            crudkit_shared::Value::U64(value) => Value::U64(value),
            crudkit_shared::Value::F32(value) => Value::F32(value),
            crudkit_shared::Value::F64(value) => Value::F64(value),
            crudkit_shared::Value::Bool(value) => Value::Bool(value),
            crudkit_shared::Value::PrimitiveDateTime(value) => Value::PrimitiveDateTime(value),
            crudkit_shared::Value::OffsetDateTime(value) => Value::OffsetDateTime(value),
            crudkit_shared::Value::Duration(value) => Value::Duration(value),
        }
    }
}

impl From<IdValue> for Value {
    fn from(id_value: IdValue) -> Self {
        match id_value {
            IdValue::String(value) => Value::String(value), // TODO: How can we differentiate between String and Text?
            IdValue::Uuid(value) => Value::Uuid(value),
            IdValue::I32(value) => Value::I32(value),
            IdValue::U32(value) => Value::U32(value),
            IdValue::I64(value) => Value::I64(value),
            IdValue::U64(value) => Value::U64(value),
            IdValue::I128(value) => Value::I128(value),
            IdValue::U128(value) => Value::U128(value),
            IdValue::Bool(value) => Value::Bool(value),
            IdValue::PrimitiveDateTime(value) => Value::PrimitiveDateTime(value),
            IdValue::OffsetDateTime(value) => Value::OffsetDateTime(value),
        }
    }
}

impl Value {
    pub fn take_string(self) -> String {
        match self {
            Self::String(string) => string,
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_optional_string(self) -> Option<String> {
        match self {
            Self::OptionalString(string) => string,
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_json_value(self) -> serde_json::Value {
        match self {
            Self::Json(value) => value,
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_optional_json_value(self) -> Option<serde_json::Value> {
        match self {
            Self::OptionalJson(value) => value,
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_uuid(self) -> uuid::Uuid {
        match self {
            Self::Uuid(uuid) => uuid,
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_optional_uuid(self) -> Option<uuid::Uuid> {
        match self {
            Self::OptionalUuid(uuid) => uuid,
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_u32(self) -> u32 {
        match self {
            Self::U32(u32) => u32,
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_u64(self) -> u64 {
        match self {
            Self::U64(u64) => u64,
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_i128(self) -> i128 {
        match self {
            Self::I128(value) => value,
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_u128(self) -> u128 {
        match self {
            Self::U128(value) => value,
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_u32_or_parse(self) -> u32 {
        match self {
            Self::U32(u32) => u32,
            Self::String(string) => string.parse().unwrap(),
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_optional_u32(self) -> Option<u32> {
        match self {
            Self::OptionalU32(u32) => u32,
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_i32(self) -> i32 {
        match self {
            Self::I32(i32) => i32,
            // This has some potential data loss...
            // TODO: Can we remove this? Without, this created a panic in startblock/servers/labels/new
            Self::U32(u32) => u32 as i32,
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_i64(self) -> i64 {
        match self {
            Self::I64(i64) => i64,
            // This has some potential data loss...
            // TODO: Can we remove this? Without, this created a panic in startblock/servers/labels/new
            //Self::U32(u32) => u32 as i32,
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_optional_i32(self) -> Option<i32> {
        match self {
            Self::I32(value) => Some(value),
            Self::OptionalI32(value) => value,
            Self::String(string) => string
                .parse::<i32>()
                .map_err(|err| warn!("take_optional_i32 could not pase string: {err}"))
                .ok(),
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_optional_i64(self) -> Option<i64> {
        match self {
            Self::I64(value) => Some(value),
            Self::OptionalI64(value) => value,
            Self::String(string) => string
                .parse::<i64>()
                .map_err(|err| warn!("take_optional_i64 could not pase string: {err}"))
                .ok(),
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_optional_u64(self) -> Option<u64> {
        match self {
            Self::U64(value) => Some(value),
            Self::OptionalU64(value) => value,
            Self::String(value) => value
                .parse::<u64>()
                .map_err(|err| warn!("take_optional_u64 could not pase string: {err}"))
                .ok(),
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_optional_i128(self) -> Option<i128> {
        match self {
            Self::I128(value) => Some(value),
            Self::OptionalI128(value) => value,
            Self::String(value) => value
                .parse::<i128>()
                .map_err(|err| warn!("take_optional_i128 could not pase string: {err}"))
                .ok(),
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_optional_u128(self) -> Option<u128> {
        match self {
            Self::U128(value) => Some(value),
            Self::OptionalU128(value) => value,
            Self::String(value) => value
                .parse::<u128>()
                .map_err(|err| warn!("take_optional_u128 could not pase string: {err}"))
                .ok(),
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_f32(self) -> f32 {
        match self {
            Self::F32(value) => value,
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_f64(self) -> f64 {
        match self {
            Self::F64(value) => value,
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_bool(self) -> bool {
        match self {
            Self::Bool(bool) => bool,
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    /*
    pub fn take_bool_or_parse(self) -> bool {
        match self {
            Self::Bool(bool) => bool,
            Self::String(string) => string.parse().unwrap(),
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    */
    pub fn take_validation_status(self) -> bool {
        match self {
            Self::ValidationStatus(bool) => bool,
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_primitive_date_time(self) -> time::PrimitiveDateTime {
        match self {
            Self::PrimitiveDateTime(primitive_date_time) => primitive_date_time,
            Self::String(string) => time::PrimitiveDateTime::parse(&string, &Rfc3339).unwrap(),
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_offset_date_time(self) -> time::OffsetDateTime {
        match self {
            Self::OffsetDateTime(offset_date_time) => offset_date_time,
            Self::String(string) => time::OffsetDateTime::parse(&string, &Rfc3339).unwrap(),
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_duration(self) -> TimeDuration {
        match self {
            Self::Duration(duration) => duration,
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_optional_primitive_date_time(self) -> Option<time::PrimitiveDateTime> {
        match self {
            Self::PrimitiveDateTime(primitive_date_time) => Some(primitive_date_time),
            Self::OptionalPrimitiveDateTime(optional_primitive_date_time) => {
                optional_primitive_date_time
            }
            // TODO: We might want to catch parsing errors and return an empty optional here.
            Self::String(string) => {
                Some(time::PrimitiveDateTime::parse(&string, &Rfc3339).unwrap())
            }
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_optional_offset_date_time(self) -> Option<time::OffsetDateTime> {
        match self {
            Self::OffsetDateTime(offset_date_time) => Some(offset_date_time),
            Self::OptionalOffsetDateTime(optional_offset_date_time) => optional_offset_date_time,
            // TODO: We might want to catch parsing errors and return an empty optional here.
            Self::String(string) => Some(time::OffsetDateTime::parse(&string, &Rfc3339).unwrap()),
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_optional_duration(self) -> Option<TimeDuration> {
        match self {
            Self::Duration(duration) => Some(duration),
            Self::OptionalDuration(optional_duration) => optional_duration,
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_select(self) -> Box<dyn CrudSelectableTrait> {
        match self {
            Self::Select(selected) => selected,
            other => panic!("unsupported type, expected select, found: {other:?}"),
        }
    }
    pub fn take_select_downcast_to<T: Clone + 'static>(self) -> T {
        match self {
            Self::Select(selected) => selected.as_any().downcast_ref::<T>().unwrap().clone(),
            other => panic!("Expected variant `Value::Select` but got `{other:?}`."),
        }
    }
    pub fn take_optional_select_downcast_to<T: Clone + 'static>(self) -> Option<T> {
        match self {
            Self::OptionalSelect(selected) => {
                selected.map(|it| it.as_any().downcast_ref::<T>().unwrap().clone())
            }
            other => panic!("Expected variant `Value::OptionalSelect` but got `{other:?}`."),
        }
    }
    pub fn take_multiselect(self) -> Vec<Box<dyn CrudSelectableTrait>> {
        match self {
            Self::Multiselect(selected) => selected,
            other => panic!("unsupported type, expected select, found: {other:?}"),
        }
    }
    pub fn take_multiselect_downcast_to<T: Clone + 'static>(self) -> Vec<T> {
        match self {
            Self::Multiselect(selected) => selected
                .into_iter()
                .map(|value| value.as_any().downcast_ref::<T>().unwrap().clone())
                .collect(),
            _ => panic!("unsupported type provided"),
        }
    }
    pub fn take_optional_multiselect_downcast_to<T: Clone + 'static>(self) -> Option<Vec<T>> {
        match self {
            Self::OptionalMultiselect(selected) => selected.map(|it| {
                it.into_iter()
                    .map(|it| it.as_any().downcast_ref::<T>().unwrap().clone())
                    .collect()
            }),
            _ => panic!("unsupported type provided"),
        }
    }
    pub fn take_one_to_one_relation(self) -> Option<u32> {
        match self {
            Value::U32(u32) => Some(u32),
            Value::OptionalU32(optional_u32) => optional_u32,
            Value::OneToOneRelation(optional_u32) => optional_u32,
            other => panic!(
                "Expected Value of variant 'U32', 'OptionalU32' or 'OneToOneRelation'. Received: {other:?}"
            ),
        }
    }
    pub fn take_reference(self) -> Vec<Box<dyn crudkit_id::IdField>> {
        match self {
            Value::Reference(id) => id,
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_custom(self) -> () {
        match self {
            Value::Custom(nothing) => nothing,
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::String(value) => f.write_str(value),
            Value::OptionalString(value) => match value {
                Some(value) => f.write_str(value),
                None => f.write_str("-"),
            },
            Value::Json(value) => f.write_str(&serde_json::to_string(value).unwrap()),
            Value::OptionalJson(value) => match value {
                Some(value) => f.write_str(&serde_json::to_string(value).unwrap()),
                None => f.write_str("-"),
            },
            Value::Uuid(value) => f.write_str(&value.to_string()),
            Value::OptionalUuid(value) => match value {
                Some(value) => f.write_str(&value.to_string()),
                None => f.write_str("-"),
            },
            Value::I32(value) => f.write_str(&value.to_string()),
            Value::U32(value) => f.write_str(&value.to_string()),
            Value::I64(value) => f.write_str(&value.to_string()),
            Value::U64(value) => f.write_str(&value.to_string()),
            Value::I128(value) => f.write_str(&value.to_string()),
            Value::U128(value) => f.write_str(&value.to_string()),
            Value::OptionalI32(value) => match value {
                Some(value) => f.write_str(&value.to_string()),
                None => f.write_str("-"),
            },
            Value::OptionalI64(value) => match value {
                Some(value) => f.write_str(&value.to_string()),
                None => f.write_str("-"),
            },
            Value::OptionalU32(value) => match value {
                Some(value) => f.write_str(&value.to_string()),
                None => f.write_str("-"),
            },
            Value::OptionalU64(value) => match value {
                Some(value) => f.write_str(&value.to_string()),
                None => f.write_str("-"),
            },
            Value::OptionalI128(value) => match value {
                Some(value) => f.write_str(&value.to_string()),
                None => f.write_str("-"),
            },
            Value::OptionalU128(value) => match value {
                Some(value) => f.write_str(&value.to_string()),
                None => f.write_str("-"),
            },
            Value::F32(value) => f.write_str(&value.to_string()),
            Value::F64(value) => f.write_str(&value.to_string()),
            Value::Bool(value) => f.write_str(&value.to_string()),
            Value::ValidationStatus(value) => f.write_str(&value.to_string()),
            Value::PrimitiveDateTime(value) => f.write_str(&value.format(&Rfc3339).unwrap()),
            Value::OffsetDateTime(value) => f.write_str(&value.format(&Rfc3339).unwrap()),
            Value::OptionalPrimitiveDateTime(value) => match value {
                Some(value) => f.write_str(&value.format(&Rfc3339).unwrap()),
                None => f.write_str(""),
            },
            Value::OptionalOffsetDateTime(value) => match value {
                Some(value) => f.write_str(&value.format(&Rfc3339).unwrap()),
                None => f.write_str(""),
            },
            Value::OneToOneRelation(value) => match value {
                Some(value) => f.write_str(&value.to_string()),
                None => f.write_str(""),
            },
            Value::Reference(id) => {
                for field in id {
                    f.write_fmt(format_args!(
                        "'{}': {:?}",
                        field.name(),
                        field.to_value() // was: into_dyn_value
                    ))?;
                }
                Ok(())
            }
            Value::Custom(_) => f.write_str("Custom"),
            Value::Select(selected) => f.write_str(&selected.to_string()),
            Value::OptionalSelect(selected) => match selected {
                Some(selected) => f.write_str(&selected.to_string()),
                None => f.write_str("NONE"),
            },
            Value::Multiselect(selected) => {
                for value in selected {
                    f.write_str(&value.to_string())?
                }
                Ok(())
            }
            Value::OptionalMultiselect(selected) => match selected {
                Some(selected) => {
                    for value in selected {
                        f.write_str(&value.to_string())?
                    }
                    Ok(())
                }
                None => f.write_str("NONE"),
            },
            // TODO: Correct formatting? Should we do a manual: [h]:[m]:[s]?
            Value::Duration(value) => f.write_str(&value.0.to_string()),
            Value::OptionalDuration(value) => match value {
                Some(value) => {
                    f.write_str(&value.0.to_string())?;
                    Ok(())
                }
                None => f.write_str("NONE"),
            },
        }
    }
}

#[derive(Debug, Error)]
#[error("The value '{value:?}' cannot be used in a condition clause.")]
pub struct NotConditionClauseCompatible {
    value: Value,
}

impl TryInto<ConditionClauseValue> for Value {
    type Error = NotConditionClauseCompatible;

    fn try_into(self) -> Result<ConditionClauseValue, Self::Error> {
        match self {
            Value::String(value) => Ok(ConditionClauseValue::String(value)),
            value @ Value::OptionalString(_) => Err(NotConditionClauseCompatible { value }),
            Value::Json(value) => Ok(ConditionClauseValue::Json(
                serde_json::to_string(&value).unwrap(),
            )),
            value @ Value::OptionalJson(_) => Err(NotConditionClauseCompatible { value }),
            Value::Uuid(value) => Ok(ConditionClauseValue::Uuid(value)),
            value @ Value::OptionalUuid(_) => Err(NotConditionClauseCompatible { value }),
            Value::I32(value) => Ok(ConditionClauseValue::I32(value)),
            Value::U32(value) => Ok(ConditionClauseValue::U32(value)),
            Value::I64(value) => Ok(ConditionClauseValue::I64(value)),
            Value::U64(value) => Ok(ConditionClauseValue::U64(value)),
            Value::I128(value) => Ok(ConditionClauseValue::I128(value)),
            Value::U128(value) => Ok(ConditionClauseValue::U128(value)),
            value @ Value::OptionalI32(_) => Err(NotConditionClauseCompatible { value }),
            value @ Value::OptionalI64(_) => Err(NotConditionClauseCompatible { value }),
            value @ Value::OptionalU32(_) => Err(NotConditionClauseCompatible { value }),
            value @ Value::OptionalU64(_) => Err(NotConditionClauseCompatible { value }),
            value @ Value::OptionalI128(_) => Err(NotConditionClauseCompatible { value }),
            value @ Value::OptionalU128(_) => Err(NotConditionClauseCompatible { value }),
            Value::F32(value) => Ok(ConditionClauseValue::F32(value)),
            Value::F64(value) => Ok(ConditionClauseValue::F64(value)),
            Value::Bool(value) => Ok(ConditionClauseValue::Bool(value)),
            value @ Value::ValidationStatus(_) => Err(NotConditionClauseCompatible { value }),
            value @ Value::PrimitiveDateTime(_) => Err(NotConditionClauseCompatible { value }),
            value @ Value::OffsetDateTime(_) => Err(NotConditionClauseCompatible { value }),
            value @ Value::OptionalPrimitiveDateTime(_) => {
                Err(NotConditionClauseCompatible { value })
            }
            value @ Value::OptionalOffsetDateTime(_) => Err(NotConditionClauseCompatible { value }),
            value @ Value::OneToOneRelation(_) => Err(NotConditionClauseCompatible { value }),
            value @ Value::Reference(_) => Err(NotConditionClauseCompatible { value }),
            value @ Value::Custom(_) => Err(NotConditionClauseCompatible { value }),
            value @ Value::Select(_) => Err(NotConditionClauseCompatible { value }),
            value @ Value::Multiselect(_) => Err(NotConditionClauseCompatible { value }),
            value @ Value::OptionalSelect(_) => Err(NotConditionClauseCompatible { value }),
            value @ Value::OptionalMultiselect(_) => Err(NotConditionClauseCompatible { value }),
            value @ Value::Duration(_) => Err(NotConditionClauseCompatible { value }),
            value @ Value::OptionalDuration(_) => Err(NotConditionClauseCompatible { value }),
        }
    }
}
