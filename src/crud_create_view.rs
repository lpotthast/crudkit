use yew::prelude::*;

use super::{
    prelude::*,
    services::controller::{create_one, CreateOne},
    types::RequestError,
};

pub enum Msg<T: CrudDataTrait> {
    Back,
    Save,
    SaveAndReturn,
    SaveAndNew,
    ValueChanged((T::FieldType, String)),
    CreatedEntity(Result<Option<T>, RequestError>),
}

#[derive(Properties, PartialEq)]
pub struct Props<T: CrudDataTrait> {
    pub config: CrudInstanceConfig<T>,
    pub list_view_available: bool,
    pub on_list_view: Callback<()>,
    pub on_entity_created: Callback<T>,
}

pub struct CrudCreateView<T: CrudDataTrait> {
    input: T,
    ongoing_save: bool,
}

impl<T: 'static + CrudDataTrait> Component for CrudCreateView<T> {
    type Message = Msg<T>;
    type Properties = Props<T>;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            input: Default::default(),
            ongoing_save: false,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Back => {
                ctx.props().on_list_view.emit(());
                false
            }
            Msg::Save => {
                let ent = self.input.clone();
                ctx.link().send_future(async move {
                    Msg::CreatedEntity(create_one::<T>(CreateOne { entity: ent }).await)
                });
                false
            }
            Msg::SaveAndReturn => todo!(),
            Msg::SaveAndNew => todo!(),
            Msg::ValueChanged((field, value)) => {
                field.set_value(&mut self.input, value);
                false
            }
            Msg::CreatedEntity(result) => {
                match result {
                    Ok(data) => match data {
                        Some(entity) => ctx.props().on_entity_created.emit(entity),
                        None => log::error!(
                            "Entity creation failed: {:?}",
                            NoData::FetchReturnedNothing
                        ),
                    },
                    Err(reason) => log::error!("Entity creation failed: {:?}", reason),
                }
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <>
            <div class={"crud-row crud-nav"}>
                <div class={"crud-col"}>
                    <CrudBtnWrapper>
                        <CrudBtn name={"Save"} variant={Variant::Primary} disabled={self.ongoing_save} onclick={ctx.link().callback(|_| Msg::Save)} />
                        <CrudBtn name={"Save and return"} variant={Variant::Primary} disabled={self.ongoing_save} onclick={ctx.link().callback(|_| Msg::SaveAndReturn)} />
                        <CrudBtn name={"Save and new"} variant={Variant::Primary} disabled={self.ongoing_save} onclick={ctx.link().callback(|_| Msg::SaveAndNew)} />
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
                elements={ctx.props().config.elements.clone()}
                entity={None}
                mode={FieldMode::Editable}
                value_changed={ctx.link().callback(Msg::ValueChanged)}
            />
            </>
        }
    }
}
