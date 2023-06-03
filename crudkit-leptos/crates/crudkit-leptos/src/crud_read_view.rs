use std::marker::PhantomData;

use crudkit_condition::IntoAllEqualCondition;
use crudkit_id::{Id, IdField};
use crudkit_web::{
    prelude::{CrudRestDataProvider, CustomUpdateFields, ReadOne},
    CrudMainTrait, CrudSimpleView, Elem, FieldMode, TabId,
};
use leptonic::prelude::*;
use leptos::*;
use uuid::Uuid;

use crate::{
    crud_fields::CrudFields, crud_instance::CrudInstanceContext, crud_table::NoDataAvailable,
};

#[derive(Debug, Clone, PartialEq)]
struct EntityReq<T: CrudMainTrait + 'static> {
    reload: Uuid,
    id: T::ReadModelId,
    data_provider: CrudRestDataProvider<T>,
}

#[component]
pub fn CrudReadView<T>(
    cx: Scope,
    _phantom: PhantomData<T>,
    #[prop(into)] api_base_url: Signal<String>,
    /// The ID of the entity being edited.
    #[prop(into)]
    id: Signal<T::ReadModelId>,
    #[prop(into)] data_provider: Signal<CrudRestDataProvider<T>>,
    #[prop(into)] elements: Signal<Vec<Elem<T::UpdateModel>>>,
    #[prop(into)] custom_fields: Signal<CustomUpdateFields<T, leptos::View>>,
    on_list_view: Callback<()>,
    on_tab_selected: Callback<TabId>,
) -> impl IntoView
where
    T: CrudMainTrait + 'static,
{
    let instance_ctx = expect_context::<CrudInstanceContext<T>>(cx);

    let entity_resource = create_local_resource(
        cx,
        move || {
            tracing::debug!("entity_req");
            EntityReq {
                reload: instance_ctx.reload.get(),
                id: id.get(),
                data_provider: data_provider.get(),
            }
        },
        move |req| async move {
            req.data_provider
            .read_one(ReadOne {
                skip: None,
                order_by: None,
                condition: Some(<T as CrudMainTrait>::ReadModelId::fields_iter(&req.id) // TODO: This is complex and requires several use statements. Should be made easier.
                .map(|field| (field.name().to_owned(), field.to_value()))
                .into_all_equal_condition()),
            })
            .await
        },
    );

    // Stores the current state of the entity or an error, if no entity could be fetched.
    // Until the initial fetch request is completed, this is in the `Err(NoDataAvailable::NotYetLoaded` state!
    let (entity, set_entity) = create_signal(
        cx,
        Result::<StoredValue<T::UpdateModel>, NoDataAvailable>::Err(NoDataAvailable::NotYetLoaded),
    );

    // Update the `entity` signal whenever we fetched a new version of the edited entity.
    create_effect(cx, move |_prev| {
        set_entity.set(match entity_resource.read(cx) {
            Some(result) => {
                tracing::info!("loaded entity data");
                match result {
                    Ok(data) => match data {
                        Some(data) => {
                            let update_model = data.into();
                            Ok(store_value(cx, update_model))
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

    let value_changed = Callback::new(cx, move |_| {});

    view! {cx,
        { move || match entity.get() {
            Ok(entity) => view! {cx,
                { move || {
                    view! {cx,
                        <Grid spacing=6 class="crud-nav">
                            <Row>
                                <Col h_align=ColAlign::End>
                                    <ButtonWrapper>
                                        <Button color=ButtonColor::Secondary on_click=move |_| on_list_view.call(())>
                                            <span style="text-decoration: underline;">{"L"}</span>{"istenansicht"}
                                        </Button>
                                    </ButtonWrapper>
                                </Col>
                            </Row>
                        </Grid>
                    }
                } }

                <CrudFields
                    // children={ChildrenRenderer::new(ctx.props().children.iter().filter(|it| match it {
                    //     Item::NestedInstance(_) => true,
                    //     Item::Relation(_) => true,
                    //     Item::Select(select) => select.props.for_model == crate::crud_reset_field::Model::Update,
                    // }).collect::<Vec<Item>>())}
                    custom_fields=custom_fields
                    api_base_url=api_base_url
                    elements=elements
                    entity=entity
                    mode=FieldMode::Readable
                    current_view=CrudSimpleView::Read
                    value_changed=value_changed
                    //     active_tab={ctx.props().config.active_tab.clone()}
                    // TODO: Propagate tab selections: ctx.props().on_tab_selected.emit(label);
                    on_tab_selection=on_tab_selected
                />
            }.into_view(cx),
            Err(no_data) => view! {cx,
                <Grid spacing=6 class="crud-nav">
                    <Row>
                        <Col h_align=ColAlign::End>
                            <ButtonWrapper>
                                <Button color=ButtonColor::Secondary on_click=move |_| on_list_view.call(())>
                                    <span style="text-decoration: underline;">{"L"}</span>{"istenansicht"}
                                </Button>
                            </ButtonWrapper>
                        </Col>
                    </Row>
                </Grid>
                <div>
                    {format!("Daten nicht verfügbar: {:?}", no_data)}
                </div>
            }.into_view(cx),
        } }
    }
}
