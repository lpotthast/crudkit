use std::collections::HashMap;

use crudkit_condition::IntoAllEqualCondition;
use crudkit_id::Id;
use crudkit_id::IdField;
use crudkit_shared::{SaveResult, Saved};

use gloo::timers::callback::Interval;
use tracing::{info, warn};
use yew::{
    html::{ChildrenRenderer, Scope},
    prelude::*,
};

use crate::crud_action::EntityModalGeneration;
use crate::crud_instance::Item;
use crate::prelude::*;

const MILLIS_UNTIL_ERROR_IS_SHOWN: u32 = 1000;

// TODO: CrudEditView tracks changes, but CrudCreateView does not. Consolidate this logic into a shared component.

pub enum Msg<T: CrudMainTrait> {
    Back,
    BackCanceled,
    BackApproved,
    LoadedEntity(Result<Option<T::ReadModel>, RequestError>),
    UpdatedEntity((Result<SaveResult<T::UpdateModel>, RequestError>, Then)),
    ShowError,
    Save,
    SaveAndReturn,
    SaveAndNew,
    Delete,
    TabSelected(Label),
    ValueChanged(
        (
            <T::UpdateModel as CrudDataTrait>::Field,
            Result<Value, String>,
        ),
    ),
    GetInput(
        (
            <T::UpdateModel as CrudDataTrait>::Field,
            Box<dyn FnOnce(Value)>,
        ),
    ),
    ActionInitialized {
        action_id: &'static str,
    },
    ActionCancelled {
        action_id: &'static str,
    },
    ActionTriggered {
        action_id: &'static str,
        action_payload: Option<T::ActionPayload>,
        action: Callback<(
            T::UpdateModel,
            Option<T::ActionPayload>,
            Callback<Result<CrudActionAftermath, CrudActionAftermath>>,
        )>,
    },
    ActionExecuted {
        action_id: &'static str,
        result: Result<CrudActionAftermath, CrudActionAftermath>,
    },
    Reload,
}

#[derive(Properties, PartialEq)]
pub struct Props<T: 'static + CrudMainTrait> {
    pub on_link: Callback<Option<Scope<CrudEditView<T>>>>,
    pub children: ChildrenRenderer<Item>,
    pub custom_fields: CustomUpdateFields<T, yew::Html>,
    pub data_provider: CrudRestDataProvider<T>,
    pub config: CrudInstanceConfig<T>,
    pub static_config: CrudStaticInstanceConfig<T>,
    /// The ID of the entity being edited.
    pub id: T::UpdateModelId,
    pub list_view_available: bool,
    pub on_entity_updated: Callback<Saved<T::UpdateModel>>,
    pub on_entity_update_aborted: Callback<String>,
    pub on_entity_not_updated_critical_errors: Callback<()>,
    pub on_entity_update_failed: Callback<RequestError>,
    pub on_list: Callback<()>,
    pub on_create: Callback<()>,
    pub on_delete: Callback<T::UpdateModel>,
    pub on_tab_selected: Callback<Label>,
    pub on_entity_action: Callback<CrudActionAftermath>,
}

pub struct CrudEditView<T: CrudMainTrait> {
    /// The input is `None`, if the entity was not yet loaded.
    /// We cannot and should not use a `Default` value,
    /// as an editable entity was already created and might therefore contain field for which no default is available.
    input: Option<T::UpdateModel>,

    /// The input is 'dirty' if was changed: The current state of `input` is not the state we started with.
    input_dirty: bool,

    /// The input is erroneous if at least one field is contained in this list.
    input_errors: HashMap<<T::UpdateModel as CrudDataTrait>::Field, String>,

    user_wants_to_activate: Vec<String>,

    user_wants_to_leave: bool,

    /// Stores the current state of the entity or an error, if no entity could be fetched.
    /// Note that, while the initial fetch request is ongoing, this is in the error state!
    ///
    /// Holds an error displayed to the user.
    /// This variable is updated after an error was stored in the `entity` field and stood there for a longer time period.
    /// This allows us to not immediately show all errors to the user.
    ///
    // We might want to store ReadModel as entity_read here, and entity_orig as an updatable version of it...
    entity: Result<T::UpdateModel, (NoData, bool, Option<Interval>)>,

    ongoing_save: bool,
    actions_executing: Vec<&'static str>,
}

enum SetFrom {
    Fetch,
    Update,
}

pub enum Then {
    DoNothing,
    OpenListView,
    OpenCreateView,
}

impl<T: 'static + CrudMainTrait> CrudEditView<T> {
    // TODO: Remove this code duplication!

    fn is_save_disabled(&self) -> bool {
        self.ongoing_save || !self.input_errors.is_empty()
    }

    fn is_delete_disabled(&self) -> bool {
        self.ongoing_save
    }

    fn load_entity(ctx: &Context<Self>) {
        let id = ctx.props().id.clone();
        let data_provider = ctx.props().data_provider.clone();
        ctx.link().send_future(async move {
            Msg::LoadedEntity(load_entity::<T>(data_provider, &id).await)
        });
    }

    fn create_error_clock(ctx: &Context<Self>) -> Interval {
        let clock_handle = {
            let link = ctx.link().clone();
            Interval::new(MILLIS_UNTIL_ERROR_IS_SHOWN, move || {
                link.send_message(Msg::ShowError)
            })
        };
        clock_handle
    }

    fn _set_entity(
        &mut self,
        entity: Result<T::UpdateModel, (NoData, bool, Option<Interval>)>,
        ctx: &Context<Self>,
    ) {
        self.entity = entity;
        match &mut self.entity {
            Ok(_entity) => {}
            Err((_reason, shown, clock)) => match (shown, clock.is_some()) {
                (true, true) => *clock = None,
                (true, false) => {}
                (false, true) => {}
                (false, false) => *clock = Some(Self::create_error_clock(ctx)),
            },
        }
    }

    /// Updates the entity field.
    fn set_entity_from_fetch_result(
        &mut self,
        data: Result<Option<T::ReadModel>, RequestError>,
        from: SetFrom,
        ctx: &Context<Self>,
    ) {
        self._set_entity(
            match data {
                Ok(data) => match data {
                    Some(entity) => Ok(entity.into()),
                    None => Err(match from {
                        SetFrom::Fetch => (NoData::FetchReturnedNothing, true, None),
                        SetFrom::Update => (NoData::UpdateReturnedNothing, true, None),
                    }),
                },
                Err(err) => Err(match from {
                    SetFrom::Fetch => (NoData::FetchFailed(err), true, None),
                    SetFrom::Update => (NoData::UpdateFailed(err), true, None),
                }),
            },
            ctx,
        );
        if let Ok(entity) = &self.entity {
            self.input = Some(entity.clone());
            self.input_dirty = false;
        }
    }

    fn set_entity_from_save_result(
        &mut self,
        data: Result<SaveResult<T::UpdateModel>, RequestError>,
        from: SetFrom,
        ctx: &Context<Self>,
    ) {
        match data {
            Ok(save_result) => match save_result {
                SaveResult::Saved(saved) => {
                    self.input = Some(saved.entity.clone());
                    self.input_dirty = false;
                    self._set_entity(Ok(saved.entity), ctx);
                }
                SaveResult::Aborted { reason: _ } => {
                    // Do nothing...
                }
                SaveResult::CriticalValidationErrors => {
                    // TODO: Do something with the critical errors?
                    // Keep current entity!
                }
            },
            Err(err) => {
                self._set_entity(
                    Err(match from {
                        SetFrom::Fetch => (NoData::FetchFailed(err), true, None),
                        SetFrom::Update => (NoData::UpdateFailed(err), true, None),
                    }),
                    ctx,
                );
            }
        };
    }

    fn save_entity(&self, ctx: &Context<Self>, and_then: Then) {
        let entity = self.input.clone().expect("Entity to be already loaded");
        let condition = <T as CrudMainTrait>::UpdateModelId::fields_iter(&ctx.props().id)
            .map(|field| (field.name().to_owned(), field.to_value()))
            .into_all_equal_condition();
        let data_provider = ctx.props().data_provider.clone();
        // TODO: Like in create_view, store ongoing_save!!
        ctx.link().send_future(async move {
            Msg::UpdatedEntity((
                data_provider
                    .update_one(UpdateOne {
                        entity,
                        condition: Some(condition),
                    })
                    .await,
                and_then,
            ))
        });
    }
}

impl<T: 'static + CrudMainTrait> Component for CrudEditView<T> {
    type Message = Msg<T>;
    type Properties = Props<T>;

    fn create(ctx: &Context<Self>) -> Self {
        ctx.props().on_link.emit(Some(ctx.link().clone()));
        CrudEditView::load_entity(ctx);
        Self {
            input: None,
            input_dirty: false,
            input_errors: HashMap::new(),
            user_wants_to_activate: vec![],
            user_wants_to_leave: false,
            entity: Err((
                NoData::NotYetLoaded,
                false,
                Some(CrudEditView::create_error_clock(ctx)),
            )),
            ongoing_save: false,
            actions_executing: vec![],
        }
    }

    fn destroy(&mut self, ctx: &Context<Self>) {
        ctx.props().on_link.emit(None);
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
                self.set_entity_from_fetch_result(data, SetFrom::Fetch, ctx);
                true
            }
            Msg::UpdatedEntity((data, and_then)) => {
                self.set_entity_from_save_result(data.clone(), SetFrom::Update, ctx);

                match data {
                    Ok(save_result) => match save_result {
                        SaveResult::Saved(saved) => {
                            ctx.props().on_entity_updated.emit(saved);
                            match and_then {
                                Then::DoNothing => {}
                                Then::OpenListView => ctx.props().on_list.emit(()),
                                Then::OpenCreateView => ctx.props().on_create.emit(()),
                            }
                        }
                        SaveResult::Aborted { reason } => {
                            ctx.props().on_entity_update_aborted.emit(reason);
                        }
                        SaveResult::CriticalValidationErrors => {
                            info!("Entity was not updated due to critical validation errors.");
                            ctx.props().on_entity_not_updated_critical_errors.emit(());
                        }
                    },
                    Err(err) => {
                        warn!(
                            "Could not update entity due to RequestError: {}",
                            err.to_string()
                        );
                        ctx.props().on_entity_update_failed.emit(err);
                    }
                }
                true
            }
            Msg::ShowError => match &mut self.entity {
                Ok(_data) => false,
                Err((_reason, show, clock)) => {
                    *show = true;
                    *clock = None;
                    true
                }
            },
            Msg::Save => {
                self.save_entity(ctx, Then::DoNothing);
                true
            }
            Msg::SaveAndReturn => {
                self.save_entity(ctx, Then::OpenListView);
                false
            }
            Msg::SaveAndNew => {
                self.save_entity(ctx, Then::OpenCreateView);
                false
            }
            Msg::Delete => {
                match &self.entity {
                    Ok(entity) => ctx.props().on_delete.emit(entity.clone()),
                    Err(_) => {
                        warn!("Cannot issue a delete event, as not entity is currently loaded!")
                    }
                }
                false
            }
            Msg::TabSelected(label) => {
                ctx.props().on_tab_selected.emit(label);
                false
            }
            Msg::ValueChanged((field, result)) => {
                match result {
                    Ok(value) => {
                        let input = self.input.as_mut().expect("Entity to be already loaded");
                        field.set_value(input, value);
                        self.input_errors.remove(&field);
                        // We might only want to set this to true if the new value was actually different to the old value!
                        match &self.entity {
                            Ok(entity) => self.input_dirty = input != entity,
                            Err(_) => self.input_dirty = false,
                        }
                        false
                    }
                    Err(err) => {
                        self.input_errors.insert(field, err);
                        // When we want to render the errors, set this to true.
                        false
                    }
                }
            }
            Msg::GetInput((field, receiver)) => {
                receiver(
                    field.get_value(self.input.as_ref().expect("Entity to be already loaded")),
                );
                false
            }
            Msg::ActionInitialized { action_id } => {
                self.user_wants_to_activate.push(action_id.to_string());
                true
            }
            Msg::ActionCancelled { action_id } => {
                if let Some(index) = self
                    .user_wants_to_activate
                    .iter()
                    .position(|it| it.as_str() == action_id)
                {
                    self.user_wants_to_activate.remove(index);
                    true
                } else {
                    false
                }
            }
            Msg::ActionTriggered {
                action_id,
                action_payload,
                action,
            } => {
                if let Some(index) = self
                    .user_wants_to_activate
                    .iter()
                    .position(|it| it.as_str() == action_id)
                {
                    self.user_wants_to_activate.remove(index);
                }
                action.emit((
                    self.input.clone().expect("Entity to be already loaded"),
                    action_payload,
                    ctx.link()
                        .callback(move |result| Msg::ActionExecuted { action_id, result }),
                ));
                if !self.actions_executing.contains(&action_id) {
                    self.actions_executing.push(action_id);
                    true
                } else {
                    false
                }
            }
            Msg::ActionExecuted { action_id, result } => {
                // We currently handle both the success and the error path in the same way. This might need to be changes in the future.
                // But the user should always state in which path we are!
                match result {
                    Ok(aftermath) => ctx.props().on_entity_action.emit(aftermath),
                    Err(aftermath) => ctx.props().on_entity_action.emit(aftermath),
                }
                if let Some(index) = self
                    .actions_executing
                    .iter()
                    .position(|it| *it == action_id)
                {
                    self.actions_executing.remove(index);
                    true
                } else {
                    false
                }
            }
            Msg::Reload => {
                CrudEditView::load_entity(ctx);
                // load_entity triggers an async operation. Handler will re-render!
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
                                            <CrudBtn name={"Speichern"} variant={Variant::Primary} disabled={self.is_save_disabled()} onclick={ctx.link().callback(|_| Msg::Save)}>
                                                <CrudBtn name={"Speichern und zurück"} variant={Variant::Primary} disabled={self.is_save_disabled()} onclick={ctx.link().callback(|_| Msg::SaveAndReturn)} />
                                                <CrudBtn name={"Speichern und neu"} variant={Variant::Primary} disabled={self.is_save_disabled()} onclick={ctx.link().callback(|_| Msg::SaveAndNew)} />
                                            </CrudBtn>
                                            <CrudBtn name={"Löschen"} variant={Variant::Danger} disabled={self.is_delete_disabled()} onclick={ctx.link().callback(|_| Msg::Delete)} />

                                            {
                                                ctx.props().static_config.entity_actions.iter()
                                                    .filter_map(|action| match action {
                                                        CrudEntityAction::Custom {id, name, icon, variant, valid_in, action, modal} => {
                                                            valid_in.contains(&States::Update).then(|| {
                                                                let action_id: &str = (&id).clone();
                                                                let action = action.clone();

                                                                if let Some(modal) = modal {
                                                                    html! {
                                                                        <>
                                                                        <CrudBtn
                                                                            name={name.clone()}
                                                                            variant={variant.clone()}
                                                                            icon={icon.clone()}
                                                                            disabled={self.actions_executing.contains(&id) || self.input.is_none()}
                                                                            onclick={ctx.link().callback(move |_| Msg::ActionInitialized { action_id }) }
                                                                        />
                                                                        if self.user_wants_to_activate.iter().any(|it| it.as_str() == action_id) {
                                                                            <CrudModal>
                                                                                {{ modal(EntityModalGeneration {
                                                                                    state: self.input.clone().expect("Entity to be already loaded"),
                                                                                    cancel: ctx.link().callback(move |_| Msg::ActionCancelled { action_id }),
                                                                                    execute: ctx.link().callback(move |action_payload| Msg::ActionTriggered { action_id, action_payload, action: action.clone() }),
                                                                                }) }}
                                                                            </CrudModal>
                                                                        }
                                                                        </>
                                                                    }
                                                                } else {
                                                                    html! {
                                                                        <CrudBtn
                                                                            name={name.clone()}
                                                                            variant={variant.clone()}
                                                                            icon={icon.clone()}
                                                                            disabled={self.actions_executing.contains(&id)}
                                                                            onclick={ctx.link().callback(move |_| Msg::ActionTriggered { action_id, action_payload: None, action: action.clone() }) }
                                                                        />
                                                                    }
                                                                }
                                                            })
                                                        }
                                                    })
                                                    .collect::<Html>()
                                            }
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
                                    children={ChildrenRenderer::new(ctx.props().children.iter().filter(|it| match it {
                                        Item::NestedInstance(_) => true,
                                        Item::Relation(_) => true,
                                        Item::Select(select) => select.props.for_model == crate::crud_reset_field::Model::Update,
                                    }).collect::<Vec<Item>>())}
                                    custom_fields={ctx.props().custom_fields.clone()}
                                    elements={ctx.props().config.elements.clone()}
                                    entity={self.input.clone()}
                                    mode={FieldMode::Editable}
                                    current_view={CrudSimpleView::Edit}
                                    value_changed={ctx.link().callback(Msg::ValueChanged)}
                                    active_tab={ctx.props().config.active_tab.clone()}
                                    on_tab_selection={ctx.link().callback(|label| Msg::TabSelected(label))}
                                />
                                </>
                            }
                        }
                        Err((reason, show, _)) => {
                            html! {
                                <>
                                <div class={"crud-row crud-nav"}>
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
                                if *show {
                                    <div>
                                        {format!("Daten nicht verfügbar: {:?}", reason)}
                                    </div>
                                }
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

pub async fn load_entity<T: CrudMainTrait>(
    data_provider: CrudRestDataProvider<T>,
    id: &T::UpdateModelId,
) -> Result<Option<T::ReadModel>, RequestError> {
    let condition = <T as CrudMainTrait>::UpdateModelId::fields_iter(id)
        .map(|field| (field.name().to_owned(), field.to_value()))
        .into_all_equal_condition();
    data_provider
        .read_one(ReadOne {
            skip: None,
            order_by: None,
            condition: Some(condition),
        })
        .await
}
