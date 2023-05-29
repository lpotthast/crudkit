use std::marker::PhantomData;

use crudkit_condition::IntoAllEqualCondition;
use crudkit_id::{Id, IdField};
use crudkit_web::{
    prelude::{CrudRestDataProvider, ReadOne},
    requests::RequestError,
    CrudMainTrait,
};
use leptonic::prelude::*;
use leptos::*;
use uuid::Uuid;

use crate::{
    crud_instance::CrudInstanceContext, crud_leave_modal::CrudLeaveModal,
    crud_table::NoDataAvailable,
};

#[derive(Debug, Clone, PartialEq)]
struct EntityReq<T: CrudMainTrait + 'static> {
    reload: Uuid,
    id: T::UpdateModelId,
    data_provider: CrudRestDataProvider<T>,
}

async fn load_entity<T: CrudMainTrait + 'static>(
    req: EntityReq<T>,
) -> Result<Option<T::ReadModel>, RequestError> {
    // TODO: This is complex and requires several use statements. Should be made easier.
    let condition = <T as CrudMainTrait>::UpdateModelId::fields_iter(&req.id)
        .map(|field| (field.name().to_owned(), field.to_value()))
        .into_all_equal_condition();

    req.data_provider
        .read_one(ReadOne {
            skip: None,
            order_by: None,
            condition: Some(condition),
        })
        .await
}

#[component]
pub fn CrudEditView<T>(
    cx: Scope,
    _phantom: PhantomData<T>,
    /// The ID of the entity being edited.
    #[prop(into)]
    id: MaybeSignal<T::UpdateModelId>,
    #[prop(into)] data_provider: Signal<CrudRestDataProvider<T>>,
) -> impl IntoView
where
    T: CrudMainTrait + 'static,
{
    let instance_ctx = expect_context::<CrudInstanceContext<T>>(cx);

    // Whenever this signal returns a new/different value, the data of the currently viewed entity is re-fetched.
    let entity_req = Signal::derive(cx, move || {
        tracing::debug!("entity_req");
        // Every server-provided resource must be reloaded when a general reload is requested!
        let reload = instance_ctx.reload.get();
        let id = id.get();
        let data_provider = data_provider.get();
        EntityReq {
            reload,
            id,
            data_provider,
        }
    });

    let entity_res = create_local_resource(cx, move || entity_req.get(), load_entity);

    // TODO: create_memo or Signal::derive??? We only want this once..
    let data = create_memo(cx, move |_prev| match entity_res.read(cx) {
        Some(result) => {
            tracing::info!("loaded entity data");
            match result {
                Ok(data) => Ok(data),
                Err(reason) => Err(NoDataAvailable::FetchFailed(reason)),
            }
        }
        None => Err(NoDataAvailable::NotYetLoaded),
    });

    let (user_wants_to_leave, set_user_wants_to_leave) = create_signal(cx, false);

    let trigger_leave = move || {};

    view! {cx,
        { move || match data.get() {
            Ok(data) => view! {cx,
                <div>
                    "fields"
                </div>
            }.into_view(cx),
            Err(no_data) => view! {cx,
                <Grid spacing=6 class="crud-nav">
                    <Row>
                        <Col h_align=ColAlign::End>
                            <ButtonWrapper>
                                <Button color=ButtonColor::Secondary on_click=move |_| trigger_leave()>
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

        <CrudLeaveModal
            show_when=user_wants_to_leave
            on_cancel=move || set_user_wants_to_leave.set(false)
            on_accept=move || set_user_wants_to_leave.set(false)
        />
    }
}
