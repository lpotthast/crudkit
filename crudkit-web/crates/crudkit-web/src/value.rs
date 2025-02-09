use crate::CrudSelectableTrait;
use crudkit_condition::ConditionClauseValue;
use std::fmt::Display;
use time::format_description::well_known::Rfc3339;
use tracing::warn;

/// All variants should be stateless / copy-replaceable.
// TODO: DEFERRED: Implement Serialize and Deserialize with typetag when wasm is supported in typetag. Comment in "typetag" occurrences.
#[derive(Debug, Clone)]
pub enum Value {
    String(String),
    OptionalString(Option<String>),
    Text(String), // TODO: Add optional text!, TODO: Remove this variant altogether and make "text" an optional editing mode for string values!
    Json(JsonValue), // TODO: Add optional json value
    OptionalJson(Option<JsonValue>),
    UuidV4(uuid::Uuid), // TODO: Add optional UuidV4 value
    UuidV7(uuid::Uuid), // TODO: Add optional UuidV7 value
    I32(i32),
    I64(i64),
    U32(u32),
    U64(u64),
    OptionalI32(Option<i32>),
    OptionalI64(Option<i64>),
    OptionalU32(Option<u32>),
    OptionalU64(Option<u64>),
    F32(f32),
    F64(f64),
    Bool(bool),
    // Specialized bool-case, render as a green check mark if false and an orange exclamation mark if true.
    ValidationStatus(bool),
    PrimitiveDateTime(time::PrimitiveDateTime),
    OffsetDateTime(time::OffsetDateTime),
    OptionalPrimitiveDateTime(Option<time::PrimitiveDateTime>),
    OptionalOffsetDateTime(Option<time::OffsetDateTime>),
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

impl Into<serde_json::Value> for JsonValue {
    fn into(self) -> serde_json::Value {
        self.value
    }
}

impl Into<String> for JsonValue {
    fn into(self) -> String {
        self.string_representation
    }
}

impl Into<Value> for crudkit_shared::Value {
    fn into(self) -> Value {
        match self {
            crudkit_shared::Value::String(value) => Value::String(value), // TODO: How can we differentiate between String and Text?
            crudkit_shared::Value::Json(value) => Value::Json(JsonValue::new(value)),
            crudkit_shared::Value::UuidV4(value) => Value::UuidV4(value),
            crudkit_shared::Value::UuidV7(value) => Value::UuidV7(value),
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
        }
    }
}

impl Into<Value> for crudkit_id::IdValue {
    fn into(self) -> Value {
        match self {
            crudkit_id::IdValue::String(value) => Value::String(value), // TODO: How can we differentiate between String and Text?
            crudkit_id::IdValue::UuidV4(value) => Value::UuidV4(value),
            crudkit_id::IdValue::UuidV7(value) => Value::UuidV7(value),
            crudkit_id::IdValue::I32(value) => Value::I32(value),
            crudkit_id::IdValue::I64(value) => Value::I64(value),
            crudkit_id::IdValue::U32(value) => Value::U32(value),
            crudkit_id::IdValue::U64(value) => Value::U64(value),
            crudkit_id::IdValue::Bool(value) => Value::Bool(value),
            crudkit_id::IdValue::PrimitiveDateTime(value) => Value::PrimitiveDateTime(value),
            crudkit_id::IdValue::OffsetDateTime(value) => Value::OffsetDateTime(value),
        }
    }
}

impl Value {
    pub fn take_string(self) -> String {
        match self {
            Self::String(string) => string,
            Self::Text(string) => string,
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_optional_string(self) -> Option<String> {
        match self {
            Self::OptionalString(string) => string,
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_text(self) -> String {
        match self {
            Self::Text(string) => string,
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_json_value(self) -> JsonValue {
        match self {
            Self::Json(json) => json,
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_inner_json_value(self) -> serde_json::Value {
        match self {
            Self::Json(json) => json.into(),
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_optional_json_value(self) -> Option<JsonValue> {
        match self {
            Self::OptionalJson(json) => json,
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_uuid_v4(self) -> uuid::Uuid {
        match self {
            Self::UuidV4(uuid) => uuid,
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_uuid_v7(self) -> uuid::Uuid {
        match self {
            Self::UuidV7(uuid) => uuid,
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
            // TODO: Can we remove this? Without, this created a panic in agnite/servers/labels/new
            Self::U32(u32) => u32 as i32,
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_i64(self) -> i64 {
        match self {
            Self::I64(i64) => i64,
            // This has some potential data loss...
            // TODO: Can we remove this? Without, this created a panic in agnite/servers/labels/new
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
            other => panic!("Expected Value of variant 'U32', 'OptionalU32' or 'OneToOneRelation'. Received: {other:?}"),
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
            Value::Text(value) => f.write_str(value),
            Value::Json(value) => f.write_str(value.get_string_representation()),
            Value::OptionalJson(value) => match value {
                Some(value) => f.write_str(value.get_string_representation()),
                None => f.write_str("-"),
            },
            Value::UuidV4(value) => f.write_str(&value.to_string()),
            Value::UuidV7(value) => f.write_str(&value.to_string()),
            Value::I32(value) => f.write_str(&value.to_string()),
            Value::I64(value) => f.write_str(&value.to_string()),
            Value::U32(value) => f.write_str(&value.to_string()),
            Value::U64(value) => f.write_str(&value.to_string()),
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
        }
    }
}

impl Into<ConditionClauseValue> for Value {
    fn into(self) -> ConditionClauseValue {
        match self {
            // TODO: Complete mapping!!
            Value::String(value) => ConditionClauseValue::String(value),
            Value::OptionalString(_value) => todo!(),
            Value::Text(value) => ConditionClauseValue::String(value),
            Value::Json(value) => ConditionClauseValue::Json(value.into()),
            Value::OptionalJson(_value) => todo!(),
            Value::UuidV4(value) => ConditionClauseValue::UuidV4(value),
            Value::UuidV7(value) => ConditionClauseValue::UuidV7(value),
            Value::I32(value) => ConditionClauseValue::I32(value),
            Value::I64(value) => ConditionClauseValue::I64(value),
            Value::U32(value) => ConditionClauseValue::U32(value),
            Value::U64(value) => ConditionClauseValue::U64(value),
            Value::OptionalI32(_value) => todo!(),
            Value::OptionalI64(_value) => todo!(),
            Value::OptionalU32(_value) => todo!(),
            Value::OptionalU64(_value) => todo!(),
            Value::F32(value) => ConditionClauseValue::F32(value),
            Value::F64(value) => ConditionClauseValue::F64(value),
            Value::Bool(value) => ConditionClauseValue::Bool(value),
            Value::ValidationStatus(_value) => todo!(),
            Value::PrimitiveDateTime(_value) => todo!(),
            Value::OffsetDateTime(_value) => todo!(),
            Value::OptionalPrimitiveDateTime(_value) => todo!(),
            Value::OptionalOffsetDateTime(_value) => todo!(),
            Value::OneToOneRelation(_value) => todo!(),
            Value::Reference(_value) => todo!(),
            Value::Custom(_value) => todo!(),
            Value::Select(_value) => todo!(),
            Value::Multiselect(_value) => todo!(),
            Value::OptionalSelect(_value) => todo!(),
            Value::OptionalMultiselect(_value) => todo!(),
        }
    }
}
