use async_trait::async_trait;
use chrono_utc_date_time::prelude::*;
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
    pub use super::CrudFieldNameTrait;
    pub use super::CrudFieldTrait;
    pub use super::CrudFieldValueTrait;
    pub use super::CrudIdTrait;
    pub use super::CrudMainTrait;
    pub use super::CrudResourceTrait;
    pub use super::CrudSelectableSource;
    pub use super::CrudSelectableTrait;
    pub use super::CrudView;
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
pub trait CrudMainTrait: CrudResourceTrait + PartialEq + Default + Debug + Clone + Send {
    type CreateModel: CrudDataTrait + Send;
    type ReadModel: CrudDataTrait + Into<Self::UpdateModel> + Send;
    type UpdateModel: CrudDataTrait + Send;
}

pub trait CrudDataTrait:
    CrudFieldTrait<Self::Field, Self>
    + CrudIdTrait<Self::Field, Self>
    + Default
    + PartialEq
    + Clone
    + Debug
    + Serialize
    + DeserializeOwned
    + Send
{
    type Field: CrudFieldNameTrait
        + CrudFieldValueTrait<Self>
        + Default
        + PartialEq
        + Eq
        + Hash
        + Clone
        + Debug
        + Serialize
        + DeserializeOwned
        + Send;
}

// TODO: Rename to CrudFieldAccessTrait
pub trait CrudFieldTrait<F: CrudFieldValueTrait<C>, C: CrudDataTrait> {
    fn get_field(field_name: &str) -> F;
}

pub trait CrudIdTrait<F: CrudFieldValueTrait<C>, C: CrudDataTrait> {
    fn get_id_field() -> F;
    fn get_id(&self) -> u32;
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

pub trait CrudSelectableTrait: Debug + Display + DynClone {
    fn as_any(&self) -> &dyn Any;
}
dyn_clone::clone_trait_object!(CrudSelectableTrait);

/// All variants should be stateless / copy-replaceable.
#[derive(Debug, Clone)]
pub enum Value {
    String(String), // TODO: Add optional string!
    Text(String),   // TODO: Add optional text!
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
    NestedTable(u32),
    Select(Box<dyn CrudSelectableTrait>),
    Multiselect(Vec<Box<dyn CrudSelectableTrait>>),
    OptionalSelect(Option<Box<dyn CrudSelectableTrait>>),
    OptionalMultiselect(Option<Vec<Box<dyn CrudSelectableTrait>>>),
    //Select(Box<dyn CrudSelectableSource<Selectable = dyn CrudSelectableTrait>>),
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
            Value::NestedTable(value) => f.write_str(&value.to_string()),
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

pub trait CrudFieldNameTrait {
    fn get_name(&self) -> &'static str;
}

pub trait CrudFieldValueTrait<T> {
    fn get_value(&self, entity: &T) -> Value;
    fn set_value(&self, entity: &mut T, value: Value);
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CrudView {
    List,
    Create,
    Read(u32),
    Edit(u32),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FieldMode {
    Display,
    Readable,
    Editable,
}

impl Default for CrudView {
    fn default() -> Self {
        Self::List
    }
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
