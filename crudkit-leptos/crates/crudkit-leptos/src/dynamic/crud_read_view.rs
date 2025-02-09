use crate::dynamic::crud_action::{CrudEntityAction, States};
use crate::dynamic::crud_action_buttons::CrudActionButtons;
use crate::dynamic::crud_action_context::CrudActionContext;
use crate::dynamic::crud_fields::CrudFields;
use crate::dynamic::crud_instance::CrudInstanceContext;
use crate::dynamic::crud_table::NoDataAvailable;
use crate::dynamic::custom_field::CustomUpdateFields;
use crate::shared::crud_instance_config::DynSelectConfig;
use crate::ReactiveValue;
use crudkit_condition::{merge_conditions, IntoAllEqualCondition};
use crudkit_id::{Id, SerializableId};
use crudkit_web::crud_rest_data_provider_dyn::{CrudRestDataProvider, ReadOne};
use crudkit_web::prelude::RequestError;
use crudkit_web::{AnyElem, AnyField, AnyModel, CrudSimpleView, FieldMode, TabId};
use leptonic::components::prelude::*;
use leptonic::prelude::*;
use leptos::prelude::*;
use std::collections::HashMap;

#[component]
pub fn CrudReadView(
    #[prop(into)] api_base_url: Signal<String>,
    /// The ID of the entity being edited.
    #[prop(into)]
    id: Signal<SerializableId>,
    #[prop(into)] data_provider: Signal<CrudRestDataProvider>,
    #[prop(into)] actions: Signal<Vec<CrudEntityAction>>,
    #[prop(into)] elements: Signal<Vec<AnyElem>>, // UpdateModel
    #[prop(into)] custom_fields: Signal<CustomUpdateFields>,
    #[prop(into)] field_config: Signal<HashMap<AnyField, DynSelectConfig>>, // UpdateModel field
    #[prop(into)] on_list_view: Callback<()>,
    #[prop(into)] on_tab_selected: Callback<(TabId,)>,
) -> impl IntoView {
    let instance_ctx = expect_context::<CrudInstanceContext>();

    let entity_resource = LocalResource::new(move || async move {
        tracing::debug!("entity_req");
        let _ = instance_ctx.reload.get();
        let equals_id_condition = id.get().0.into_iter().into_all_equal_condition();
        data_provider
            .read()
            .read_one(ReadOne {
                skip: None,
                order_by: None,
                condition: merge_conditions(
                    instance_ctx.base_condition.get(),
                    Some(equals_id_condition),
                ),
            })
            .await
            .and_then(|json| {
                instance_ctx
                    .static_config
                    .read_value()
                    .deserialize_read_one_response
                    .run((json,))
                    .map_err(|de_err| RequestError::Deserialize(de_err.to_string()))
            })
    });

    // Stores the current state of the entity or an error, if no entity could be fetched.
    // Until the initial fetch request is completed, this is in the `Err(NoDataAvailable::NotYetLoaded` state!
    let (entity, set_entity) = signal(Result::<ReadSignal<AnyModel>, NoDataAvailable>::Err(
        NoDataAvailable::NotYetLoaded,
    ));

    // TODO: Read and Edit view have some things in common, like loading the current entity and creating the signals map. Can this be simplified or extracted?
    let (signals, set_sig) =
        signal::<StoredValue<HashMap<AnyField, ReactiveValue>>>(StoredValue::new(HashMap::new()));

    // Update the `entity` signal whenever we fetched a new version of the edited entity.
    Effect::new(move |_prev| {
        set_entity.set(match entity_resource.get() {
            Some(result) => {
                tracing::info!("loaded entity data");
                match result.take() {
                    Ok(data) => match data {
                        Some(read_model) => {
                            let update_model = instance_ctx
                                .static_config
                                .read_value()
                                .read_model_to_update_model
                                .run((read_model,));

                            // Creating signals for all fields of the loaded entity, so that input fields can work on the data.
                            set_sig.set({
                                let signals = instance_ctx
                                    .static_config
                                    .read_value()
                                    .update_model_to_signal_map
                                    .run((update_model.clone(),));
                                StoredValue::new(signals)
                            });

                            Ok(RwSignal::new(update_model).read_only())
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

    let action_ctx = CrudActionContext::new();
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
                            <Grid gap=Size::Em(0.6) attr:class="crud-nav">
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
                                                on_press=move |_| on_list_view.run(())
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
                    />
                }.into_any()
            }
            (Err(no_data), _) => {
                let on_list_view = on_list_view.clone();
                view! {
                    <Grid gap=Size::Em(0.6) attr:class="crud-nav">
                        <Row>
                            <Col h_align=ColAlign::End>
                                <ButtonWrapper>
                                    <Button color=ButtonColor::Secondary on_press=move |_| on_list_view.run(())>
                                        <span style="text-decoration: underline;">{"L"}</span>
                                        {"istenansicht"}
                                    </Button>
                                </ButtonWrapper>
                            </Col>
                        </Row>
                    </Grid>
                    <div>{format!("Daten nicht verfügbar: {:?}", no_data)}</div>
                }.into_any()
            }
        }}
    }
}
