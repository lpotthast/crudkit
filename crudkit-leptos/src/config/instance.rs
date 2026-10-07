use crate::config::CrudViewRegistry;
use crate::config::FieldRendererRegistry;
use crate::config::{CrudAction, CrudEntityAction};
use crudkit_core::Order;
use crudkit_core::condition::Condition;
use crudkit_core::id::SerializableId;
use crudkit_web::field::HeaderOptions;
use crudkit_web::http::ReqwestExecutor;
use crudkit_web::list::{ItemsPerPage, PageNr};
use crudkit_web::model_handler::ModelHandler;
use crudkit_web::prelude::*;
use crudkit_web::view::CrudView;
use indexmap::IndexMap;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::fmt::Debug;
use std::sync::Arc;

/// Definition of a column, shown in list view.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Header {
    /// Read-model field whose value the column shows and by which it is ordered.
    pub field: DynReadField,
    /// Display name, width, ordering permission, and date-time formatting of the column.
    pub options: HeaderOptions,
}

impl Header {
    /// Creates a column showing `field` with `options`.
    pub fn showing(field: impl ErasedReadField, options: HeaderOptions) -> Header {
        Self {
            field: DynReadField::new(field),
            options,
        }
    }
}

impl From<(DynReadField, HeaderOptions)> for Header {
    fn from((field, options): (DynReadField, HeaderOptions)) -> Self {
        Self { field, options }
    }
}

/// Configuration of one mounted [`CrudInstance`](crate::components::instance::CrudInstance).
///
/// The first group of fields seeds the instance's reactive state when it mounts and is restored
/// by [`CrudInstanceContext::reset`](crate::instance::CrudInstanceContext::reset) where
/// applicable. The second group is treated as static for the lifetime of the mount. Start from
/// [`CrudInstanceConfig::new`] and complete it with struct update syntax.
#[derive(Debug, Clone)]
pub struct CrudInstanceConfig {
    /* Later to-be mutable data. */
    /// Base URL of the CrudKit API. Requests go to `{api_base_url}/{resource_name}/crud/v1/...`.
    pub api_base_url: String,
    /// View used to initialize instance-owned navigation and as the reset destination.
    pub initial_view: CrudView,
    /// Columns of the list view, in display order.
    pub list_columns: Vec<Header>,
    /// Layout of the create view.
    pub create_elements: CreateElements,
    /// Layout of the edit and read views, both of which render the update model.
    pub elements: UpdateElements,
    /// Initial ordering of the list, restored on reset. Earlier entries take precedence; an empty
    /// map leaves the order to the server.
    pub order_by: IndexMap<DynReadField, Order>,
    /// The number of items shown per page in the list view.
    pub items_per_page: ItemsPerPage,
    /// The current page to display, e.g. `PageNr::first()`. One-based index.
    pub page_nr: PageNr,
    /// Condition restricting every list, count, read, and update request of the instance. A
    /// parent condition derived from [`CrudParentConfig`] is combined with it.
    pub base_condition: Option<Condition>,

    /* Immutable data */
    /// Name of the resource, used as path segment of its API routes.
    pub resource_name: String,
    /// Executor sending the instance's HTTP requests.
    pub reqwest_executor: Arc<dyn ReqwestExecutor>,
    /// Type-erased access to the resource's concrete create, read, and update models.
    pub model_handler: ModelHandler,
    /// Actions operating on the resource as a whole, offered by the list view.
    pub actions: Vec<CrudAction>,
    /// Actions operating on the current entity, offered by the views their valid states allow.
    pub entity_actions: Vec<CrudEntityAction>,
    /// Visibility and follow-up behavior for controls in CrudKit's built-in views.
    pub builtin_view_controls: CrudBuiltinViewControls,
    /// Built-in replacements and application-defined view renderers for this instance. Defaults
    /// to the registry provided with [`crate::hooks::views::provide_crud_view_registry`],
    /// or CrudKit's built-in views.
    pub view_registry: Option<CrudViewRegistry>,
    /// Renderer overrides for read-model fields, used by list cells and
    /// [`use_crud_row_fields`](crate::hooks::use_crud_row_fields).
    pub read_field_renderer: FieldRendererRegistry<DynReadField>,
    /// Renderer overrides for create-model fields, used by the create view.
    pub create_field_renderer: FieldRendererRegistry<DynCreateField>,
    /// Renderer overrides for update-model fields, used by the edit and read views.
    pub update_field_renderer: FieldRendererRegistry<DynUpdateField>,
}

impl CrudInstanceConfig {
    /// Creates a configuration of resource `R`, served below `api_base_url` and requested through
    /// `reqwest_executor`.
    ///
    /// Everything else starts empty or at its default: no list columns, layouts, actions, or
    /// registered field renderers, the table as initial view, and the view registry provided by
    /// the application. Complete it with struct update syntax.
    #[must_use]
    pub fn new<R>(
        api_base_url: impl Into<String>,
        reqwest_executor: Arc<dyn ReqwestExecutor>,
    ) -> Self
    where
        R: crudkit_web::resource::Resource,
        R::CreateModel: ErasedCreateModel,
        R::ReadModel: ErasedReadModel,
        R::UpdateModel: ErasedUpdateModel + From<R::ReadModel>,
        <R::ReadModel as Model>::Field: ErasedReadField,
        <R::CreateModel as Model>::Field: ErasedCreateField,
        <R::UpdateModel as Model>::Field: ErasedUpdateField,
    {
        Self {
            api_base_url: api_base_url.into(),
            initial_view: CrudView::default(),
            list_columns: Vec::new(),
            create_elements: CreateElements::None,
            elements: Vec::new(),
            order_by: IndexMap::new(),
            items_per_page: ItemsPerPage::default(),
            page_nr: PageNr::default(),
            base_condition: None,
            resource_name: R::resource_name().to_owned(),
            reqwest_executor,
            model_handler: ModelHandler::new::<R::CreateModel, R::ReadModel, R::UpdateModel>(),
            actions: Vec::new(),
            entity_actions: Vec::new(),
            builtin_view_controls: CrudBuiltinViewControls::default(),
            view_registry: None,
            read_field_renderer: FieldRendererRegistry::builder().build(),
            create_field_renderer: FieldRendererRegistry::builder().build(),
            update_field_renderer: FieldRendererRegistry::builder().build(),
        }
    }

    pub(crate) fn split(self) -> (CrudMutableInstanceConfig, CrudStaticInstanceConfig) {
        (
            CrudMutableInstanceConfig {
                api_base_url: self.api_base_url,
                initial_view: self.initial_view,
                headers: self.list_columns,
                create_elements: self.create_elements,
                elements: self.elements,
                order_by: self.order_by,
                items_per_page: self.items_per_page,
                page: self.page_nr,
                base_condition: self.base_condition,
            },
            CrudStaticInstanceConfig {
                resource_name: self.resource_name,
                reqwest_executor: self.reqwest_executor,
                model_handler: self.model_handler,
                actions: self.actions,
                entity_actions: self.entity_actions,
                builtin_view_controls: self.builtin_view_controls,
                view_registry: self
                    .view_registry
                    .or_else(use_context::<CrudViewRegistry>)
                    .unwrap_or_default(),
                read_field_renderer: self.read_field_renderer,
                create_field_renderer: self.create_field_renderer,
                update_field_renderer: self.update_field_renderer,
            },
        )
    }
}

#[derive(Debug, Clone)] // TODO: Serialize, Deserialize
pub(crate) struct CrudMutableInstanceConfig {
    pub api_base_url: String,
    pub initial_view: CrudView,
    pub headers: Vec<Header>,
    pub create_elements: CreateElements,
    pub elements: UpdateElements,
    pub order_by: IndexMap<DynReadField, Order>,
    pub items_per_page: ItemsPerPage,
    pub page: PageNr,
    pub base_condition: Option<Condition>,
}

/// This config is non-serializable. Every piece of runtime-changing data relevant to be tracked and reloaded should be part of the `CrudInstanceConfig` struct.
#[derive(Debug, Clone)]
pub(crate) struct CrudStaticInstanceConfig {
    pub resource_name: String,
    pub reqwest_executor: Arc<dyn ReqwestExecutor>,
    pub model_handler: ModelHandler,
    pub actions: Vec<CrudAction>,
    pub entity_actions: Vec<CrudEntityAction>,
    pub builtin_view_controls: CrudBuiltinViewControls,
    pub view_registry: CrudViewRegistry,
    pub read_field_renderer: FieldRendererRegistry<DynReadField>,
    pub create_field_renderer: FieldRendererRegistry<DynCreateField>,
    pub update_field_renderer: FieldRendererRegistry<DynUpdateField>,
}

/// Follow-up performed after a successful create operation.
#[derive(Debug, Clone, Copy)]
pub enum CrudCreateSaveTarget {
    /// Open the built-in edit view for the created entity.
    EditView,
    /// Resolve the created id to an arbitrary view.
    View(Callback<SerializableId, CrudView>),
    /// Perform the navigation object's configured return action.
    Return,
    /// Keep the create view mounted.
    Stay,
}

impl CrudCreateSaveTarget {
    /// Opens one fixed view after creation.
    #[must_use]
    pub fn view(view: CrudView) -> Self {
        Self::View(Callback::new(move |_| view.clone()))
    }

    /// Builds the destination from the created entity id.
    pub fn dynamic(callback: impl Fn(SerializableId) -> CrudView + Send + Sync + 'static) -> Self {
        Self::View(Callback::new(callback))
    }
}

/// Placement of CrudKit action controls relative to the built-in form.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CrudActionsPlacement {
    /// Render controls inside the built-in form.
    Inline,
    /// Publish controls through an application-placed actions outlet.
    External,
}

/// Controls which buttons CrudKit's built-in views render.
#[derive(Debug, Clone, Copy)]
// Each flag toggles one independent control, so the flags do not form a state machine.
#[allow(clippy::struct_excessive_bools)]
pub struct CrudBuiltinViewControls {
    /// Whether create and edit views show the primary save button.
    pub show_save: bool,
    /// Whether create and edit views show the save-and-return button.
    pub show_save_and_back: bool,
    /// Whether the create view shows the save-and-create-another button.
    pub show_save_and_new: bool,
    /// Whether table, read, and edit views show delete controls.
    pub show_delete: bool,
    /// Whether read and edit views show the configured return action.
    pub show_return: bool,
    /// Follow-up performed after a successful create operation.
    pub create_save_target: CrudCreateSaveTarget,
    /// Placement of create action controls.
    pub create_actions_placement: CrudActionsPlacement,
}

impl CrudBuiltinViewControls {
    /// Returns controls suitable for a single entity embedded in an application-owned host.
    #[must_use]
    pub fn embedded_single_entity() -> Self {
        Self {
            show_save: true,
            show_save_and_back: false,
            show_save_and_new: false,
            show_delete: false,
            show_return: false,
            create_save_target: CrudCreateSaveTarget::Return,
            create_actions_placement: CrudActionsPlacement::Inline,
        }
    }

    /// Sets where create action controls are rendered.
    #[must_use]
    pub fn with_create_actions_placement(mut self, placement: CrudActionsPlacement) -> Self {
        self.create_actions_placement = placement;
        self
    }
}

impl Default for CrudBuiltinViewControls {
    fn default() -> Self {
        Self {
            show_save: true,
            show_save_and_back: true,
            show_save_and_new: true,
            show_delete: true,
            show_return: true,
            create_save_target: CrudCreateSaveTarget::EditView,
            create_actions_placement: CrudActionsPlacement::Inline,
        }
    }
}

/// Scopes a nested instance to the entity shown by another instance under the same manager.
///
/// The child restricts its requests to entities whose `referencing_field` equals the parent's
/// `referenced_field` and prefills that field in new entities.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CrudParentConfig {
    /// The name of the parent instance from which the referenced id should be loaded.
    pub name: &'static str,

    /// The field of the parent instance from which the referenced id should be loaded. For example: "id".
    pub referenced_field: Cow<'static, str>,

    /// The `own` field in which the reference is stored. For example: "`user_id`", when referencing a User entity.
    pub referencing_field: Cow<'static, str>, // TODO: This should be: T::ReadModel::Field? (ClusterCertificateField::CreatedAt)
}

/// Layout of the edit and read views: a sequence of fields, separators, and enclosing groups.
pub type UpdateElements = Vec<Elem<DynUpdateField>>;

/// Layout of the create view.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CreateElements {
    /// No layout. The built-in create view shows a "no fields" message instead of a form.
    None,
    /// A sequence of fields, separators, and enclosing groups.
    Custom(Vec<Elem<DynCreateField>>),
}
