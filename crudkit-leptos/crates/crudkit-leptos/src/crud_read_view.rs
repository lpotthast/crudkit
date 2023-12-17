use std::{collections::HashMap, marker::PhantomData};

use crudkit_condition::{merge_conditions, Condition, IntoAllEqualCondition};
use crudkit_id::{Id, IdField};
use crudkit_web::{
    prelude::{CrudRestDataProvider, ReadOne},
    CrudDataTrait, CrudFieldValueTrait, CrudMainTrait, CrudSimpleView, Elem, FieldMode, TabId,
};
use leptonic::prelude::*;
use leptos::*;
use uuid::Uuid;

use crate::{
    crud_action_buttons::CrudActionButtons,
    crud_fields::CrudFields,
    crud_instance::CrudInstanceContext,
    crud_instance_config::DynSelectConfig,
    crud_table::NoDataAvailable,
    prelude::{CrudActionContext, CrudEntityAction, States, CustomUpdateFields},
    IntoReactiveValue, ReactiveValue,
};

#[derive(Debug, Clone, PartialEq)]
struct EntityReq<T: CrudMainTrait + 'static> {
    condition: Option<Condition>,
    reload: Uuid,
    data_provider: CrudRestDataProvider<T>,
}

#[component]
pub fn CrudReadView<T>(
    _phantom: PhantomData<T>,
    #[prop(into)] api_base_url: Signal<String>,
    /// The ID of the entity being edited.
    #[prop(into)]
    id: Signal<T::ReadModelId>,
    #[prop(into)] data_provider: Signal<CrudRestDataProvider<T>>,
    #[prop(into)] actions: Signal<Vec<CrudEntityAction<T>>>,
    #[prop(into)] elements: Signal<Vec<Elem<T::UpdateModel>>>,
    #[prop(into)] custom_fields: Signal<CustomUpdateFields<T, leptos::View>>,
    #[prop(into)] field_config: Signal<
        HashMap<<T::UpdateModel as CrudDataTrait>::Field, DynSelectConfig>,
    >,
    #[prop(into)] on_list_view: Callback<()>,
    #[prop(into)] on_tab_selected: Callback<TabId>,
) -> impl IntoView
where
    T: CrudMainTrait + 'static,
{
    let instance_ctx = expect_context::<CrudInstanceContext<T>>();

    let entity_resource = create_local_resource(
        move || {
            tracing::debug!("entity_req");
            let id = id.get();
            let equals_id_condition =
                <T as CrudMainTrait>::ReadModelId::fields_iter(&id) // TODO: This is complex and requires several use statements. Should be made easier.
                    .map(|field| (field.name().to_owned(), field.to_value()))
                    .into_all_equal_condition();
            EntityReq {
                condition: merge_conditions(
                    instance_ctx.base_condition.get(),
                    Some(equals_id_condition),
                ),
                reload: instance_ctx.reload.get(),
                data_provider: data_provider.get(),
            }
        },
        move |req| async move {
            req.data_provider
                .read_one(ReadOne {
                    skip: None,
                    order_by: None,
                    condition: req.condition,
                })
                .await
        },
    );

    // Stores the current state of the entity or an error, if no entity could be fetched.
    // Until the initial fetch request is completed, this is in the `Err(NoDataAvailable::NotYetLoaded` state!
    let (entity, set_entity) = create_signal(
        Result::<ReadSignal<T::UpdateModel>, NoDataAvailable>::Err(NoDataAvailable::NotYetLoaded),
    );

    // TODO: Read and Edit view have some things in common, like loading the current entity and creating the signals map. Can this be simplified or extracted?
    let (signals, set_sig) = create_signal::<
        StoredValue<HashMap<<T::UpdateModel as CrudDataTrait>::Field, ReactiveValue>>,
    >(store_value(HashMap::new()));

    // Update the `entity` signal whenever we fetched a new version of the edited entity.
    create_effect(move |_prev| {
        set_entity.set(match entity_resource.get() {
            Some(result) => {
                tracing::info!("loaded entity data");
                match result {
                    Ok(data) => match data {
                        Some(data) => {
                            let update_model = data.into();

                            // Creating signals for all fields of the loaded entity, so that input fields can work on the data.
                            set_sig.set({
                                let mut map = HashMap::new();
                                for field in T::UpdateModel::get_all_fields() {
                                    let initial = field.get_value(&update_model);
                                    map.insert(field, initial.into_reactive_value());
                                }
                                store_value(map)
                            });

                            Ok(create_rw_signal(update_model).read_only())
                        }
                        None => Err(NoDataAvailable::RequestReturnedNoData(format!(
                            "Eintrag existiert nicht."
                        ))),
                    },
                    Err(reason) => Err(NoDataAvailable::RequestFailed(reason)),
                }
            }
            None => Err(NoDataAvailable::NotYetLoaded),
        })
    });

    let value_changed = Callback::new(move |_| {});

    let action_ctx = CrudActionContext::<T>::new();
    let maybe_entity = Signal::derive(move || {
        if let Ok(entity) = entity.get() {
            Some(entity.get())
        } else {
            None
        }
    });

    view! {
        {move || match (entity.get(), signals.get()) {
            (Ok(entity), signals) => {
                let on_list_view = on_list_view.clone();
                view! {
                    {move || {
                        let on_list_view = on_list_view.clone();
                        view! {
                            <Grid spacing=Size::Em(0.6) class="crud-nav">
                                <Row>
                                    <Col xs=6 h_align=ColAlign::Start>
                                        <CrudActionButtons
                                            action_ctx=action_ctx
                                            actions=actions
                                            input=maybe_entity
                                            required_state=States::Read
                                        />
                                    </Col>
                                    <Col xs=6 h_align=ColAlign::End>
                                        <ButtonWrapper>
                                            <Button
                                                color=ButtonColor::Secondary
                                                on_click=move |_| on_list_view.call(())
                                            >
                                                <span style="text-decoration: underline;">{"L"}</span>
                                                {"istenansicht"}
                                            </Button>
                                        </ButtonWrapper>
                                    </Col>
                                </Row>
                            </Grid>
                        }
                    }}

                    <CrudFields
                        custom_fields=custom_fields
                        field_config=field_config
                        api_base_url=api_base_url
                        elements=elements
                        signals=signals
                        mode=FieldMode::Readable
                        current_view=CrudSimpleView::Read
                        value_changed=value_changed.clone()
                        // active_tab={ctx.props().config.active_tab.clone()}
                        on_tab_selection=on_tab_selected.clone()
                        entity=entity.into()
                    />
                }.into_view()
            }
            (Err(no_data), _) => {
                let on_list_view = on_list_view.clone();
                view! {
                    <Grid spacing=Size::Em(0.6) class="crud-nav">
                        <Row>
                            <Col h_align=ColAlign::End>
                                <ButtonWrapper>
                                    <Button color=ButtonColor::Secondary on_click=move |_| on_list_view.call(())>
                                        <span style="text-decoration: underline;">{"L"}</span>
                                        {"istenansicht"}
                                    </Button>
                                </ButtonWrapper>
                            </Col>
                        </Row>
                    </Grid>
                    <div>{format!("Daten nicht verfügbar: {:?}", no_data)}</div>
                }.into_view()
            }
        }}
    }
}
