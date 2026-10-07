//! The context of a mounted instance: its configuration, list state, and pending interactions.

#![deny(missing_docs)]

use crate::config::CrudActionAftermath;
use crate::config::{
    CreateElements, CrudBuiltinViewControls, CrudInstanceConfig, CrudMutableInstanceConfig,
    CrudParentConfig, CrudStaticInstanceConfig, FieldRendererRegistry, Header, UpdateElements,
};
use crate::hooks::delete::CrudDeleteState;
use crate::hooks::form::CrudCreateActions;
use crate::hooks::instance::use_crud_instance;
use crate::hooks::leave::CrudLeaveConfirmation;
use crate::hooks::notify::{CrudNotifier, use_crud_notifier};
use crate::hooks::texts::use_crud_texts;
use crate::instance::{CrudInstanceMgrContext, InstanceState};
use crate::instance::{
    CrudNavigation, EnclosingNavigationScope, provide_enclosing_navigation_scope,
    use_enclosing_navigation_scope,
};
use crudkit_core::Order;
use crudkit_core::condition::{Condition, ConditionClause, ConditionElement};
use crudkit_core::id::{SerializableId, SerializableIdEntry};
use crudkit_web::layout::TabId;
use crudkit_web::list::toggle_order;
use crudkit_web::list::{ItemsPerPage, PageNr};
use crudkit_web::prelude::*;
use indexmap::IndexMap;
use leptos::context::Provider;
use leptos::prelude::*;
use std::collections::HashSet;
use uuid::Uuid;

/// Shared runtime state for one mounted instance.
///
/// [`crate::components::instance::CrudInstance`], [`crate::instance::CrudInstanceProvider`], and
/// [`crate::instance::provide_crud_instance`] provide this value to descendant components.
/// Copies refer to the same arena-owned signals and stored values. A copy is never immediately an
/// independent state snapshot.
///
/// Public mutation methods preserve the instance invariants. Signal setters remain private so
/// callers cannot bypass those methods.
#[derive(Debug, Clone, Copy)]
pub struct CrudInstanceContext {
    /// Volatile identifier for this mount.
    pub id: Uuid,

    /// Stable instance name used by [`CrudInstanceMgrContext`] lookups.
    /// Provided by the user. Required to be unique across the entire application.
    pub name: &'static str,

    default_config: StoredValue<CrudMutableInstanceConfig>,
    pub(crate) static_config: StoredValue<CrudStaticInstanceConfig>,
    pub(crate) data_provider: Signal<DynCrudRestDataProvider>,
    pub(crate) headers: ReadSignal<Vec<Header>>,
    pub(crate) create_elements: ReadSignal<CreateElements>,
    pub(crate) update_elements: ReadSignal<UpdateElements>,

    /// Navigation used by this mounted instance.
    pub navigation: CrudNavigation,

    /// The page the user is currently on in the list view.
    pub current_page: ReadSignal<PageNr>,
    set_current_page: WriteSignal<PageNr>,

    /// The amount of items shown per page in the list view.
    pub items_per_page: ReadSignal<ItemsPerPage>,
    set_items_per_page: WriteSignal<ItemsPerPage>,

    /// How data should be ordered when querying data for the list view.
    pub order_by: ReadSignal<IndexMap<DynReadField, Order>>,
    set_order_by: WriteSignal<IndexMap<DynReadField, Order>>,

    /// Parent-resource configuration, when this instance is nested below another resource.
    pub parent: StoredValue<Option<CrudParentConfig>>,

    /// Current parent entity ID when parent-resource scoping can resolve one.
    pub parent_id: Signal<Option<SerializableId>>,

    /// Condition restricting this resource to the resolved parent entity.
    pub parent_id_referencing_condition: Signal<Option<Condition>>,

    /// The base condition applicable when fetching data.
    pub base_condition: Signal<Option<Condition>>,

    /// Deletion requests awaiting confirmation.
    pub(crate) deletion: CrudDeleteState,

    /// Navigation attempts awaiting confirmation to discard unsaved input.
    pub(crate) leave_confirmation: CrudLeaveConfirmation,

    /// Token changed by [`Self::reload`] to refresh server-provided data.
    pub reload: ReadSignal<Uuid>,
    set_reload: WriteSignal<Uuid>,

    create_actions: ReadSignal<Option<CrudCreateActions>>,
    set_create_actions: WriteSignal<Option<CrudCreateActions>>,

    /// The selected tab of every tab group the user selected a tab in. See [`Self::selected_tab`].
    selected_tabs: RwSignal<HashSet<TabId>>,

    /// Destination of notifications emitted by this instance.
    pub notifier: CrudNotifier,

    /// Controls of the built-in views, as given to the current view. See
    /// [`Self::builtin_view_controls`].
    controls: Signal<CrudBuiltinViewControls>,
}

impl CrudInstanceContext {
    /// Returns the configured list columns.
    #[must_use]
    pub fn list_columns(&self) -> Signal<Vec<Header>> {
        self.headers.into()
    }

    /// Returns the configured create layout.
    #[must_use]
    pub fn create_elements(&self) -> Signal<CreateElements> {
        self.create_elements.into()
    }

    /// Returns the configured read and edit layout.
    #[must_use]
    pub fn update_elements(&self) -> Signal<UpdateElements> {
        self.update_elements.into()
    }

    /// Returns the configured renderers of read-model fields, used in lists.
    #[must_use]
    pub fn read_field_renderers(&self) -> FieldRendererRegistry<DynReadField> {
        self.static_config.read_value().read_field_renderer.clone()
    }

    /// Returns the configured renderers of create-model fields.
    #[must_use]
    pub fn create_field_renderers(&self) -> FieldRendererRegistry<DynCreateField> {
        self.static_config
            .read_value()
            .create_field_renderer
            .clone()
    }

    /// Returns the configured renderers of update-model fields, used in read and edit views.
    #[must_use]
    pub fn update_field_renderers(&self) -> FieldRendererRegistry<DynUpdateField> {
        self.static_config
            .read_value()
            .update_field_renderer
            .clone()
    }

    /// Returns the controls of the built-in views: the instance's configuration, unless the
    /// current view was given its own. Tracks changes.
    #[must_use]
    pub fn builtin_view_controls(&self) -> CrudBuiltinViewControls {
        self.controls.get()
    }

    /// Returns this context as seen by a view given its own `navigation` or `controls`.
    pub(crate) fn for_view(
        self,
        navigation: Option<CrudNavigation>,
        controls: Option<Signal<CrudBuiltinViewControls>>,
    ) -> Self {
        Self {
            navigation: navigation.unwrap_or(self.navigation),
            controls: controls.unwrap_or(self.controls),
            ..self
        }
    }

    /// Returns the name of the resource on the wire.
    #[must_use]
    pub fn resource_name(&self) -> String {
        self.static_config.read_value().resource_name.clone()
    }

    /// Selects the list page to load.
    pub(crate) fn set_page(&self, page_number: PageNr) {
        self.set_current_page.set(page_number);
    }

    /// Sets the number of entities shown on each list page.
    pub(crate) fn set_items_per_page(&self, items_per_page: ItemsPerPage) {
        self.set_items_per_page.set(items_per_page);
    }

    // TODO: Why is this here and CrudInstanceConfig#update_order_by exists?
    /// Applies an ordering interaction for `field`.
    ///
    /// The interaction toggles the field between ascending and descending. It
    /// clears existing ordering first unless `append` is set.
    // False positive: `field` is moved into the ordering through the update closure.
    #[allow(clippy::needless_pass_by_value)]
    pub(crate) fn toggle_order_by(&self, field: DynReadField, append: bool) {
        tracing::debug!(?field, append, "order by");
        self.set_order_by
            .update(|order_by| toggle_order(order_by, field, append));
    }

    /// Replaces the list ordering.
    pub(crate) fn set_order_by(&self, order_by: IndexMap<DynReadField, Order>) {
        self.set_order_by.set(order_by);
    }

    /// Returns the selected tab of the tab group consisting of `tabs`, unless none was selected yet.
    ///
    /// The instance remembers the selection of each tab group, so it survives re-rendering a layout
    /// and switching between views. Layouts whose tab groups use the same tab IDs share their
    /// selection.
    #[must_use]
    pub fn selected_tab(&self, tabs: &[TabId]) -> Option<TabId> {
        self.selected_tabs
            .with(|selected| tabs.iter().find(|tab| selected.contains(*tab)).cloned())
    }

    /// Selects `tab` in the tab group consisting of `tabs`. Ignores tabs outside of the group.
    pub fn select_tab(&self, tabs: &[TabId], tab: TabId) {
        let unchanged = self
            .selected_tabs
            .with_untracked(|selected| selected.contains(&tab));
        if unchanged || !tabs.contains(&tab) {
            return;
        }
        self.selected_tabs.update(|selected| {
            for other in tabs {
                selected.remove(other);
            }
            selected.insert(tab);
        });
    }

    // TODO: Other functions do not take a . Should the instance provide its  to store it in this context? Would allow everyone to have access.
    /// Applies the shared UI effects from an action result.
    ///
    /// Both success and failure aftermaths may publish a toast or reload the
    /// instance. The `Result` variant records the action outcome; the contained
    /// [`CrudActionAftermath`] defines the UI effects in either case.
    pub fn handle_action_outcome(&self, outcome: Result<CrudActionAftermath, CrudActionAftermath>) {
        tracing::info!(?outcome, "handling action outcome");

        let CrudActionAftermath {
            notification,
            reload_data,
        } = match outcome {
            Ok(outcome) | Err(outcome) => outcome,
        };

        if let Some(notification) = notification {
            self.notifier.notify(notification);
        }

        if reload_data {
            self.reload();
        }
    }

    /// Changes the reload token so data-dependent views fetch current server state.
    pub fn reload(&self) {
        self.set_reload.set(Uuid::new_v4());
    }

    /// Returns the save controls of the mounted create form, if any. Tracks changes.
    #[must_use]
    pub fn create_actions(&self) -> Option<CrudCreateActions> {
        self.create_actions.get()
    }

    pub(crate) fn set_create_actions(&self, actions: Option<CrudCreateActions>) {
        self.set_create_actions.set(actions);
    }

    /// Reset this instance to its default configuration.
    /// Every change made by the user is reverted.
    pub fn reset(&self) {
        let context = *self;
        self.navigation
            .attempt(move || context.reset_committed(), || {});
    }

    fn reset_committed(&self) {
        let default = self.default_config.get_value();
        self.deletion.clear();
        self.set_current_page.set(default.page);
        self.set_items_per_page.set(default.items_per_page);
        self.set_order_by.set(default.order_by.clone());
        self.set_create_actions.set(None);
        self.navigation
            .navigate_committed(default.initial_view.clone());
    }
}

// TODO: Effect::new over all signals in config, bundle, serialize and store...

/// Input of [`provide_crud_instance`].
#[derive(Debug)]
pub struct ProvideCrudInstanceInput {
    /// Stable name used to register the instance with its manager.
    pub name: &'static str,
    /// Resource, view, renderer, control, and request configuration.
    pub config: CrudInstanceConfig,
    /// Parent-resource relationship used to scope child data.
    pub parent: Option<CrudParentConfig>,
    /// Caller-owned navigation that replaces manager-created navigation.
    pub navigation: Option<CrudNavigation>,
}

impl ProvideCrudInstanceInput {
    /// Creates an input mounting `config` as `name`.
    #[must_use]
    pub fn new(name: &'static str, config: CrudInstanceConfig) -> Self {
        Self {
            name,
            config,
            parent: None,
            navigation: None,
        }
    }
}

/// Mounts an instance like [`crate::instance::CrudInstanceProvider`] and provides its context in
/// the current reactive owner, e.g. in an application's own provider component or in tests.
///
/// The context reaches everything rendered by the calling component. Call it in a component that
/// renders nothing but the instance's content, so siblings do not see the instance.
///
/// # Panics
///
/// Panics when called outside of a [`crate::instance::CrudInstanceMgr`].
// Providing the context is the point; using the returned copy is optional.
#[allow(clippy::must_use_candidate)]
pub fn provide_crud_instance(input: ProvideCrudInstanceInput) -> CrudInstanceContext {
    let ProvideCrudInstanceInput {
        name,
        config,
        parent,
        navigation,
    } = input;
    let ctx = create_instance_context(name, config, parent, navigation, None);
    provide_context(ctx);
    provide_enclosing_navigation_scope(ctx.navigation.scope());
    ctx
}

pub(crate) fn create_instance_context(
    name: &'static str,
    config: CrudInstanceConfig,
    parent: Option<CrudParentConfig>,
    navigation: Option<CrudNavigation>,
    on_context_created: Option<Callback<CrudInstanceContext>>,
) -> CrudInstanceContext {
    // Unique id of this instance. Volatile. Not persistent between rerenders.
    let id = Uuid::new_v4();

    let (config, static_config) = config.split();

    let static_config = StoredValue::new(static_config);

    let (api_base_url, _set_api_base_url) = signal(config.api_base_url.clone());
    let mgr = expect_context::<CrudInstanceMgrContext>();
    let instance_navigation = navigation.unwrap_or_else(|| {
        use_enclosing_navigation_scope()
            .unwrap_or_else(|| mgr.navigation_scope())
            .child_navigation(config.initial_view.clone())
    });
    let view = instance_navigation.current();
    let navigation = instance_navigation.scoped();
    let leave_confirmation = CrudLeaveConfirmation::host(navigation);

    mgr.register(name, InstanceState { name, view });

    let (headers, _set_headers) = signal(config.headers.clone());
    let (current_page, set_current_page) = signal(config.page);
    let (items_per_page, set_items_per_page) = signal(config.items_per_page);
    let (order_by, set_order_by) = signal(config.order_by.clone());

    let parent = StoredValue::new(parent);
    let parent_id = Signal::derive(move || {
        parent
            .read_value()
            .as_ref()
            .and_then(|parent| get_parent_id(parent, mgr))
    });
    let parent_id_referencing_condition = Signal::derive(move || {
        parent
            .read_value()
            .as_ref()
            .and_then(|parent| parent_id_referencing_condition(parent, mgr))
    });
    let base_condition = config.base_condition.clone();
    let base_condition = Signal::derive(move || {
        crudkit_core::condition::merge_conditions(
            parent_id_referencing_condition.get(),
            base_condition.clone(),
        )
    });
    let (create_elements, _set_create_elements) = signal(config.create_elements.clone());
    let (update_elements, _set_update_elements) = signal(config.elements.clone());
    let (reload, set_reload) = signal(Uuid::new_v4());
    let (create_actions, set_create_actions) = signal(None::<CrudCreateActions>);
    let selected_tabs = RwSignal::new(HashSet::new());

    let controls = Signal::stored(static_config.read_value().builtin_view_controls);
    let default_config = StoredValue::new(config);
    let notifier = use_crud_notifier();

    let data_provider = Signal::derive(move || {
        DynCrudRestDataProvider::new(
            api_base_url.get(),
            static_config.read_value().reqwest_executor.clone(),
            static_config.read_value().resource_name.clone(),
            static_config.read_value().model_handler,
        )
    });
    let deletion = CrudDeleteState::new(
        data_provider,
        Callback::new(move |()| set_reload.set(Uuid::new_v4())),
        notifier,
        use_crud_texts(),
        navigation,
        base_condition,
    );

    let ctx = CrudInstanceContext {
        id,
        name,
        default_config,
        static_config,
        data_provider,
        headers,
        create_elements,
        update_elements,
        navigation,
        current_page,
        set_current_page,
        items_per_page,
        set_items_per_page,
        order_by,
        set_order_by,
        parent,
        parent_id,
        parent_id_referencing_condition,
        base_condition,
        deletion,
        leave_confirmation,
        reload,
        set_reload,
        create_actions,
        set_create_actions,
        selected_tabs,
        notifier,
        controls,
    };
    if let Some(on_context_created) = on_context_created {
        on_context_created.run(ctx);
    }
    ctx
}

/// Provides `context` to a view rendered with `navigation`, in a navigation scope of its own.
/// Returns the view's navigation.
pub(crate) fn provide_view_context(
    context: &CrudInstanceContext,
    navigation: CrudNavigation,
) -> CrudNavigation {
    let navigation = navigation.scoped();
    provide_context(CrudInstanceContext {
        navigation,
        ..*context
    });
    provide_enclosing_navigation_scope(navigation.scope());
    navigation
}

/// Renders `content` with the surrounding instance as seen by a view given its own `navigation`
/// or `controls`, so the hooks, components, and nested instances in `content` use them too.
pub(crate) fn with_view_overrides<V: IntoView + 'static>(
    navigation: Option<CrudNavigation>,
    controls: Option<Signal<CrudBuiltinViewControls>>,
    content: impl FnOnce() -> V + Send + 'static,
) -> impl IntoView {
    let ctx = use_crud_instance().for_view(navigation, controls);
    view! {
        <Provider value=ctx>
            <Provider value=EnclosingNavigationScope(ctx.navigation.scope())>{content()}</Provider>
        </Provider>
    }
}

/// Condition restricting entities to those referencing the current parent entity.
fn parent_id_referencing_condition(
    parent: &CrudParentConfig,
    mgr: CrudInstanceMgrContext,
) -> Option<Condition> {
    let id = get_parent_id(parent, mgr)?;
    let entry = id
        .into_entries()
        .find(|entry| entry.field_name == parent.referenced_field);

    let Some(SerializableIdEntry {
        field_name: _,
        value,
    }) = entry
    else {
        tracing::warn!(
            referenced_field = %parent.referenced_field,
            "Referenced field not found in parent ID"
        );
        return None;
    };

    let Ok(clause_value) = value.clone().try_into() else {
        tracing::warn!("Parent ID value not convertible to condition clause value");
        return None;
    };

    Some(Condition::All(vec![ConditionElement::Clause(
        ConditionClause {
            column_name: parent.referencing_field.to_string(),
            operator: crudkit_core::condition::Operator::Equal,
            value: clause_value,
        },
    )]))
}

fn get_parent_id(parent: &CrudParentConfig, mgr: CrudInstanceMgrContext) -> Option<SerializableId> {
    // Uses untracked access via StoredValue::read_value() internally.
    // Otherwise, at instance nesting depth 3, rendering the instance and registering it would
    // cause instance at depth 2 to register this change here and force a field-rerender.
    let parent_state = mgr.get_by_name(parent.name)?;
    parent_state.view.get_untracked().subject
}
