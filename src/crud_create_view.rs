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
    CreatedEntity(Result<Option<T>, RequestError>, Then),
}

pub enum Then {
    DoNothing,
    OpenListView,
    Reset,
}

#[derive(Properties, PartialEq)]
pub struct Props<T: CrudDataTrait> {
    pub api_base_url: String,
    pub config: CrudInstanceConfig<T>,
    pub list_view_available: bool,
    pub on_list_view: Callback<()>,
    pub on_entity_created: Callback<(T, Option<CrudView>)>,
}

pub struct CrudCreateView<T: CrudDataTrait> {
    input: T,
    ongoing_save: bool,
}

impl<T: 'static + CrudDataTrait> CrudCreateView<T> {
    fn create_entity(&mut self, ctx: &Context<Self>, then: Then) {
        let base_url = ctx.props().api_base_url.clone();
        let ent = self.input.clone();
        self.ongoing_save = true;
        ctx.link().send_future(async move {
            Msg::CreatedEntity(create_one::<T>(&base_url, CreateOne { entity: ent }).await, then)
        });
    }

    fn reset(&mut self) {
        self.input = Default::default();
    }
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
                self.create_entity(ctx, Then::DoNothing);
                false
            }
            Msg::SaveAndReturn => {
                self.create_entity(ctx, Then::OpenListView);
                false
            }
            Msg::SaveAndNew => {
                self.create_entity(ctx, Then::Reset);
                false
            }
            Msg::ValueChanged((field, value)) => {
                field.set_value(&mut self.input, value);
                false
            }
            Msg::CreatedEntity(result, then) => {
                self.ongoing_save = false;
                match result {
                    Ok(data) => match data {
                        Some(entity) => {
                            match then {
                                Then::DoNothing => {
                                    ctx.props().on_entity_created.emit((entity, None));
                                },
                                Then::OpenListView => {
                                    ctx.props().on_entity_created.emit((entity, Some(CrudView::List)));
                                },
                                Then::Reset => {
                                    ctx.props().on_entity_created.emit((entity, Some(CrudView::Create)));
                                    self.reset();
                                },
                            }
                        },
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
                api_base_url={ctx.props().api_base_url.clone()}
                elements={ctx.props().config.elements.clone()}
                entity={None}
                mode={FieldMode::Editable}
                value_changed={ctx.link().callback(Msg::ValueChanged)}
            />
            </>
        }
    }
}
