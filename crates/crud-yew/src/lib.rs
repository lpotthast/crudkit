use chrono_utc_date_time::prelude::*;
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
use yewbi::Bi;

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
    pub use derive_field::Field;

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
    pub use super::crud_instance::CrudInstance;
    pub use super::crud_instance::CrudInstanceConfig;
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
    pub use super::CrudActionTrait;
    pub use super::CrudDataTrait;
    pub use super::CrudFieldNameTrait;
    pub use super::CrudFieldTrait;
    pub use super::CrudFieldValueTrait;
    pub use super::CrudIdTrait;
    pub use super::CrudMainTrait;
    pub use super::CrudResourceTrait;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
pub trait CrudMainTrait: CrudResourceTrait + PartialEq + Default + Debug + Clone {
    type ReadModel: CrudDataTrait + Into<Self::UpdateModel>;
    type UpdateModel: CrudDataTrait;
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
        + DeserializeOwned;
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

pub trait CrudSelectableTrait: Debug + Display {
    fn as_any(&self) -> &dyn Any;
}

#[derive(Debug)]
pub enum Value {
    String(String), // TODO: Add optional string!
    Text(String),
    U32(u32),
    I32(i32),
    Bool(bool),
    // Specialized bool-case, render as a green check mark if false and an orange exclamation mark if true.
    ValidationStatus(bool),
    UtcDateTime(UtcDateTime),
    OptionalUtcDateTime(Option<UtcDateTime>),
    OneToOneRelation(Option<u32>),
    NestedTable(u32),
    Select(Option<Box<dyn CrudSelectableTrait>>),
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
    pub fn take_bool(self) -> bool {
        match self {
            Self::Bool(bool) => bool,
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_bool_or_parse(self) -> bool {
        match self {
            Self::Bool(bool) => bool,
            Self::String(string) => string.parse().unwrap(),
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
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
    pub fn take_select_downcast_to<T: Clone + 'static>(self) -> Option<T> {
        match self {
            Self::Select(value) => {
                value.map(|value| value.as_any().downcast_ref::<T>().unwrap().clone())
            }
            _ => panic!("unsupported type provided"),
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::String(string) => f.write_str(string),
            Value::Text(string) => f.write_str(string),
            Value::U32(u32) => f.write_str(&u32.to_string()),
            Value::I32(i32) => f.write_str(&i32.to_string()),
            Value::Bool(bool) => f.write_str(&bool.to_string()),
            Value::ValidationStatus(bool) => f.write_str(&bool.to_string()),
            Value::UtcDateTime(utc_date_time) => f.write_str(&utc_date_time.to_rfc3339()),
            Value::OptionalUtcDateTime(optional_utc_date_time) => match optional_utc_date_time {
                Some(utc_date_time) => f.write_str(&utc_date_time.to_rfc3339()),
                None => f.write_str(""),
            },
            Value::OneToOneRelation(option_u32) => match option_u32 {
                Some(u32) => f.write_str(&u32.to_string()),
                None => f.write_str(""),
            },
            Value::NestedTable(u32) => f.write_str(&u32.to_string()),
            Value::Select(optional_value) => match optional_value {
                Some(value) => f.write_str(&value.to_string()),
                None => f.write_str("NULL"),
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

pub trait CrudActionTrait: Debug {
    fn get_name(&self) -> String;
    fn get_icon(&self) -> Option<Bi>;
    fn eq(&self, other: &dyn CrudActionTrait) -> bool;
}

#[derive(PartialEq, Debug)]
pub struct ShowListViewAction {
    name: String,
    icon: Option<Bi>,
}

impl Default for ShowListViewAction {
    fn default() -> Self {
        Self {
            name: "List".to_owned(),
            icon: Some(Bi::List),
        }
    }
}

impl CrudActionTrait for ShowListViewAction {
    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn get_icon(&self) -> Option<Bi> {
        self.icon
    }

    fn eq(&self, other: &dyn CrudActionTrait) -> bool {
        self.get_icon() == other.get_icon() && self.get_name() == other.get_name()
    }
}

#[derive(PartialEq, Debug)]
pub struct ShowReadViewAction {
    name: String,
    icon: Option<Bi>,
}

impl Default for ShowReadViewAction {
    fn default() -> Self {
        Self {
            name: "Read".to_owned(),
            icon: Some(Bi::Eye),
        }
    }
}

impl CrudActionTrait for ShowReadViewAction {
    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn get_icon(&self) -> Option<Bi> {
        self.icon
    }

    fn eq(&self, other: &dyn CrudActionTrait) -> bool {
        self.get_icon() == other.get_icon() && self.get_name() == other.get_name()
    }
}

#[derive(PartialEq, Debug)]
pub struct ShowEditViewAction {
    name: String,
    icon: Option<Bi>,
}

impl Default for ShowEditViewAction {
    fn default() -> Self {
        Self {
            name: "Edit".to_owned(),
            icon: Some(Bi::Pencil),
        }
    }
}

impl CrudActionTrait for ShowEditViewAction {
    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn get_icon(&self) -> Option<Bi> {
        self.icon
    }

    fn eq(&self, other: &dyn CrudActionTrait) -> bool {
        self.get_icon() == other.get_icon() && self.get_name() == other.get_name()
    }
}

#[derive(PartialEq, Debug)]
pub struct DeleteAction {
    name: String,
    icon: Option<Bi>,
}

impl Default for DeleteAction {
    fn default() -> Self {
        Self {
            name: "Delete".to_owned(),
            icon: Some(Bi::Trash),
        }
    }
}

impl CrudActionTrait for DeleteAction {
    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn get_icon(&self) -> Option<Bi> {
        self.icon
    }

    fn eq(&self, other: &dyn CrudActionTrait) -> bool {
        self.get_icon() == other.get_icon() && self.get_name() == other.get_name()
    }
}

impl PartialEq for dyn CrudActionTrait + '_ {
    fn eq(&self, that: &dyn CrudActionTrait) -> bool {
        CrudActionTrait::eq(self, that)
    }
}

impl PartialEq<dyn CrudActionTrait> for Box<dyn CrudActionTrait + '_> {
    fn eq(&self, that: &dyn CrudActionTrait) -> bool {
        CrudActionTrait::eq(&**self, that)
    }
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
