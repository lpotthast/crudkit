use crudkit_shared::Order;
use crudkit_web::prelude::*;
use indexmap::IndexMap;
use leptonic::prelude::*;
use leptos::*;
use serde::{Deserialize, Serialize};

use crate::{crud_instance_config::{CrudStaticInstanceConfig, CrudInstanceConfig, CreateElements, NestedConfig}, prelude::CrudListViewL};

#[component]
pub fn CrudInstanceL<T>(
    cx: Scope,

    // TODO: Analyze children once on creation and on prop changes. Pass generated data-structure to children!
    // TODO: Only allow easy-to-parse structure:
    /*
       tbd...
       ListDetails {

       }
       FieldDetails {

       }
    */
    //#[prop_or_default]
    //pub children: ChildrenRenderer<Item>,
    #[prop(into)] name: String,

    #[prop(into)] api_base_url: Signal<String>,
    #[prop(into)] view: Signal<CrudView<T::ReadModelId, T::UpdateModelId>>,
    #[prop(into)] headers: Signal<Vec<(<T::ReadModel as CrudDataTrait>::Field, HeaderOptions)>>,
    #[prop(into)] create_elements: Signal<CreateElements<T>>,
    #[prop(into)] elements: Signal<Vec<Elem<T::UpdateModel>>>,
    #[prop(into)] order_by: Signal<IndexMap<<T::ReadModel as CrudDataTrait>::Field, Order>>,
    #[prop(into)] items_per_page: Signal<u64>,
    #[prop(into)] page: Signal<u64>,
    #[prop(into)] active_tab: Signal<Option<Label>>,
    #[prop(into)] nested: Signal<Option<NestedConfig>>,

    static_config: CrudStaticInstanceConfig<T>,
    //pub portal_target: Option<String>,
) -> impl IntoView
where
    T: CrudMainTrait + 'static,
{
    let data_provider = Signal::derive(cx, move || {
        CrudRestDataProvider::<T>::new(api_base_url.get())
    });

    let on_reset = move || {};

    let body =  move || match view.get() {
        CrudView::List => {
            view! {cx,
                <CrudListViewL
                    data_provider=data_provider
                    headers=headers
                    order_by=order_by
                    //children={ctx.props().children.clone()}
                    //custom_fields={self.static_config.custom_read_fields.clone()}
                    //static_config={self.static_config.clone()}
                    //on_reset={ctx.link().callback(|_| Msg::Reset)}
                    //on_create={ctx.link().callback(|_| Msg::Create)}
                    //on_read={ctx.link().callback(Msg::Read)}
                    //on_edit={ctx.link().callback(|read_model: T::ReadModel| Msg::Edit(read_model.into()))}
                    //on_delete={ctx.link().callback(|entity| Msg::Delete(DeletableModel::Read(entity)))}
                    //on_order_by={ctx.link().callback(Msg::OrderBy)}
                    //on_page_selected={ctx.link().callback(Msg::PageSelected)}
                    //on_item_count_selected={ctx.link().callback(Msg::ItemCountSelected)}
                    //on_entity_action={ctx.link().callback(Msg::EntityAction)}
                    //on_global_action={ctx.link().callback(Msg::GlobalAction)}
                    //on_link={ctx.link().callback(|link: Option<Scope<CrudListView<T>>>|
                    //    Msg::ViewLinked(link.map(|link| ViewLink::List(link))))}
                />
            }.into_view(cx)
        },
        CrudView::Create => {
            view! {cx,
                "create"
                //<CrudCreateView<T>
                //    data_provider={self.data_provider.clone()}
                //    parent_id={self.parent_id.clone()}
                //    children={ctx.props().children.clone()}
                //    custom_create_fields={self.static_config.custom_create_fields.clone()}
                //    custom_update_fields={self.static_config.custom_update_fields.clone()}
                //    config={self.config.clone()}
                //    list_view_available={true}
                //    on_list_view={ctx.link().callback(|_| Msg::List)}
                //    on_entity_created={ctx.link().callback(Msg::EntityCreated)}
                //    on_entity_creation_aborted={ctx.link().callback(Msg::EntityCreationAborted)}
                //    on_entity_not_created_critical_errors={ctx.link().callback(|_| Msg::EntityNotCreatedDueToCriticalErrors)}
                //    on_entity_creation_failed={ctx.link().callback(Msg::EntityCreationFailed)}
                //    on_link={ctx.link().callback(|link: Option<Scope<CrudCreateView<T>>>|
                //        Msg::ViewLinked(link.map(|link| ViewLink::Create(link))))}
                //    on_tab_selected={ctx.link().callback(|label| Msg::TabSelected(label))}
                // />
            }.into_view(cx)
        },
        CrudView::Read(id) => {
            view! {cx,
                "read"
                //<CrudReadView<T>
                //    data_provider={self.data_provider.clone()}
                //    children={ctx.props().children.clone()}
                //    custom_fields={self.static_config.custom_update_fields.clone()}
                //    config={self.config.clone()}
                //    id={id.clone()}
                //    list_view_available={true}
                //    on_list_view={ctx.link().callback(|_| Msg::List)}
                //    on_tab_selected={ctx.link().callback(|label| Msg::TabSelected(label))}
                // />
            }.into_view(cx)
        },
        CrudView::Edit(id) => {
            view! {cx,
                "edit"
                //<CrudEditView<T>
                //    data_provider={self.data_provider.clone()}
                //    children={ctx.props().children.clone()}
                //    custom_fields={self.static_config.custom_update_fields.clone()}
                //    config={self.config.clone()}
                //    static_config={self.static_config.clone()}
                //    id={id.clone()}
                //    list_view_available={true}
                //    on_entity_updated={ctx.link().callback(Msg::EntityUpdated)}
                //    on_entity_update_aborted={ctx.link().callback(Msg::EntityUpdateAborted)}
                //    on_entity_not_updated_critical_errors={ctx.link().callback(|_| Msg::EntityNotUpdatedDueToCriticalErrors)}
                //    on_entity_update_failed={ctx.link().callback(Msg::EntityUpdateFailed)}
                //    on_list={ctx.link().callback(|_| Msg::List)}
                //    on_create={ctx.link().callback(|_| Msg::Create)}
                //    on_delete={ctx.link().callback(|entity| Msg::Delete(DeletableModel::Update(entity)))}
                //    on_link={ctx.link().callback(|link: Option<Scope<CrudEditView<T>>>|
                //        Msg::ViewLinked(link.map(|link| ViewLink::Edit(link))))}
                //    on_tab_selected={ctx.link().callback(|label| Msg::TabSelected(label))}
                //    on_entity_action={ctx.link().callback(Msg::CustomEntityAction)}
                // />
            }.into_view(cx)
        },
    };

    view! {cx,
        "CrudInstance: " {name}

        <div class="crud-instance">
            <div class="body">
                { body }
            </div>
        </div>
    }
}
