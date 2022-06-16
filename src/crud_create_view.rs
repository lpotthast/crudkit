use yew::{html::ChildrenRenderer, prelude::*};

use crate::crud_instance::Item;

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
    pub children: ChildrenRenderer<Item>,
    pub api_base_url: String,
    pub parent_id: Option<u32>,
    pub config: CrudInstanceConfig<T>,
    pub list_view_available: bool,
    pub on_list_view: Callback<()>,
    pub on_entity_created: Callback<(T, Option<CrudView>)>,
}

pub struct CrudCreateView<T: CrudDataTrait> {
    initial_data: Option<T>,
    input: T,
    ongoing_save: bool,
}

impl<T: 'static + CrudDataTrait> CrudCreateView<T> {
    fn create_entity(&mut self, ctx: &Context<Self>, then: Then) {
        let base_url = ctx.props().api_base_url.clone();
        let ent = self.input.clone();
        self.ongoing_save = true;
        ctx.link().send_future(async move {
            Msg::CreatedEntity(
                create_one::<T>(&base_url, CreateOne { entity: ent }).await,
                then,
            )
        });
    }

    fn reset(&mut self) {
        self.input = Default::default();
    }
}

impl<T: 'static + CrudDataTrait> Component for CrudCreateView<T> {
    type Message = Msg<T>;
    type Properties = Props<T>;

    fn create(ctx: &Context<Self>) -> Self {
        let mut entity: T = Default::default();
        if let Some(nested) = &ctx.props().config.nested {
            if let Some(parent_id) = ctx.props().parent_id {
                T::get_field(nested.reference_field.as_str())
                    .set_value(&mut entity, format!("{}", parent_id));
                log::info!("successfully set parent id to reference field");
            } else {
                log::error!("CrudInstance is configured to be a nested instance but no parent id was passed down!");
            }
        } else {
            log::info!("no nested config");
        }
        Self {
            initial_data: Some(entity.clone()),
            input: entity,
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
                        Some(entity) => match then {
                            Then::DoNothing => {
                                ctx.props().on_entity_created.emit((entity, None));
                            }
                            Then::OpenListView => {
                                ctx.props()
                                    .on_entity_created
                                    .emit((entity, Some(CrudView::List)));
                            }
                            Then::Reset => {
                                ctx.props()
                                    .on_entity_created
                                    .emit((entity, Some(CrudView::Create)));
                                self.reset();
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
                        <CrudBtn name={"Speichern"} variant={Variant::Primary} disabled={self.ongoing_save} onclick={ctx.link().callback(|_| Msg::Save)}>
                            <CrudBtn name={"Speichern und zurück"} variant={Variant::Primary} disabled={self.ongoing_save} onclick={ctx.link().callback(|_| Msg::SaveAndReturn)} />
                            <CrudBtn name={"Speichern und neu"} variant={Variant::Primary} disabled={self.ongoing_save} onclick={ctx.link().callback(|_| Msg::SaveAndNew)} />
                        </CrudBtn>
                    </CrudBtnWrapper>
                </div>

                <div class={"crud-col crud-col-flex-end"}>
                    <CrudBtnWrapper>
                        <CrudBtn name={"_back"} variant={Variant::Default} onclick={ctx.link().callback(|_| Msg::Back)}>
                            <CrudBtnName>
                                <span style="text-decoration: underline;">{"L"}</span>{"istenansicht"}
                            </CrudBtnName>
                        </CrudBtn>
                    </CrudBtnWrapper>
                </div>
            </div>

            <CrudFields<T>
                api_base_url={ctx.props().api_base_url.clone()}
                children={ctx.props().children.clone()}
                elements={ctx.props().config.elements.clone()}
                entity={self.initial_data.clone()}
                mode={FieldMode::Editable}
                value_changed={ctx.link().callback(Msg::ValueChanged)}
            />
            </>
        }
    }

    fn rendered(&mut self, _ctx: &Context<Self>, first_render: bool) {
        if first_render {
            // We only want to pass down data once!!!
            // After we did that, no rerender of this component must overwrite the users input data!
            self.initial_data = None;
        }
    }
}
