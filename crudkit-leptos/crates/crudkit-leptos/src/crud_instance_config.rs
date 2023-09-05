use std::{collections::HashMap, fmt::Debug, hash::Hash, rc::Rc};

use crudkit_condition::Condition;
use crudkit_shared::Order;
use crudkit_web::prelude::*;
use dyn_clone::DynClone;
use indexmap::{indexmap, IndexMap};
use leptonic::prelude::*;
use leptos::{View, *};
use serde::{Deserialize, Serialize};

use crate::crud_action::{CrudAction, CrudEntityAction};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CrudInstanceConfig<T: CrudMainTrait> {
    pub api_base_url: String,
    pub view: CrudView<T::ReadModelId, T::UpdateModelId>,
    pub headers: Vec<(<T::ReadModel as CrudDataTrait>::Field, HeaderOptions)>,
    // serde bound used as described in: https://github.com/serde-rs/serde/issues/1296
    #[serde(bound = "")]
    pub create_elements: CreateElements<T>,
    // serde bound used as described in: https://github.com/serde-rs/serde/issues/1296
    #[serde(bound = "")]
    pub elements: Vec<Elem<T::UpdateModel>>,
    pub order_by: IndexMap<<T::ReadModel as CrudDataTrait>::Field, Order>,
    pub items_per_page: u64,
    pub page: u64,
    pub active_tab: Option<Label>,
    pub base_condition: Option<Condition>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CrudParentConfig {
    /// The name of the parent instance from which the referenced id should be loaded.
    pub name: &'static str,

    /// The field of the parent instance from which the referenced id should be loaded. For example: "id".
    pub referenced_field: String,

    /// The `own` field in which the reference is stored. For example: "user_id", when referencing a User entity.
    pub referencing_field: String, // TODO: This should be: T::ReadModel::Field? (ClusterCertificateField::CreatedAt)
}

pub trait SelectConfigTrait: Debug + DynClone {
    fn render_select(
        &self,

        selected: leptos::Signal<Box<dyn CrudSelectableTrait>>,
        set_selected: SimpleCallback<Box<dyn CrudSelectableTrait>>,
    ) -> View;
    fn render_optional_select(
        &self,

        selected: leptos::Signal<Option<Box<dyn CrudSelectableTrait>>>,
        set_selected: SimpleCallback<Option<Box<dyn CrudSelectableTrait>>>,
    ) -> View;
}
dyn_clone::clone_trait_object!(SelectConfigTrait);

pub type DynSelectConfig = Box<dyn SelectConfigTrait>;

#[derive(Debug, Clone)]
pub enum SelectOptionsProvider<
    O: Debug + Clone + PartialEq + Eq + Hash + CrudSelectableTrait + 'static,
> {
    Static {
        options: Vec<O>,
    },
    Dynamic {
        provider: Rc<dyn CrudSelectableSource<Selectable = O>>,
    },
}

impl<O: Debug + Clone + PartialEq + Eq + Hash + CrudSelectableTrait + 'static>
    SelectOptionsProvider<O>
{
    pub fn provide(
        &self,
    ) -> MaybeSignal<Option<Result<Vec<O>, Rc<dyn std::error::Error + Send + Sync + 'static>>>>
    {
        match self {
            SelectOptionsProvider::Static { options } => Some(Ok(options.clone())).into(),
            SelectOptionsProvider::Dynamic { provider } => {
                let provider = provider.clone();
                let load_action = create_action(move |()| {
                    let provider = provider.clone();
                    async move { provider.load().await }
                });
                load_action.dispatch(());
                let load_action_value = load_action.value();
                load_action_value.read_only().into()
            }
        }
    }
}

#[derive(Clone)]
pub struct SelectConfig<O: Debug + Clone + PartialEq + Eq + Hash + CrudSelectableTrait + 'static> {
    pub options_provider: SelectOptionsProvider<O>,
    pub renderer: Callback<O, View>,
}

impl<O: Debug + Clone + PartialEq + Eq + Hash + CrudSelectableTrait + 'static> Debug
    for SelectConfig<O>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SelectConfig")
            .field("options", &self.options_provider)
            .field("renderer", &self.renderer)
            .finish()
    }
}

impl<O: Debug + Clone + PartialEq + Eq + Hash + CrudSelectableTrait + 'static> SelectConfigTrait
    for SelectConfig<O>
{
    fn render_select(
        &self,

        selected: Signal<Box<dyn CrudSelectableTrait>>,
        set_selected: SimpleCallback<Box<dyn CrudSelectableTrait>>,
    ) -> View {
        let options = self.options_provider.provide();
        let selected =
            Signal::derive(move || selected.get().as_any().downcast_ref::<O>().unwrap().clone());
        let set_selected = create_callback(move |o: O| set_selected.call(Box::new(o)));
        let renderer = self.renderer;
        view! {
            {move || {
                let option = options.get();
                match option {
                    Some(result) => {
                        match result {
                            Ok(options) => {
                                view! {
                                    <Select
                                        options=options
                                        selected=selected
                                        set_selected=set_selected
                                        search_text_provider=create_callback(move |o: O| { o.to_string() })

                                        render_option=renderer
                                    />
                                }
                                    .into_view()
                            }
                            Err(err) => format!("Could not load options... Err: {err:?}").into_view(),
                        }
                    }
                    None => "Loading...".into_view(),
                }
            }}
        }
        .into_view()
    }

    fn render_optional_select(
        &self,

        selected: Signal<Option<Box<dyn CrudSelectableTrait>>>,
        set_selected: SimpleCallback<Option<Box<dyn CrudSelectableTrait>>>,
    ) -> View {
        let options = self.options_provider.provide();
        let selected = Signal::derive(move || {
            selected
                .get()
                .map(|it| it.as_any().downcast_ref::<O>().unwrap().clone())
        });
        let set_selected = create_callback(move |o: Option<O>| match o {
            Some(o) => set_selected.call(Some(Box::new(o))),
            None => set_selected.call(None),
        });
        let renderer = self.renderer;
        view! {
            {move || {
                let option = options.get();
                match option {
                    Some(result) => {
                        match result {
                            Ok(options) => {
                                view! {
                                    <OptionalSelect
                                        options=options
                                        selected=selected
                                        set_selected=set_selected
                                        search_text_provider=create_callback(move |o: O| { o.to_string() })

                                        render_option=renderer
                                        allow_deselect=true
                                    />
                                }
                                    .into_view()
                            }
                            Err(err) => format!("Could not load options... Err: {err:?}").into_view(),
                        }
                    }
                    None => "Loading...".into_view(),
                }
            }}
        }
        .into_view()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CreateElements<T: CrudMainTrait> {
    None,
    Custom(Vec<Elem<T::CreateModel>>),
}

/// This config is non-serializable. Every piece of runtime-changing data relevant to be tracked and reloaded should be part of the CrudInstanceConfig struct.
#[derive(Debug, Clone)]
pub struct CrudStaticInstanceConfig<T: CrudMainTrait + 'static> {
    pub actions: Vec<CrudAction<T>>,
    pub entity_actions: Vec<CrudEntityAction<T>>,
    pub create_field_select_config:
        HashMap<<T::CreateModel as CrudDataTrait>::Field, DynSelectConfig>,
    pub read_field_select_config: HashMap<<T::ReadModel as CrudDataTrait>::Field, DynSelectConfig>,
    pub update_field_select_config:
        HashMap<<T::UpdateModel as CrudDataTrait>::Field, DynSelectConfig>,
    pub custom_read_fields: CustomReadFields<T, View>,
    pub custom_create_fields: CustomCreateFields<T, View>,
    pub custom_update_fields: CustomUpdateFields<T, View>,
}

impl<T: CrudMainTrait> Default for CrudStaticInstanceConfig<T> {
    fn default() -> Self {
        Self {
            actions: Default::default(),
            entity_actions: Default::default(),
            create_field_select_config: Default::default(),
            read_field_select_config: Default::default(),
            update_field_select_config: Default::default(),
            custom_read_fields: Default::default(),
            custom_create_fields: Default::default(),
            custom_update_fields: Default::default(),
        }
    }
}

impl<T: CrudMainTrait> Default for CrudInstanceConfig<T> {
    fn default() -> Self {
        Self {
            api_base_url: "".to_owned(),
            view: CrudView::default(),
            // headers: vec![( // TODO: build from id fields_iter
            //     T::ReadModel::get_id_field(),
            //     HeaderOptions {
            //         display_name: "ID".to_owned(),
            //         min_width: true,
            //         ordering_allowed: true,
            //         date_time_display: DateTimeDisplay::LocalizedLocal,
            //     },
            // )],
            headers: vec![],
            create_elements: CreateElements::None,
            elements: vec![],
            // order_by: indexmap! { // TODO: Nothing? First id field? All id fields?
            //     T::ReadModel::get_id_field() => Order::Asc,
            // },
            order_by: indexmap! {},
            items_per_page: 10,
            page: 1,
            active_tab: None,
            base_condition: None,
        }
    }
}

impl<T: CrudMainTrait> CrudInstanceConfig<T> {
    pub fn update_order_by(
        &mut self,
        field: <T::ReadModel as CrudDataTrait>::Field,
        options: OrderByUpdateOptions,
    ) {
        let prev = self.order_by.get(&field).cloned();
        if !options.append {
            self.order_by.clear();
        }
        self.order_by.insert(
            field,
            match prev {
                Some(order) => match order {
                    Order::Asc => Order::Desc,
                    Order::Desc => Order::Asc,
                },
                None => Order::Asc,
            },
        );
    }
}
