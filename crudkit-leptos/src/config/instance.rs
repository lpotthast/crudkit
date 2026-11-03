use crate::config::CrudViewRegistry;
use crate::config::FieldRendererRegistry;
use crate::config::{CrudAction, CrudEntityAction};
use crudkit_core::Order;
use crudkit_web::data_source::{CrudDataSource, CrudRestDataSource, CrudScope};
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

/// Configuration of one mounted [`CrudInstance`](crate::instance::CrudInstance).
///
/// The first group of fields seeds the instance's reactive state when it mounts and is restored
/// by [`CrudInstanceContext::reset`](crate::instance::CrudInstanceContext::reset) where
/// applicable. The second group is treated as static for the lifetime of the mount. Start from
/// [`CrudInstanceConfig::new`] or [`CrudInstanceConfig::with_data_source`] and complete it with
/// struct update syntax.
#[derive(Debug, Clone)]
pub struct CrudInstanceConfig {
    /* Later to-be mutable data. */
    /// View used to initialize instance-owned navigation and as the reset destination.
    pub initial_view: CrudView,
    /// Columns of the list view, in display order.
    pub list_columns: Vec<Header>,
    /// Layout of the create view.
    pub create_elements: CreateElements,
    /// Layout of the edit and read views, both of which render the update model.
    pub elements: UpdateElements,
    /// Initial ordering of the list, restored on reset. Earlier entries take precedence; an empty
    /// map leaves the order to the data source. Fields the data source cannot order by are left
    /// out.
    pub order_by: IndexMap<DynReadField, Order>,
    /// The number of items shown per page in the list view.
    pub items_per_page: ItemsPerPage,
    /// The current page to display, e.g. `PageNr::first()`. One-based index. A list whose data
    /// source pages by cursor starts at the first page.
    pub page_nr: PageNr,
    /// The parents of the instance's entities, e.g. from the application's route, and the base
    /// condition, which restricts every operation of the instance except creation. The instance
    /// passes it to every operation of its data source. A [`CrudParentConfig`] adds the parent's
    /// ID to its parents.
    pub scope: CrudScope,

    /* Immutable data */
    /// Where the instance reads and changes its entities.
    pub data_source: Arc<dyn CrudDataSource>,
    /// Type-erased access to the resource's concrete create, read, and update models.
    pub model_handler: ModelHandler,
    /// Actions operating on the resource as a whole, offered by
    /// [`CrudResourceActions`](crate::atoms::CrudResourceActions).
    pub actions: Vec<CrudAction>,
    /// Actions operating on the current entity, offered by
    /// [`CrudEntityActions`](crate::atoms::CrudEntityActions) in the forms their valid states
    /// allow.
    pub entity_actions: Vec<CrudEntityAction>,
    /// The renderers of this instance's views. Defaults to the registry provided with
    /// [`crate::hooks::views::provide_crud_view_registry`], else an empty registry.
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
    /// Creates a configuration of resource `R`, served by CrudKit's REST routes below
    /// `api_base_url` (`{api_base_url}/{resource_name}/crud/v1/...`) and requested through
    /// `reqwest_executor`. Its data source is a [`CrudRestDataSource`].
    ///
    /// Everything else starts as [`Self::with_data_source`] describes.
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
        Self::with_data_source::<R>(Arc::new(CrudRestDataSource::new(
            api_base_url.into(),
            reqwest_executor,
            R::resource_name().to_owned(),
            model_handler::<R>(),
        )))
    }

    /// Creates a configuration of resource `R`, whose entities `data_source` reads and changes.
    ///
    /// Everything else starts empty or at its default: no list columns, layouts, actions, or
    /// registered field renderers, an empty scope, the table as initial view, and the view
    /// registry provided by the application. Complete it with struct update syntax.
    #[must_use]
    pub fn with_data_source<R>(data_source: Arc<dyn CrudDataSource>) -> Self
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
            initial_view: CrudView::default(),
            list_columns: Vec::new(),
            create_elements: Vec::new(),
            elements: Vec::new(),
            order_by: IndexMap::new(),
            items_per_page: ItemsPerPage::default(),
            page_nr: PageNr::default(),
            scope: CrudScope::default(),
            data_source,
            model_handler: model_handler::<R>(),
            actions: Vec::new(),
            entity_actions: Vec::new(),
            view_registry: None,
            read_field_renderer: FieldRendererRegistry::default(),
            create_field_renderer: FieldRendererRegistry::default(),
            update_field_renderer: FieldRendererRegistry::default(),
        }
    }

    pub(crate) fn split(self) -> (CrudMutableInstanceConfig, CrudStaticInstanceConfig) {
        (
            CrudMutableInstanceConfig {
                initial_view: self.initial_view,
                headers: self.list_columns,
                create_elements: self.create_elements,
                elements: self.elements,
                order_by: self.order_by,
                items_per_page: self.items_per_page,
                page: self.page_nr,
                scope: self.scope,
            },
            CrudStaticInstanceConfig {
                data_source: self.data_source,
                model_handler: self.model_handler,
                actions: self.actions,
                entity_actions: self.entity_actions,
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
    pub initial_view: CrudView,
    pub headers: Vec<Header>,
    pub create_elements: CreateElements,
    pub elements: UpdateElements,
    pub order_by: IndexMap<DynReadField, Order>,
    pub items_per_page: ItemsPerPage,
    pub page: PageNr,
    pub scope: CrudScope,
}

/// This config is non-serializable. Every piece of runtime-changing data relevant to be tracked and reloaded should be part of the `CrudInstanceConfig` struct.
#[derive(Debug, Clone)]
pub(crate) struct CrudStaticInstanceConfig {
    pub data_source: Arc<dyn CrudDataSource>,
    pub model_handler: ModelHandler,
    pub actions: Vec<CrudAction>,
    pub entity_actions: Vec<CrudEntityAction>,
    pub view_registry: CrudViewRegistry,
    pub read_field_renderer: FieldRendererRegistry<DynReadField>,
    pub create_field_renderer: FieldRendererRegistry<DynCreateField>,
    pub update_field_renderer: FieldRendererRegistry<DynUpdateField>,
}

/// Scopes a nested instance to the entity shown by another instance under the same manager.
///
/// The child adds the parent's ID to the parents of its [`CrudScope`]: the value of the parent's
/// `referenced_field`, referred to through the child's `referencing_field`. Its data source keeps
/// every operation below that parent, and new entities get the value in their `referencing_field`
/// if the create model has that field.
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

/// Layout of the create view: a sequence of fields, separators, and enclosing groups.
pub type CreateElements = Vec<Elem<DynCreateField>>;

/// Returns the model handler of the concrete models of resource `R`.
fn model_handler<R>() -> ModelHandler
where
    R: crudkit_web::resource::Resource,
    R::CreateModel: ErasedCreateModel,
    R::ReadModel: ErasedReadModel,
    R::UpdateModel: ErasedUpdateModel + From<R::ReadModel>,
    <R::ReadModel as Model>::Field: ErasedReadField,
    <R::CreateModel as Model>::Field: ErasedCreateField,
    <R::UpdateModel as Model>::Field: ErasedUpdateField,
{
    ModelHandler::new::<R::CreateModel, R::ReadModel, R::UpdateModel>()
}
