use crud_shared_types::{ConditionClause, ConditionElement};
use yew::prelude::*;

use super::{
    prelude::*,
    services::controller::{read_one, ReadOne},
    types::RequestError,
};

pub enum Msg<T: CrudDataTrait> {
    Back,
    LoadedEntity(Result<Option<T>, RequestError>),
}

#[derive(Properties, PartialEq)]
pub struct Props<T: CrudDataTrait> {
    pub config: CrudInstanceConfig<T>,
    pub id: u32,
    pub list_view_available: bool,
    pub on_list_view: Callback<()>,
}

pub struct CrudReadView<T: CrudDataTrait> {
    entity: Result<T, NoData>,
}

impl<T: 'static + CrudDataTrait> Component for CrudReadView<T> {
    type Message = Msg<T>;
    type Properties = Props<T>;

    fn create(ctx: &Context<Self>) -> Self {
        let id = ctx.props().id;
        ctx.link()
            .send_future(async move { Msg::LoadedEntity(load_entity(id).await) });
        Self {
            entity: Err(NoData::NotYetLoaded),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Back => {
                ctx.props().on_list_view.emit(());
                false
            }
            Msg::LoadedEntity(data) => {
                self.entity = match data {
                    Ok(data) => match data {
                        Some(entity) => Ok(entity),
                        None => Err(NoData::FetchReturnedNothing),
                    },
                    Err(err) => Err(NoData::FetchFailed(err)),
                };
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div>
                {
                    match &self.entity {
                        Ok(entity) => {
                            html! {
                                <>
                                <div class={"crud-row crud-nav"}>
                                    <div class={"crud-col crud-col-flex-end"}>
                                        <CrudBtnWrapper>
                                            <CrudBtn name={"_back"} variant={Variant::Default} onclick={ctx.link().callback(|_| Msg::Back)}>
                                                <span style="text-decoration: underline;">{"L"}</span>{"ist view"}
                                            </CrudBtn>
                                        </CrudBtnWrapper>
                                    </div>
                                </div>

                                <CrudFields<T>
                                    elements={ctx.props().config.elements.clone()}
                                    entity={entity.clone()}
                                    mode={FieldMode::Readable}
                                    value_changed={|_| {}}
                                />
                                </>
                            }
                        }
                        Err(reason) => {
                            html! {
                                <>
                                <div class={"crud-row crud-nav"}>
                                    <div class={"crud-col crud-col-flex-end"}>
                                        <CrudBtnWrapper>
                                            <CrudBtn name={"_back"} variant={Variant::Default} onclick={ctx.link().callback(|_| Msg::Back)}>
                                                <span style="text-decoration: underline;">{"L"}</span>{"ist view"}
                                            </CrudBtn>
                                        </CrudBtnWrapper>
                                    </div>
                                </div>
                                <div>
                                    {format!("Data not available: {:?}", reason)}
                                </div>
                                </>
                            }
                        }
                    }
                }
            </div>
        }
    }
}

pub async fn load_entity<T: CrudDataTrait>(id: u32) -> Result<Option<T>, RequestError> {
    read_one::<T>(ReadOne {
        skip: None,
        order_by: None,
        condition: Some(vec![ConditionElement::Clause(ConditionClause {
            column_name: T::get_id_field_name(),
            operator: crud_shared_types::Operator::Equal,
            value: crud_shared_types::ConditionClauseValue::U32(id),
        })]),
    })
    .await
}
