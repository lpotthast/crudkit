use crud_shared_types::{validation::SerializableValidations, SaveResult, Saved};
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
    ValueChanged((<T::UpdateModel as CrudDataTrait>::Field, Value)),
    CreatedEntity(Result<SaveResult<T::UpdateModel>, RequestError>, Then),
    GetInput(
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
    pub on_entity_not_created_critical_errors: Callback<SerializableValidations>,
    pub on_entity_creation_failed: Callback<RequestError>,
}

pub struct CrudCreateView<T: CrudMainTrait> {
    initial_data: Option<T::UpdateModel>,
    input: T::UpdateModel,
    ongoing_save: bool,
}

impl<T: 'static + CrudMainTrait> CrudCreateView<T> {
    fn create_entity(&mut self, ctx: &Context<Self>, then: Then) {
        let ent = self.input.clone();
        let data_provider = ctx.props().data_provider.clone();
        self.ongoing_save = true;
        ctx.link().send_future(async move {
            Msg::CreatedEntity(
                data_provider.create_one(CreateOne { entity: ent }).await,
                then,
            )
        });
    }

    fn reset(&mut self) {
        self.input = Default::default();
    }
}

impl<T: 'static + CrudMainTrait> Component for CrudCreateView<T> {
    type Message = Msg<T>;
    type Properties = Props<T>;

    fn create(ctx: &Context<Self>) -> Self {
        ctx.props().on_link.emit(Some(ctx.link().clone()));
        let mut entity: T::UpdateModel = Default::default();
        if let Some(nested) = &ctx.props().config.nested {
            if let Some(parent_id) = ctx.props().parent_id {
                T::UpdateModel::get_field(nested.reference_field.as_str())
                    .set_value(&mut entity, Value::U32(parent_id));
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
            Msg::ValueChanged((field, value)) => {
                field.set_value(&mut self.input, value);
                false
            }
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
                                self.reset();
                            }
                        },
                        SaveResult::CriticalValidationErrors(serializable_validations) => {
                            // TODO: Store validations in store!
                            log::error!(
                                "Entity was not created due to critical errors {:?}",
                                serializable_validations
                            );
                            ctx.props()
                                .on_entity_not_created_critical_errors
                                .emit(serializable_validations);
                        }
                    },
                    Err(reason) => {
                        log::error!("Entity creation failed: {:?}", reason);
                        ctx.props().on_entity_creation_failed.emit(reason.clone());
                    }
                }
                false
            }
            Msg::GetInput((field, receiver)) => {
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

            <CrudFields<T::UpdateModel>
                api_base_url={ctx.props().config.api_base_url.clone()}
                children={ctx.props().children.clone()}
                elements={ctx.props().config.elements.clone()}
                entity={self.initial_data.clone()}
                mode={FieldMode::Editable}
                current_view={CrudView::Create}
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
