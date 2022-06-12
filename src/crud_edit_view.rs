use crud_shared_types::{ConditionClause, ConditionClauseValue, ConditionElement, Operator};
use yew::prelude::*;

use super::{
    prelude::*,
    services::controller::{read_one, update_one, ReadOne, UpdateOne},
    types::RequestError,
};

pub enum Msg<T: CrudDataTrait> {
    Back,
    BackCanceled,
    BackApproved,
    LoadedEntity(Result<Option<T>, RequestError>),
    UpdatedEntity(Result<Option<T>, RequestError>),
    Save,
    SaveAndReturn,
    SaveAndNew,
    Delete,
    ValueChanged((T::FieldType, String)),
}

#[derive(Properties, PartialEq)]
pub struct Props<T: CrudDataTrait> {
    pub api_base_url: String,
    pub config: CrudInstanceConfig<T>,
    pub id: u32,
    pub list_view_available: bool,
    pub on_list: Callback<()>,
    pub on_create: Callback<()>,
    pub on_delete: Callback<T>,
}

pub struct CrudEditView<T: CrudDataTrait> {
    input: T,
    input_dirty: bool,
    user_wants_to_leave: bool,
    entity: Result<T, NoData>,
    ongoing_save: bool,
}

enum SetFrom {
    Fetch,
    Update,
}

impl<T: 'static + CrudDataTrait> CrudEditView<T> {
    fn set_entity(&mut self, data: Result<Option<T>, RequestError>, from: SetFrom) {
        self.entity = match data {
            Ok(data) => match data {
                Some(entity) => Ok(entity),
                None => Err(match from {
                    SetFrom::Fetch => NoData::FetchReturnedNothing,
                    SetFrom::Update => NoData::UpdateReturnedNothing,
                }),
            },
            Err(err) => Err(match from {
                SetFrom::Fetch => NoData::FetchFailed(err),
                SetFrom::Update => NoData::UpdateFailed(err),
            }),
        };
        if let Ok(entity) = &self.entity {
            self.input = entity.clone();
            self.input_dirty = false;
        }
    }

    fn save_entity(&self, ctx: &Context<Self>) {
        let base_url = ctx.props().api_base_url.clone();
        let ent = self.input.clone();
        let id = ctx.props().id;
        // TODO: Like in create_view, store ongoing_save!!
        ctx.link().send_future(async move {
            Msg::UpdatedEntity(
                update_one::<T>(&base_url, UpdateOne {
                    entity: ent,
                    condition: Some(vec![ConditionElement::Clause(ConditionClause {
                        column_name: T::get_id_field_name(),
                        operator: Operator::Equal,
                        value: ConditionClauseValue::U32(id),
                    })]),
                })
                .await,
            )
        });
    }
}

impl<T: 'static + CrudDataTrait> Component for CrudEditView<T> {
    type Message = Msg<T>;
    type Properties = Props<T>;

    fn create(ctx: &Context<Self>) -> Self {
        let base_url = ctx.props().api_base_url.clone();
        let id = ctx.props().id;
        ctx.link()
            .send_future(async move { Msg::LoadedEntity(load_entity(&base_url, id).await) });
        Self {
            input: Default::default(),
            input_dirty: false,
            user_wants_to_leave: false,
            entity: Err(NoData::NotYetLoaded),
            ongoing_save: false,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Back => {
                self.user_wants_to_leave = true;
                match self.input_dirty {
                    true => {
                        // Waiting for the modal to be resolved!
                        true
                    }
                    false => {
                        ctx.props().on_list.emit(());
                        false
                    }
                }
            }
            Msg::BackCanceled => {
                self.user_wants_to_leave = false;
                true
            }
            Msg::BackApproved => {
                ctx.props().on_list.emit(());
                false
            }
            Msg::LoadedEntity(data) => {
                self.set_entity(data, SetFrom::Fetch);
                true
            }
            Msg::UpdatedEntity(data) => {
                self.set_entity(data, SetFrom::Update);
                true
            }
            Msg::Save => {
                self.save_entity(ctx);
                true
            }
            Msg::SaveAndReturn => {
                self.save_entity(ctx);
                ctx.props().on_list.emit(());
                false
            }
            Msg::SaveAndNew => {
                self.save_entity(ctx);
                ctx.props().on_create.emit(());
                false
            }
            Msg::Delete => {
                match &self.entity {
                    Ok(entity) => ctx.props().on_delete.emit(entity.clone()),
                    Err(_) => log::warn!(
                        "Cannot issue a delete event, as not entity is currently loaded!"
                    ),
                }
                false
            }
            Msg::ValueChanged((field, value)) => {
                field.set_value(&mut self.input, value);
                // We might only want to set this to true if the new value was actually different to the old value!
                match &self.entity {
                    Ok(entity) => self.input_dirty = &self.input != entity,
                    Err(_) => self.input_dirty = false,
                }
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div>
                {
                    match &self.entity {
                        Ok(_entity) => {
                            html! {
                                <>
                                <div class={"crud-row crud-nav"}>
                                    <div class={"crud-col"}>
                                        <CrudBtnWrapper>
                                            <CrudBtn name={"Save"} variant={Variant::Primary} disabled={self.ongoing_save} onclick={ctx.link().callback(|_| Msg::Save)} />
                                            <CrudBtn name={"Save and return"} variant={Variant::Primary} disabled={self.ongoing_save} onclick={ctx.link().callback(|_| Msg::SaveAndReturn)} />
                                            <CrudBtn name={"Save and new"} variant={Variant::Primary} disabled={self.ongoing_save} onclick={ctx.link().callback(|_| Msg::SaveAndNew)} />
                                            <CrudBtn name={"Delete"} variant={Variant::Danger} disabled={self.ongoing_save} onclick={ctx.link().callback(|_| Msg::Delete)} />
                                        </CrudBtnWrapper>
                                    </div>

                                    <div class={"crud-col crud-col-flex-end"}>
                                        <CrudBtnWrapper>
                                            <CrudBtn name={"_back"} variant={Variant::Default} onclick={ctx.link().callback(|_| Msg::Back)}>
                                                <span style="text-decoration: underline;">{"L"}</span>{"ist view"}
                                            </CrudBtn>
                                        </CrudBtnWrapper>
                                    </div>
                                </div>

                                <CrudFields<T>
                                    api_base_url={ctx.props().api_base_url.clone()}
                                    elements={ctx.props().config.elements.clone()}
                                    entity={self.input.clone()}
                                    mode={FieldMode::Editable}
                                    value_changed={ctx.link().callback(Msg::ValueChanged)}
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
                if self.user_wants_to_leave {
                    <CrudModal>
                        <CrudLeaveModal
                            on_cancel={ctx.link().callback(|_| Msg::BackCanceled)}
                            on_leave={ctx.link().callback(|_| Msg::BackApproved)}
                        />
                    </CrudModal>
                }
            </div>
        }
    }
}

pub async fn load_entity<T: CrudDataTrait>(base_url: &str, id: u32) -> Result<Option<T>, RequestError> {
    read_one::<T>(base_url, ReadOne {
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
