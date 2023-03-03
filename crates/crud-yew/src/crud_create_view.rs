use std::collections::HashMap;

use tracing::{error, info};
use yew::{
    html::{ChildrenRenderer, Scope},
    prelude::*,
};

use crud_id::SerializableId;
use crud_shared::{SaveResult, Saved};

use crate::{
    crud_instance::Item,
    services::crud_rest_data_provider::{CreateOne, CrudRestDataProvider},
    types::custom_field::{CustomCreateFields, CustomUpdateFields},
};

use super::{prelude::*, types::RequestError};

pub enum Msg<T: CrudMainTrait> {
    Back,
    Save,
    SaveAndReturn,
    SaveAndNew,
    TabSelected(Label),

    /// This message must only be sent if the component is actually using the CreateModel, the program will otherwise panic!
    CreateModelFieldChanged(
        (
            <T::CreateModel as CrudDataTrait>::Field,
            Result<Value, String>,
        ),
    ),

    // After saving an entity, the CRUD system always return the UpdateModel!
    CreatedEntity(Result<SaveResult<T::UpdateModel>, RequestError>, Then),

    /// This message must only be sent if the component is actually using the CreateModel, the program will otherwise panic!
    GetCreateModelFieldValue(
        (
            <T::CreateModel as CrudDataTrait>::Field,
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
    pub custom_create_fields: CustomCreateFields<T>,
    pub custom_update_fields: CustomUpdateFields<T>,
    pub data_provider: CrudRestDataProvider<T>,
    /// Required because when creating the initial CreateModel, we have to set the "parent id" field of that model to the given id.
    /// TODO: Only a subset of the parent id might be required to for matching. Consider a CreateModel#initialize_with_parent_id(ParentId)...
    pub parent_id: Option<SerializableId>,
    pub config: CrudInstanceConfig<T>,
    pub list_view_available: bool,
    pub on_list_view: Callback<()>,
    // TODO: consolidate these into one "on_entity_creation_attempt" with type Result<CreateResult<T::UpdateModel>, SomeErrorType>?
    pub on_entity_created: Callback<(
        Saved<T::UpdateModel>,
        Option<CrudView<T::ReadModelId, T::UpdateModelId>>,
    )>,
    pub on_entity_creation_aborted: Callback<String>,
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
    initial_data: Option<T::CreateModel>,
    input: T::CreateModel,
    ongoing_save: bool,

    // TODO: input_dirty like in EditView? Why not here?
    /// The input is erroneous if at least one field is contained in this list.
    create_input_errors: HashMap<<T::CreateModel as CrudDataTrait>::Field, String>,
}

impl<T: 'static + CrudMainTrait> CrudCreateView<T> {
    fn create_entity(&mut self, ctx: &Context<Self>, then: Then) {
        let create_model = self.input.clone();
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

    fn reset(&mut self, ctx: &Context<Self>) {
        self.input = default_create_model(ctx);
    }
}

fn default_create_model<T: CrudMainTrait + 'static>(
    ctx: &Context<CrudCreateView<T>>,
) -> T::CreateModel {
    let mut entity: T::CreateModel = Default::default();
    if let Some(nested) = &ctx.props().config.nested {
        if let Some(parent_id) = &ctx.props().parent_id {
            let (_field_name, value) = parent_id
                .0
                .iter()
                .find(|(field_name, _value)| field_name == nested.parent_field.as_str())
                .expect("related parent field must be part of the parents id!");

            T::CreateModel::get_field(nested.reference_field.as_str())
                .set_value(&mut entity, value.clone().into());
            info!("successfully set parent id to reference field");
        } else {
            error!("CrudInstance is configured to be a nested instance but no parent id was passed down!");
        }
    }
    entity
}

impl<T: 'static + CrudMainTrait> Component for CrudCreateView<T> {
    type Message = Msg<T>;
    type Properties = Props<T>;

    fn create(ctx: &Context<Self>) -> Self {
        ctx.props().on_link.emit(Some(ctx.link().clone()));
        let create_model = default_create_model(ctx);
        Self {
            initial_data: Some(create_model.clone()),
            input: create_model,
            ongoing_save: false,
            create_input_errors: HashMap::new(),
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
            Msg::CreateModelFieldChanged((field, result)) => match result {
                Ok(value) => {
                    field.set_value(&mut self.input, value);
                    self.create_input_errors.remove(&field);
                    false
                }
                Err(err) => {
                    self.create_input_errors.insert(field, err);
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
                        SaveResult::Aborted { reason } => {
                            ctx.props().on_entity_creation_aborted.emit(reason);
                        }
                        SaveResult::CriticalValidationErrors => {
                            info!("Entity was not created due to critical validation errors.");
                            ctx.props().on_entity_not_created_critical_errors.emit(());
                        }
                    },
                    Err(reason) => {
                        error!("Entity creation failed: {:?}", reason);
                        ctx.props().on_entity_creation_failed.emit(reason.clone());
                    }
                }
                false
            }
            Msg::GetCreateModelFieldValue((field, receiver)) => {
                receiver(field.get_value(&self.input));
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
            {
                match &ctx.props().config.create_elements {
                    CreateElements::None => html! {},
                    CreateElements::Custom(create_elements) => html! {
                        <CrudFields<T::CreateModel>
                            api_base_url={ctx.props().config.api_base_url.clone()}
                            children={ChildrenRenderer::new(ctx.props().children.iter().filter(|it| match it {
                                Item::NestedInstance(_) => true,
                                Item::Relation(_) => true,
                                Item::Select(select) => select.props.for_model == crate::crud_reset_field::Model::Create,
                            }).collect::<Vec<Item>>())}
                            custom_fields={ctx.props().custom_create_fields.clone()}
                            elements={create_elements.clone()}
                            entity={self.initial_data.clone()}
                            mode={FieldMode::Editable}
                            current_view={CrudSimpleView::Create}
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
            // The initial_data field is used to declare the initial field states.
            // But: We only want to pass down that data once!
            // After we did that, no rerender of this component must overwrite the users input data.
            // Leaving it to Some(*) would erase the user input on the second render, as this data is passed as the 'entity' to the CrudFields component.
            self.initial_data = None;
        }
    }
}
