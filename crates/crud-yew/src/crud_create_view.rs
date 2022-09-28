use crud_shared_types::{SaveResult, Saved};
use yew::{
    html::{ChildrenRenderer, Scope},
    prelude::*,
};

use crate::{
    crud_instance::Item,
    services::crud_rest_data_provider::{CreateOne, CrudRestDataProvider},
};

use super::{prelude::*, types::RequestError};

pub enum Msg<T: CrudMainTrait> {
    Back,
    Save,
    SaveAndReturn,
    SaveAndNew,
    TabSelected(Label),
    /// This message must only be sent if the component is actually using the CreateModel, the program will otherwise panic!
    CreateModelFieldChanged((<T::CreateModel as CrudDataTrait>::Field, Value)),
    /// This message must only be sent if the component is actually using the UpdateModel, the program will otherwise panic!
    UpdateModelFieldChanged((<T::UpdateModel as CrudDataTrait>::Field, Value)),
    // After saving an entity, the CRUD system always return the UpdateModel!
    CreatedEntity(Result<SaveResult<T::UpdateModel>, RequestError>, Then),
    /// This message must only be sent if the component is actually using the CreateModel, the program will otherwise panic!
    GetCreateModelFieldValue(
        (
            <T::CreateModel as CrudDataTrait>::Field,
            Box<dyn FnOnce(Value)>,
        ),
    ),
    /// This message must only be sent if the component is actually using the UpdateModel, the program will otherwise panic!
    GetUpdateModelFieldValue(
        (
            <T::UpdateModel as CrudDataTrait>::Field,
            Box<dyn FnOnce(Value)>,
        ),
    ),
}

pub enum Then {
    DoNothing,
    OpenListView,
    Reset,
}

#[derive(Properties, PartialEq)]
pub struct Props<T: 'static + CrudMainTrait> {
    pub on_link: Callback<Option<Scope<CrudCreateView<T>>>>,
    pub children: ChildrenRenderer<Item>,
    pub data_provider: CrudRestDataProvider<T>,
    pub parent_id: Option<u32>,
    pub config: CrudInstanceConfig<T>,
    pub list_view_available: bool,
    pub on_list_view: Callback<()>,
    // TODO: consolidate these into one "on_entity_creation_attempt" with type Result<CreateResult<T::UpdateModel>, RequestError>?
    pub on_entity_created: Callback<(Saved<T::UpdateModel>, Option<CrudView>)>,
    pub on_entity_not_created_critical_errors: Callback<()>,
    pub on_entity_creation_failed: Callback<RequestError>,
    pub on_tab_selected: Callback<Label>,
}

/// The create view shows the form with which the user can CREATE a new entity of the given resource.
/// NOTE: The instance configuration allows to specify the fields shown when updating the entity (required)
/// as well as specifying the fields shown when creating the entity (optional).
/// If the model for creating and updating an entity is the same, the user may only specify the required fields for updating.
/// These fields are then also used for creation, requiring this component to be able to work with the create and the update model!
/// This component decides on its own, depending on the instance configuration, which fields to display.
pub struct CrudCreateView<T: CrudMainTrait> {
    mode: Mode<T>,
    ongoing_save: bool,
}

enum Mode<T: CrudMainTrait> {
    UseCreateModel {
        initial_data: Option<T::CreateModel>,
        input: T::CreateModel,
    },
    UseUpdateModel {
        initial_data: Option<T::UpdateModel>,
        input: T::UpdateModel,
    },
}

impl<T: 'static + CrudMainTrait> CrudCreateView<T> {
    fn create_entity(&mut self, ctx: &Context<Self>, then: Then) {
        match &self.mode {
            Mode::UseCreateModel {
                initial_data: _,
                input: create_model_input,
            } => {
                let create_model = create_model_input.clone();
                let data_provider = ctx.props().data_provider.clone();
                self.ongoing_save = true;
                ctx.link().send_future(async move {
                    Msg::CreatedEntity(
                        data_provider
                            .create_one_from_create_model(CreateOne {
                                entity: create_model,
                            })
                            .await,
                        then,
                    )
                });
            }
            Mode::UseUpdateModel {
                initial_data: _,
                input: update_model_input,
            } => {
                let update_model = update_model_input.clone();
                let data_provider = ctx.props().data_provider.clone();
                self.ongoing_save = true;
                ctx.link().send_future(async move {
                    Msg::CreatedEntity(
                        data_provider
                            .create_one_from_update_model(CreateOne {
                                entity: update_model,
                            })
                            .await,
                        then,
                    )
                });
            }
        }
    }

    fn reset(&mut self, ctx: &Context<Self>) {
        match &mut self.mode {
            Mode::UseCreateModel {
                initial_data: _,
                input: create_model_input,
            } => *create_model_input = default_create_model(ctx),
            Mode::UseUpdateModel {
                initial_data: _,
                input: update_model_input,
            } => *update_model_input = default_update_model(ctx),
        };
    }
}

fn default_create_model<T: CrudMainTrait + 'static>(
    ctx: &Context<CrudCreateView<T>>,
) -> T::CreateModel {
    let mut entity: T::CreateModel = Default::default();
    if let Some(nested) = &ctx.props().config.nested {
        if let Some(parent_id) = ctx.props().parent_id {
            T::CreateModel::get_field(nested.reference_field.as_str())
                .set_value(&mut entity, Value::U32(parent_id));
            log::info!("successfully set parent id to reference field");
        } else {
            log::error!("CrudInstance is configured to be a nested instance but no parent id was passed down!");
        }
    }
    entity
}

fn default_update_model<T: CrudMainTrait + 'static>(
    ctx: &Context<CrudCreateView<T>>,
) -> T::UpdateModel {
    let mut entity: T::UpdateModel = Default::default();
    if let Some(nested) = &ctx.props().config.nested {
        if let Some(parent_id) = ctx.props().parent_id {
            T::UpdateModel::get_field(nested.reference_field.as_str())
                .set_value(&mut entity, Value::U32(parent_id));
            log::info!("successfully set parent id to reference field");
        } else {
            log::error!("CrudInstance is configured to be a nested instance but no parent id was passed down!");
        }
    }
    entity
}

impl<T: 'static + CrudMainTrait> Component for CrudCreateView<T> {
    type Message = Msg<T>;
    type Properties = Props<T>;

    fn create(ctx: &Context<Self>) -> Self {
        ctx.props().on_link.emit(Some(ctx.link().clone()));

        let mode = match &ctx.props().config.create_elements {
            CreateElements::Default => {
                let update_model = default_update_model(ctx);
                Mode::UseUpdateModel {
                    initial_data: Some(update_model.clone()),
                    input: update_model,
                }
            }
            CreateElements::Custom(_create_elements) => {
                let create_model = default_create_model(ctx);
                Mode::UseCreateModel {
                    initial_data: Some(create_model.clone()),
                    input: create_model,
                }
            }
        };

        Self {
            mode,
            ongoing_save: false,
        }
    }

    fn destroy(&mut self, ctx: &Context<Self>) {
        ctx.props().on_link.emit(None);
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
            Msg::TabSelected(label) => {
                ctx.props().on_tab_selected.emit(label);
                false
            }
            Msg::CreateModelFieldChanged((field, value)) => match &mut self.mode {
                Mode::UseCreateModel {
                    initial_data: _,
                    input,
                } => {
                    field.set_value(input, value);
                    false
                }
                Mode::UseUpdateModel {
                    initial_data: _,
                    input: _,
                } => {
                    panic!("Cannot process CrudCreateView::Msg::CreateModelFieldChanged, as we are using Mode::UseUpdateModel! Sending this message is not allowed.");
                }
            },
            Msg::UpdateModelFieldChanged((field, value)) => match &mut self.mode {
                Mode::UseCreateModel {
                    initial_data: _,
                    input: _,
                } => {
                    panic!("Cannot process CrudCreateView::Msg::UpdateModelFieldChanged, as we are using Mode::UseCreateModel! Sending this message is not allowed.");
                }
                Mode::UseUpdateModel {
                    initial_data: _,
                    input,
                } => {
                    field.set_value(input, value);
                    false
                }
            },
            Msg::CreatedEntity(result, then) => {
                self.ongoing_save = false;
                match result {
                    Ok(create_result) => match create_result {
                        SaveResult::Saved(created) => match then {
                            Then::DoNothing => {
                                ctx.props().on_entity_created.emit((created, None));
                            }
                            Then::OpenListView => {
                                ctx.props()
                                    .on_entity_created
                                    .emit((created, Some(CrudView::List)));
                            }
                            Then::Reset => {
                                ctx.props()
                                    .on_entity_created
                                    .emit((created, Some(CrudView::Create)));
                                self.reset(ctx);
                            }
                        },
                        SaveResult::CriticalValidationErrors => {
                            log::info!("Entity was not created due to critical validation errors.");
                            ctx.props().on_entity_not_created_critical_errors.emit(());
                        }
                    },
                    Err(reason) => {
                        log::error!("Entity creation failed: {:?}", reason);
                        ctx.props().on_entity_creation_failed.emit(reason.clone());
                    }
                }
                false
            }
            Msg::GetCreateModelFieldValue((field, receiver)) => match &self.mode {
                Mode::UseCreateModel {
                    initial_data: _,
                    input: create_model_input,
                } => {
                    receiver(field.get_value(&create_model_input));
                    false
                }
                Mode::UseUpdateModel {
                    initial_data: _,
                    input: _,
                } => {
                    panic!("Cannot process CrudCreateView::Msg::GetCreateModelFieldValue, as we are using Mode::UseUpdateModel! Sending this message is not allowed.");
                }
            },
            Msg::GetUpdateModelFieldValue((field, receiver)) => match &self.mode {
                Mode::UseCreateModel {
                    initial_data: _,
                    input: _,
                } => {
                    panic!("Cannot process CrudCreateView::Msg::GetUpdateModelFieldValue, as we are using Mode::UseCreateModel! Sending this message is not allowed.");
                }
                Mode::UseUpdateModel {
                    initial_data: _,
                    input: update_model_input,
                } => {
                    receiver(field.get_value(&update_model_input));
                    false
                }
            },
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
            {
                match &ctx.props().config.create_elements {
                    CreateElements::Default => html! {
                        <CrudFields<T::UpdateModel>
                            api_base_url={ctx.props().config.api_base_url.clone()}
                            children={ChildrenRenderer::new(ctx.props().children.iter().filter(|it| match it {
                                Item::NestedInstance(_) => true,
                                Item::Relation(_) => true,
                                Item::Select(select) => select.props.for_model == crate::crud_reset_field::Model::Update,
                            }).collect::<Vec<Item>>())}
                            elements={ctx.props().config.elements.clone()}
                            entity={match &self.mode {
                                Mode::UseCreateModel { initial_data: _, input: _ } => {
                                    panic!("forbidden path")
                                },
                                Mode::UseUpdateModel { initial_data, input: _ } => {
                                    initial_data.clone()
                                },
                            }}
                            mode={FieldMode::Editable}
                            current_view={CrudView::Create}
                            value_changed={ctx.link().callback(Msg::UpdateModelFieldChanged)}
                            active_tab={ctx.props().config.active_tab.clone()}
                            on_tab_selection={ctx.link().callback(|label| Msg::TabSelected(label))}
                        />
                    },
                    CreateElements::Custom(create_elements) => html! {
                        <CrudFields<T::CreateModel>
                            api_base_url={ctx.props().config.api_base_url.clone()}
                            children={ChildrenRenderer::new(ctx.props().children.iter().filter(|it| match it {
                                Item::NestedInstance(_) => true,
                                Item::Relation(_) => true,
                                Item::Select(select) => select.props.for_model == crate::crud_reset_field::Model::Create,
                            }).collect::<Vec<Item>>())}
                            elements={create_elements.clone()}
                            entity={match &self.mode {
                                Mode::UseCreateModel { initial_data, input: _ } => {
                                    initial_data.clone()
                                },
                                Mode::UseUpdateModel { initial_data: _, input: _ } => {
                                    panic!("forbidden path")
                                },
                            }}
                            mode={FieldMode::Editable}
                            current_view={CrudView::Create}
                            value_changed={ctx.link().callback(Msg::CreateModelFieldChanged)}
                            active_tab={ctx.props().config.active_tab.clone()}
                            on_tab_selection={ctx.link().callback(|label| Msg::TabSelected(label))}
                        />
                    },
                }
            }
            </>
        }
    }

    fn rendered(&mut self, _ctx: &Context<Self>, first_render: bool) {
        if first_render {
            // The initial_*_data fields are used to declare the initial field states.
            // But: We only want to pass down that data once!
            // After we did that, no rerender of this component must overwrite the users input data.
            // Leaving it to Some(*) would erase the user input on every render, as this data is passed as the 'entity' to the CrudFields component.
            match &mut self.mode {
                Mode::UseCreateModel {
                    initial_data: initial_create_model_data,
                    input: _,
                } => {
                    *initial_create_model_data = None;
                }
                Mode::UseUpdateModel {
                    initial_data: initial_update_model_data,
                    input: _,
                } => {
                    *initial_update_model_data = None;
                }
            };
        }
    }
}
