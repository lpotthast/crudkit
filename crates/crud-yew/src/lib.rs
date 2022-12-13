use async_trait::async_trait;
use chrono_utc_date_time::prelude::*;
use crud_shared_types::{prelude::*, id::SerializableId};
use dyn_clone::DynClone;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    any::Any,
    fmt::{Debug, Display},
    hash::Hash,
};
use types::RequestError;
use wasm_bindgen::JsCast;
use web_sys::KeyboardEvent;
use yew::prelude::*;

pub mod crud_action;
pub mod crud_alert;
pub mod crud_btn;
pub mod crud_btn_group;
pub mod crud_btn_name;
pub mod crud_btn_wrapper;
pub mod crud_collapsible;
pub mod crud_create_view;
pub mod crud_datetime;
pub mod crud_datetime_date_selector;
pub mod crud_datetime_time_selector;
pub mod crud_delete_modal;
pub mod crud_edit_view;
pub mod crud_field;
pub mod crud_field_label;
pub mod crud_fields;
pub mod crud_icon;
pub mod crud_image_chooser_modal;
pub mod crud_image_gallery;
pub mod crud_instance;
pub mod crud_leave_modal;
pub mod crud_list_view;
pub mod crud_modal;
pub mod crud_modal_host;
pub mod crud_nested_instance;
pub mod crud_pagination;
pub mod crud_progress_bar;
pub mod crud_quicksearch;
pub mod crud_read_view;
pub mod crud_related_field;
pub mod crud_relation;
pub mod crud_reset_field;
pub mod crud_safe_html;
pub mod crud_select;
pub mod crud_select_field;
pub mod crud_separator;
pub mod crud_slider;
pub mod crud_tab;
pub mod crud_table;
pub mod crud_table_body;
pub mod crud_table_footer;
pub mod crud_table_header;
pub mod crud_tabs;
pub mod crud_tiptap_editor;
pub mod crud_toast;
pub mod crud_toasts;
pub mod crud_toggle;
pub mod crud_tree;
pub mod js_tiptap;

pub mod services;
pub mod stores;
pub mod types;

mod event_functions;

pub mod prelude {
    pub use derive_crud_resource::CrudResource;
    pub use derive_crud_selectable::CrudSelectable;
    pub use derive_crud_action_payload::CrudActionPayload;
    pub use derive_field::Field;
    pub use derive_field_value::FieldValue;

    pub use super::crud_action::CrudAction;
    pub use super::crud_action::CrudActionAftermath;
    pub use super::crud_action::CrudActionTrait;
    pub use super::crud_action::CrudEntityAction;
    pub use super::crud_action::States;
    pub use super::crud_alert::CrudAlert;
    pub use super::crud_btn::CrudBtn;
    pub use super::crud_btn_group::CrudBtnGroup;
    pub use super::crud_btn_name::CrudBtnName;
    pub use super::crud_btn_wrapper::CrudBtnWrapper;
    pub use super::crud_collapsible::CrudCollapsible;
    pub use super::crud_create_view::CrudCreateView;
    pub use super::crud_datetime::CrudDatetime;
    pub use super::crud_datetime_date_selector::CrudDatetimeDateSelector;
    pub use super::crud_datetime_time_selector::CrudDatetimeTimeSelector;
    pub use super::crud_delete_modal::CrudDeleteModal;
    pub use super::crud_edit_view::CrudEditView;
    pub use super::crud_field::CrudField;
    pub use super::crud_field_label::CrudFieldLabel;
    pub use super::crud_fields::CrudFields;
    pub use super::crud_icon::CrudIcon;
    pub use super::crud_image_chooser_modal::CrudImageChooserModal;
    pub use super::crud_image_gallery::CrudImageGallery;
    pub use super::crud_instance::CreateElements;
    pub use super::crud_instance::CrudInstance;
    pub use super::crud_instance::CrudInstanceConfig;
    pub use super::crud_instance::CrudStaticInstanceConfig;
    pub use super::crud_leave_modal::CrudLeaveModal;
    pub use super::crud_list_view::CrudListView;
    pub use super::crud_modal::CrudModal;
    pub use super::crud_modal_host::CrudModalHost;
    pub use super::crud_nested_instance::CrudNestedInstance;
    pub use super::crud_pagination::CrudPagination;
    pub use super::crud_progress_bar::CrudProgressBar;
    pub use super::crud_quicksearch::CrudQuickSearch;
    pub use super::crud_quicksearch::CrudQuickSearchOption;
    pub use super::crud_read_view::CrudReadView;
    pub use super::crud_related_field::CrudRelatedField;
    pub use super::crud_relation::CrudRelation;
    pub use super::crud_reset_field::CrudResetField;
    pub use super::crud_safe_html::CrudSafeHtml;
    pub use super::crud_select::CrudSelect;
    pub use super::crud_select_field::CrudSelectField;
    pub use super::crud_separator::CrudSeparator;
    pub use super::crud_slider::CrudSlider;
    pub use super::crud_tab::CrudTab;
    pub use super::crud_table::CrudTable;
    pub use super::crud_table_body::CrudTableBody;
    pub use super::crud_table_footer::CrudTableFooter;
    pub use super::crud_table_header::CrudTableHeader;
    pub use super::crud_tabs::CrudTabs;
    pub use super::crud_tiptap_editor::CrudTipTapEditor;
    pub use super::crud_toast::CrudToast;
    pub use super::crud_toasts::CrudToasts;
    pub use super::crud_toggle::{CrudToggle, CrudToggleIcons};
    pub use super::crud_tree::CrudTree;
    pub use super::types::toasts::Toast;
    pub use super::types::toasts::ToastAutomaticallyClosing;
    pub use super::types::toasts::ToastVariant;
    pub use super::CrudDataTrait;
    pub use super::CrudActionPayload;
    pub use super::EmptyActionPayload;
    pub use super::CrudFieldNameTrait;
    pub use super::CrudFieldValueTrait;
    pub use super::CrudIdTrait;
    pub use super::CrudMainTrait;
    pub use super::CrudResourceTrait;
    pub use super::CrudSelectableSource;
    pub use super::CrudSimpleView;
    pub use super::CrudSelectableTrait;
    pub use super::DeletableModel;
    pub use super::CrudView;
    pub use super::SerializableCrudView;
    pub use super::Elem;
    pub use super::Enclosing;
    pub use super::FieldMode;
    pub use super::FieldOptions;
    pub use super::Group;
    pub use super::HeaderOptions;
    pub use super::Label;
    pub use super::Layout;
    pub use super::NoData;
    pub use super::OrderByUpdateOptions;
    pub use super::Tab;
    pub use super::Value;
    pub use super::Variant;
}

#[derive(Debug, Clone, PartialEq)]
pub enum NoData {
    NotYetLoaded,
    FetchFailed(RequestError),
    // TODO: Can probably be deleted at some point...
    FetchReturnedNothing,
    CreateFailed(RequestError),
    // TODO: Can probably be deleted at some point...
    CreateReturnedNothing,
    UpdateFailed(RequestError),
    // TODO: Can probably be deleted at some point...
    UpdateReturnedNothing,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Variant {
    Default, // TODO: Consider to remove this variant...
    Primary,
    Secondary,
    Success,
    Info,
    Warn,
    Danger,
}

// TODO: This con be computed statically!
impl From<Variant> for Classes {
    fn from(variant: Variant) -> Self {
        match variant {
            Variant::Default => classes!("type-default"),
            Variant::Primary => classes!("type-primary"),
            Variant::Secondary => classes!("type-secondary"),
            Variant::Success => classes!("type-success"),
            Variant::Info => classes!("type-default"),
            Variant::Warn => classes!("type-warn"),
            Variant::Danger => classes!("type-danger"),
        }
    }
}

// TODO: impl Clone if both types are clone, same for debug, ...
pub trait CrudMainTrait: CrudResourceTrait + PartialEq + Default + Debug + Clone + Serialize + Send {

    type CreateModel: CrudDataTrait + Send;

    type ReadModelIdField: IdField<Value = crud_shared_types::IdValue> + Serialize + Send;
    type ReadModelId: Serialize + DeserializeOwned + Id<Field = Self::ReadModelIdField> + PartialEq + Clone + Send;
    type ReadModel: Serialize + CrudDataTrait
        + Into<Self::UpdateModel>
        + CrudIdTrait<Id = Self::ReadModelId>
        + Send;

    type UpdateModelIdField: IdField<Value = crud_shared_types::IdValue> + Serialize + Send;
    type UpdateModelId: Serialize + DeserializeOwned + Id<Field = Self::UpdateModelIdField> + PartialEq + Clone + Send;
    type UpdateModel: Serialize + CrudDataTrait
        + CrudIdTrait<Id = Self::UpdateModelId>
        + Send;

    type ActionPayload: Serialize + CrudActionPayload;
}

pub trait CrudActionPayload:
    PartialEq
    + Clone
    + Debug
    + Serialize
    + DeserializeOwned
    + Send {
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub struct EmptyActionPayload {}

impl CrudActionPayload for EmptyActionPayload {}

// Note: This does not have CrudIdTrait as a super trait, as not all data model (namely the CreateModel) can supply an ID!
pub trait CrudDataTrait:
    Default
    + PartialEq
    + Clone
    + Debug
    + Serialize
    + DeserializeOwned
    + Send
{
    type Field: CrudFieldNameTrait
        + CrudFieldValueTrait<Self>
        + PartialEq
        + Eq
        + Hash
        + Clone
        + Debug
        + Serialize
        + DeserializeOwned
        + Send;

    fn get_field(field_name: &str) -> Self::Field;
}

/// Allows us to access the ID of an entity.
/// The ID type must provide more fine grained access (for example to individual fields).
pub trait CrudIdTrait {
    type Id: Id;

    fn get_id(&self) -> Self::Id;
}

pub trait CrudResourceTrait {
    fn get_resource_name() -> &'static str
    where
        Self: Sized;
}

#[async_trait]
pub trait CrudSelectableSource: Debug {
    type Selectable: CrudSelectableTrait;

    fn new() -> Self;

    async fn load(
    ) -> Result<Vec<Self::Selectable>, Box<dyn std::error::Error + Send + Sync + 'static>>;

    fn set_selectable(&mut self, selectable: Vec<Self::Selectable>);

    /// May return None if selectable options were not yet loaded.
    fn get_selectable(&self) -> Option<Vec<Self::Selectable>>;
}

//#[typetag::serde(tag = "type")]
pub trait CrudSelectableTrait: Debug + Display + DynClone{
    fn as_any(&self) -> &dyn Any;
}
dyn_clone::clone_trait_object!(CrudSelectableTrait);

pub trait Foo {

    fn as_any(&self) -> &dyn Any;
}

/// All variants should be stateless / copy-replaceable.
// TODO: DEFERRED: Implement Serialize and Deserialize with typetag when wasm is supported in typetag. Comment in "typetag" occurrences.
#[derive(Debug, Clone)]
pub enum Value {
    String(String), // TODO: Add optional string!
    Text(String),   // TODO: Add optional text!
    Json(JsonValue), // TODO: Add optional json value
    OptionalJson(Option<JsonValue>),
    UuidV4(uuid::Uuid), // TODO: Add optional UuidV4 value
    UuidV7(uuid::Uuid), // TODO: Add optional UuidV7 value
    Ulid(ulid::Ulid), // TODO: Add optional Ulid value
    U32(u32),
    OptionalU32(Option<u32>),
    I32(i32),
    I64(i64),
    OptionalI64(Option<i64>),
    F32(f32),
    Bool(bool),
    // Specialized bool-case, render as a green check mark if false and an orange exclamation mark if true.
    ValidationStatus(bool),
    UtcDateTime(UtcDateTime),
    OptionalUtcDateTime(Option<UtcDateTime>),
    OneToOneRelation(Option<u32>),
    NestedTable(Vec<Box<dyn DynIdField>>),
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

impl Into<Value> for crud_shared_types::Value {
    fn into(self) -> Value {
        match self {
            crud_shared_types::Value::String(value) => Value::String(value), // TODO: How can we differentiate between String and Text?
            crud_shared_types::Value::Json(value) => Value::Json(JsonValue::new(value)),
            crud_shared_types::Value::UuidV4(value) => Value::UuidV4(value),
            crud_shared_types::Value::UuidV7(value) => Value::UuidV7(value),
            crud_shared_types::Value::Ulid(value) => Value::Ulid(value),
            crud_shared_types::Value::I32(value) => Value::I32(value),
            crud_shared_types::Value::I64(value) => Value::I64(value),
            crud_shared_types::Value::U32(value) => Value::U32(value),
            crud_shared_types::Value::F32(value) => Value::F32(value),
            crud_shared_types::Value::Bool(value) => Value::Bool(value),
            crud_shared_types::Value::DateTime(value) => Value::UtcDateTime(value),
        }
    }
}

impl Into<Value> for crud_shared_types::IdValue {
    fn into(self) -> Value {
        match self {
            crud_shared_types::IdValue::String(value) => Value::String(value), // TODO: How can we differentiate between String and Text?
            crud_shared_types::IdValue::UuidV4(value) => Value::UuidV4(value),
            crud_shared_types::IdValue::UuidV7(value) => Value::UuidV7(value),
            crud_shared_types::IdValue::Ulid(value) => Value::Ulid(value),
            crud_shared_types::IdValue::I32(value) => Value::I32(value),
            crud_shared_types::IdValue::I64(value) => Value::I64(value),
            crud_shared_types::IdValue::U32(value) => Value::U32(value),
            crud_shared_types::IdValue::Bool(value) => Value::Bool(value),
            crud_shared_types::IdValue::DateTime(value) => Value::UtcDateTime(value),
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
    pub fn take_text(self) -> String {
        match self {
            Self::Text(string) => string,
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_inner_json_value(self) -> serde_json::Value {
        match self {
            Self::Json(json) => json.into(),
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_u32(self) -> u32 {
        match self {
            Self::U32(u32) => u32,
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
    pub fn take_i32(self) -> i32 {
        match self {
            Self::I32(i32) => i32,
            // This has some potential data loss...
            // TODO: Can we remove this? Without, this created a panic in fcs/servers/labels/new
            Self::U32(u32) => u32 as i32,
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_i64(self) -> i64 {
        match self {
            Self::I64(i64) => i64,
            // This has some potential data loss...
            // TODO: Can we remove this? Without, this created a panic in fcs/servers/labels/new
            //Self::U32(u32) => u32 as i32,
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_optional_i64(self) -> Option<i64> {
        match self {
            Self::I64(value) => Some(value),
            Self::OptionalI64(value) => value,
            Self::String(string) => string
                .parse::<i64>()
                .map_err(|err| log::warn!("take_optional_i64 could not pase string: {err}"))
                .ok(),
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_f32(self) -> f32 {
        match self {
            Self::F32(f32) => f32,
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
    pub fn take_date_time(self) -> UtcDateTime {
        match self {
            Self::UtcDateTime(utc_date_time) => utc_date_time,
            Self::String(string) => UtcDateTime::parse_from_rfc3339(&string).unwrap(),
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_optional_date_time(self) -> Option<UtcDateTime> {
        match self {
            Self::UtcDateTime(utc_date_time) => Some(utc_date_time),
            Self::OptionalUtcDateTime(optional_utc_date_time) => optional_utc_date_time,
            // TODO: We might want to catch parsing errors and return an empty optional here.
            Self::String(string) => Some(UtcDateTime::parse_from_rfc3339(&string).unwrap()),
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
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::String(value) => f.write_str(value),
            Value::Text(value) => f.write_str(value),
            Value::Json(value) => f.write_str(value.get_string_representation()),
            Value::OptionalJson(value) => match value {
                Some(value) => f.write_str(value.get_string_representation()),
                None => f.write_str("-")
            },
            Value::UuidV4(value) => f.write_str(&value.to_string()),
            Value::UuidV7(value) => f.write_str(&value.to_string()),
            Value::Ulid(value) => f.write_str(&value.to_string()),
            Value::U32(value) => f.write_str(&value.to_string()),
            Value::OptionalU32(value) => match value {
                Some(value) => f.write_str(&value.to_string()),
                None => f.write_str("-"),
            },
            Value::I32(value) => f.write_str(&value.to_string()),
            Value::I64(value) => f.write_str(&value.to_string()),
            Value::OptionalI64(value) => match value {
                Some(value) => f.write_str(&value.to_string()),
                None => f.write_str("-"),
            },
            Value::F32(value) => f.write_str(&value.to_string()),
            Value::Bool(value) => f.write_str(&value.to_string()),
            Value::ValidationStatus(value) => f.write_str(&value.to_string()),
            Value::UtcDateTime(value) => f.write_str(&value.to_rfc3339()),
            Value::OptionalUtcDateTime(value) => match value {
                Some(value) => f.write_str(&value.to_rfc3339()),
                None => f.write_str(""),
            },
            Value::OneToOneRelation(value) => match value {
                Some(value) => f.write_str(&value.to_string()),
                None => f.write_str(""),
            },
            Value::NestedTable(id) => {
                for field in id {
                    f.write_fmt(format_args!("'{}': {:?}", field.dyn_name(), field.into_dyn_value()))?;
                }
                Ok(())
            },
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
            Value::Text(value) => ConditionClauseValue::String(value),
            Value::Json(value) => ConditionClauseValue::Json(value.into()),
            Value::OptionalJson(value) => todo!(),
            Value::UuidV4(value) => ConditionClauseValue::UuidV4(value),
            Value::UuidV7(value) => ConditionClauseValue::UuidV7(value),
            Value::Ulid(value) => ConditionClauseValue::Ulid(value),
            Value::U32(value) => ConditionClauseValue::U32(value),
            Value::OptionalU32(value) => todo!(),
            Value::I32(value) => ConditionClauseValue::I32(value),
            Value::I64(value) => ConditionClauseValue::I64(value),
            Value::OptionalI64(value) => todo!(),
            Value::F32(value) => ConditionClauseValue::F32(value),
            Value::Bool(value) => ConditionClauseValue::Bool(value),
            Value::ValidationStatus(value) => todo!(),
            Value::UtcDateTime(value) => todo!(),
            Value::OptionalUtcDateTime(value) => todo!(),
            Value::OneToOneRelation(value) => todo!(),
            Value::NestedTable(value) => todo!(),
            Value::Select(value) => todo!(),
            Value::Multiselect(value) => todo!(),
            Value::OptionalSelect(value) => todo!(),
            Value::OptionalMultiselect(value) => todo!(),
        }
    }
}

// TODO: Remove
//#[typetag::serde]
/*
impl crud_shared_types::id::IdFieldValue for Value {
    fn into_condition_clause_value(&self) -> ConditionClauseValue {
        // Note: This requires clone, because we take &self. We take &self, so that the trait remains dynamically usable.
        self.clone().into()
    }
}
*/

pub trait CrudFieldNameTrait {
    fn get_name(&self) -> &'static str;
}

pub trait CrudFieldValueTrait<T> {
    fn get_value(&self, entity: &T) -> Value;
    fn set_value(&self, entity: &mut T, value: Value);
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CrudView<ReadId, UpdateId>
where
    ReadId: Id + Serialize + DeserializeOwned,
    UpdateId: Id + Serialize + DeserializeOwned,
{
    List,
    Create,
    #[serde(bound = "")]
    Read(ReadId),
    #[serde(bound = "")]
    Edit(UpdateId),
}


// TODO :PartialEq
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SerializableCrudView {
    List,
    Create,
    #[serde(bound = "")]
    Read(SerializableId),
    #[serde(bound = "")]
    Edit(SerializableId),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CrudSimpleView {
    List,
    Create,
    Read,
    Edit,
}

impl<ReadId, UpdateId> Into<SerializableCrudView> for CrudView<ReadId, UpdateId>
where
    ReadId: Id + Serialize + DeserializeOwned,
    UpdateId: Id + Serialize + DeserializeOwned,
{
    fn into(self) -> SerializableCrudView {
        match self {
            CrudView::List => SerializableCrudView::List,
            CrudView::Create => SerializableCrudView::Create,
            CrudView::Read(id) => SerializableCrudView::Read(id.into_serializable_id()),
            CrudView::Edit(id) => SerializableCrudView::Edit(id.into_serializable_id()),
        }
    }
}

impl<ReadId, UpdateId> Into<CrudSimpleView> for CrudView<ReadId, UpdateId>
where
    ReadId: Id + Serialize + DeserializeOwned,
    UpdateId: Id + Serialize + DeserializeOwned,
{
    fn into(self) -> CrudSimpleView {
        match self {
            CrudView::List => CrudSimpleView::List,
            CrudView::Create => CrudSimpleView::Create,
            CrudView::Read(_) => CrudSimpleView::Read,
            CrudView::Edit(_) => CrudSimpleView::Edit,
        }
    }
}

impl <ReadId, UpdateId> Default for CrudView<ReadId, UpdateId>
where
    ReadId: Id + Serialize + DeserializeOwned,
    UpdateId: Id + Serialize + DeserializeOwned,
{
    fn default() -> Self {
        Self::List
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FieldMode {
    Display,
    Readable,
    Editable,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DeletableModel<ReadModel: CrudDataTrait + CrudIdTrait, UpdateModel: CrudDataTrait + CrudIdTrait> {
    Read(ReadModel),
    Update(UpdateModel),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HeaderOptions {
    pub display_name: String,
    pub min_width: bool,
    pub ordering_allowed: bool,
    pub date_time_display: DateTimeDisplay,
}

// TODO: we might want to use the builder pattern instead of relying on ..Default.default()
impl Default for HeaderOptions {
    fn default() -> Self {
        Self {
            display_name: Default::default(),
            min_width: false,
            ordering_allowed: true,
            date_time_display: DateTimeDisplay::LocalizedLocal,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum DateTimeDisplay {
    IsoUtc,
    LocalizedLocal,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Label {
    name: String,
}

impl Label {
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self { name: name.into() }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FieldOptions {
    pub disabled: bool,
    pub label: Option<Label>,
    pub date_time_display: DateTimeDisplay,
    //validations: Vec<u32>,
}

impl Default for FieldOptions {
    fn default() -> Self {
        Self {
            disabled: false,
            label: None,
            date_time_display: DateTimeDisplay::LocalizedLocal,
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct OrderByUpdateOptions {
    pub append: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Elem<T: CrudDataTrait> {
    // serde bound used as described in: https://github.com/serde-rs/serde/issues/1296
    #[serde(bound = "")]
    Enclosing(Enclosing<T>),
    Field((T::Field, FieldOptions)),
    Separator,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tab<T: CrudDataTrait> {
    pub label: Label,
    #[serde(bound = "")]
    pub group: Group<T>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Enclosing<T: CrudDataTrait> {
    #[serde(bound = "")]
    None(Group<T>),
    #[serde(bound = "")]
    Tabs(Vec<Tab<T>>),
    #[serde(bound = "")]
    Card(Group<T>),
}

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

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Group<T: CrudDataTrait> {
    pub layout: Layout,
    // serde bound used as described in: https://github.com/serde-rs/serde/issues/1296
    #[serde(bound = "")]
    pub children: Vec<Elem<T>>,
}

pub fn event_target_as<T: JsCast>(event: Event) -> Result<T, String> {
    event
        .target()
        .ok_or_else(|| format!("Unable to obtain target from event: {:?}", event))
        .and_then(|event_target| {
            event_target
                .dyn_into::<T>()
                .map_err(|err| format!("Unable to cast event_target to T: {:?}", err.to_string()))
        })
}

pub fn keyboard_event_target_as<T: JsCast>(event: KeyboardEvent) -> Result<T, String> {
    event
        .target()
        .ok_or_else(|| format!("Unable to obtain target from event: {:?}", event))
        .and_then(|event_target| {
            event_target
                .dyn_into::<T>()
                .map_err(|err| format!("Unable to cast event_target to T: {:?}", err.to_string()))
        })
}
