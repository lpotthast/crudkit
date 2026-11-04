//! Create, edit, and read forms.

use crate::config::{CrudEntityViewKind, CrudUiTexts};
use crate::hooks::field::CrudFieldMap;
use crate::hooks::field::{ReactiveField, reactive_fields};
use crate::hooks::instance::use_crud_instance;
use crate::hooks::notify::{CrudNotification, CrudNotifier};
use crate::hooks::texts::{use_crud_texts, with_texts};
use crate::instance::CrudInstanceContext;
use crate::instance::CrudNavigation;
use crudkit_core::Value;
use crudkit_core::id::SerializableId;
use crudkit_core::validation::PartialSerializableAggregateViolations;
use crudkit_web::data_source::{
    CrudCompletion, CrudCreateRequest, CrudFieldProblem, CrudReadRequest, CrudSaved,
    CrudUpdateRequest,
};
use crudkit_web::http::RequestError;
use crudkit_web::load_state::LoadState;
use crudkit_web::prelude::*;
use crudkit_web::view::CrudView;
use leptos::prelude::*;
use std::any::TypeId;
use std::collections::{HashMap, HashSet};
use std::fmt;

/// What happens after an entity was saved successfully.
#[derive(Debug, Clone, Copy)]
pub enum CrudSaveFollowUp {
    /// Keep the current view. An edit form goes on editing the saved entity, a create form starts
    /// a fresh draft.
    Stay,
    /// Return from the current view, see [`CrudNavigation::return_from_current`].
    Return,
    /// Open the saved entity in the edit view. After editing, this stays.
    Edit,
    /// Start another entity: a create form starts a fresh draft, an edit form opens a create view.
    CreateAnother,
    /// Open the view returned for the saved entity's id.
    View(Callback<SerializableId, CrudView>),
}

impl CrudSaveFollowUp {
    /// Performs this follow-up on `navigation` after a form of `kind` saved the entity `id`.
    /// `stay` runs for [`Self::Stay`], and where the form already shows what the follow-up asks
    /// for: [`Self::Edit`] in an edit form, [`Self::CreateAnother`] in a create form.
    fn perform(
        self,
        navigation: CrudNavigation,
        id: SerializableId,
        kind: CrudEntityViewKind,
        stay: impl Fn(),
    ) {
        match self {
            Self::Stay => stay(),
            Self::Edit if kind == CrudEntityViewKind::Update => stay(),
            Self::CreateAnother if kind == CrudEntityViewKind::Create => stay(),
            Self::Return => {
                navigation.return_committed();
            }
            Self::Edit => {
                navigation.navigate_committed(CrudView::edit(id));
            }
            Self::CreateAnother => {
                navigation.navigate_committed(CrudView::create());
            }
            Self::View(resolve) => {
                navigation.navigate_committed(resolve.run(id));
            }
        }
    }
}

/// The form surrounding the caller: a [`CrudCreateForm`](crate::atoms::CrudCreateForm),
/// [`CrudEditForm`](crate::atoms::CrudEditForm), or [`CrudDetails`](crate::atoms::CrudDetails).
///
/// Read it with [`use_crud_form`], e.g. to render a form's status, dirty state, or save controls
/// in markup the atoms do not cover.
#[derive(Clone, Copy)]
pub struct CrudFormHandle {
    /// Whether the form creates, edits, or shows an entity.
    pub kind: CrudEntityViewKind,
    /// Whether the form's entity is shown. Always ready in create forms.
    pub status: Signal<CrudEntityLoadStatus>,
    /// Whether the form holds unsaved changes.
    pub is_dirty: Signal<bool>,
    /// Whether some field holds input that could not be read, which keeps the form from saving.
    pub has_errors: Signal<bool>,
    /// Whether a save request is in flight.
    pub is_saving: Signal<bool>,
    /// Whether the form can be saved now.
    pub can_save: Signal<bool>,
    /// Saves the form, then performs the given follow-up. `None` in details, which do not save.
    pub save: Option<Callback<CrudSaveFollowUp>>,
    /// Non-critical validation violations the server reported for the last successful save, if
    /// any. Always `None` in details.
    pub violations: Signal<Option<PartialSerializableAggregateViolations>>,
    /// What saving does when the saving control does not say, e.g. on submit.
    pub follow_up: CrudSaveFollowUp,
    /// The shown entity including unsaved changes, which entity actions act on. `None` in create
    /// forms and while the entity loads.
    pub draft: Signal<Option<DynUpdateModel>>,
    /// Why the last save failed, while it applies, see [`CrudFormState::save_error`]. Always
    /// `None` in details.
    pub save_error: Signal<Option<RequestError>>,
    /// The problems of fields that the last save reported for fields the form does not show. The
    /// form shows them with the failure's message, e.g. in a
    /// [`CrudFormError`](crate::atoms::CrudFormError).
    pub unshown_problems: Signal<Vec<CrudFieldProblem>>,
    /// Reloads the form's entity to show its current state, e.g. after a save was refused as
    /// [outdated](RequestError::Outdated). A form with unsaved changes keeps them. `None` in create
    /// forms.
    pub reload: Option<Callback<()>>,
    /// The `TypeId` of the form's field type, e.g. `DynCreateField`.
    pub(crate) field_type: TypeId,
    /// Whether the form is a `<form>` element, which its save button submits. A form nested in
    /// another one is not.
    pub(crate) submits: bool,
}

impl fmt::Debug for CrudFormHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CrudFormHandle")
            .field("kind", &self.kind)
            .field("follow_up", &self.follow_up)
            .finish_non_exhaustive()
    }
}

/// Returns the surrounding form.
///
/// # Panics
///
/// Panics when called outside of a form, or inside an instance nested in a form.
#[must_use]
pub fn use_crud_form() -> CrudFormHandle {
    try_use_crud_form().expect("`use_crud_form` must be called inside a CrudKit form")
}

/// Returns the surrounding form, unless an instance lies in between.
pub(crate) fn try_use_crud_form() -> Option<CrudFormHandle> {
    use_context::<Option<CrudFormHandle>>().flatten()
}

/// Returns the form surrounding `atom`.
///
/// # Panics
///
/// Panics when `atom` is rendered outside of a form, or inside an instance nested in a form.
pub(crate) fn expect_crud_form(atom: &'static str) -> CrudFormHandle {
    try_use_crud_form().unwrap_or_else(|| {
        panic!("`{atom}` must be rendered inside a CrudKit form, e.g. a `CrudEditForm`")
    })
}

/// Draft state of a form editing one entity, with one [`ReactiveField`] per field.
///
/// The form keeps the complete draft model for saving and dirty checks, and one reactive value per
/// field for fine-grained rendering. [`Self::set_field`] keeps both in sync.
pub struct CrudFormState<F: TypeErasedField> {
    /// The reactive values of all fields. Changes when an entity (re)loads.
    pub fields: Signal<CrudFieldMap<F>>,
    /// The current draft, or `None` while no entity is loaded.
    pub draft: Signal<Option<F::Model>>,
    /// Input errors keyed by field.
    pub errors: Signal<HashMap<F, String>>,
    /// Why the last save failed, while it applies. A successful save and loading another entity
    /// clear it, and so does reloading the entity after a save was refused as
    /// [outdated](RequestError::Outdated).
    pub save_error: Signal<Option<RequestError>>,
    /// The problems of fields that the last save reported ([`RequestError::InvalidFields`]),
    /// without those of fields the user changed since. They do not keep the form from saving.
    pub problems: Signal<Vec<CrudFieldProblem>>,
    field_values: RwSignal<CrudFieldMap<F>>,
    draft_model: RwSignal<Option<F::Model>>,
    baseline: RwSignal<Option<F::Model>>,
    input_errors: RwSignal<HashMap<F, String>>,
    failure: RwSignal<SaveFailure>,
    /// The names of the fields that the form shows, each with the number of its mounted atoms.
    shown_fields: RwSignal<HashMap<String, usize>>,
    /// The fields the user set since the form loaded or saved its entity.
    touched: StoredValue<HashSet<F>>,
    /// Whether the draft differs from the loaded or default entity.
    pub is_dirty: Signal<bool>,
    /// Whether the form holds an entity. Field values exist only while this is `true`.
    pub is_ready: Signal<bool>,
    /// Whether some field holds input that could not be converted into a value. Such a draft is
    /// not saved, because it does not reflect what the user sees.
    pub has_errors: Signal<bool>,
    /// Whether a save request is in flight. Fields are read-only meanwhile, so that the saved
    /// entity, which becomes the form's new state, cannot overwrite input made during the request.
    pub is_saving: Signal<bool>,
}

// Deriving `Clone` and `Copy` would require `F: Clone` and `F: Copy`. The handle is `Copy` for every
// field type, so both are implemented manually.
#[allow(clippy::expl_impl_clone_on_copy)]
impl<F: TypeErasedField> Clone for CrudFormState<F> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<F: TypeErasedField> Copy for CrudFormState<F> {}

impl<F: TypeErasedField> fmt::Debug for CrudFormState<F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CrudFormState").finish_non_exhaustive()
    }
}

impl<F: TypeErasedField> CrudFormState<F> {
    fn new(is_saving: Signal<bool>) -> Self {
        let draft = RwSignal::new(None::<F::Model>);
        let baseline = RwSignal::new(None::<F::Model>);
        let errors = RwSignal::new(HashMap::new());
        let fields = RwSignal::new(StoredValue::new(HashMap::new()));
        let failure = RwSignal::new(SaveFailure::default());
        Self {
            fields: fields.into(),
            draft: draft.into(),
            errors: errors.into(),
            save_error: Memo::new(move |_| failure.read().error.clone()).into(),
            problems: Memo::new(move |_| failure.read().problems.clone()).into(),
            field_values: fields,
            draft_model: draft,
            baseline,
            input_errors: errors,
            failure,
            shown_fields: RwSignal::new(HashMap::new()),
            touched: StoredValue::new(HashSet::new()),
            is_dirty: Memo::new(move |_| match (&*draft.read(), &*baseline.read()) {
                (Some(draft), Some(baseline)) => draft != baseline,
                _ => false,
            })
            .into(),
            // A memo, so that views keyed on readiness do not re-render on every draft change.
            is_ready: Memo::new(move |_| draft.read().is_some()).into(),
            has_errors: Memo::new(move |_| !errors.read().is_empty()).into(),
            is_saving,
        }
    }

    /// Returns the reactive value of `field`. Tracks entity (re)loads.
    pub fn field(&self, field: &F) -> Option<ReactiveField> {
        self.fields
            .get()
            .with_value(|fields| fields.get(field).copied())
    }

    /// Returns why `field` is invalid, if it is: its input error, or else the problems that the
    /// last save reported for it.
    pub fn field_error(&self, field: F) -> Signal<Option<String>> {
        let (errors, problems) = (self.errors, self.problems);
        Signal::derive(move || {
            errors.read().get(&field).cloned().or_else(|| {
                let name = field.name();
                let details = problems
                    .read()
                    .iter()
                    .filter(|problem| problem.field == name)
                    .map(|problem| problem.detail.clone())
                    .collect::<Vec<_>>();
                (!details.is_empty()).then(|| details.join(" "))
            })
        })
    }

    /// Applies user input for `field`.
    ///
    /// A value updates both the draft model and the field's reactive value and clears the field's
    /// input error. An error records the input error and leaves the draft unchanged. Either way,
    /// the problems that the last save reported for the field no longer apply.
    pub fn set_field(&self, field: F, value: Result<Value, String>) {
        let name = field.name();
        let has_problems = self
            .failure
            .with_untracked(|failure| failure.problems.iter().any(|problem| problem.field == name));
        if has_problems {
            self.failure.update(|failure| {
                failure.problems.retain(|problem| problem.field != name);
            });
        }
        self.touched.update_value(|touched| {
            touched.insert(field.clone());
        });
        match value {
            Ok(value) => {
                self.draft_model.update(|draft| {
                    if let Some(draft) = draft {
                        field.set_value(draft, value.clone());
                    }
                });
                self.input_errors.update(|errors| {
                    errors.remove(&field);
                });
                if let Some(reactive) = self.field(&field) {
                    reactive.set(value);
                } else {
                    tracing::error!(?field, "form has no reactive value for field");
                }
            }
            Err(err) => self.input_errors.update(|errors| {
                errors.insert(field, err);
            }),
        }
    }

    /// Replaces draft, baseline, and field values with `model`.
    fn load(&self, model: F::Model, fields: HashMap<F, ReactiveField>) {
        self.field_values.set(StoredValue::new(fields));
        self.baseline.set(Some(model.clone()));
        self.draft_model.set(Some(model));
        self.input_errors.update(HashMap::clear);
        self.touched.update_value(HashSet::clear);
        self.record_save_error(None);
    }

    /// Makes `model` the version the draft is compared against, and applies the user's changes to
    /// it, so that the form shows the current state of the entity and keeps the user's input.
    ///
    /// The fields that the user changed keep their draft value: those the user set to another
    /// value than the old baseline's, and those whose input could not be read. The other fields
    /// take the value of `model`, and so does what the model holds besides its fields, e.g. its
    /// version. `values` returns the value of every field of a model. A save refused as outdated
    /// no longer applies.
    fn rebase(&self, model: F::Model, values: impl Fn(&F::Model) -> Vec<(F, Value)>) {
        let (Some(draft), Some(baseline)) = (
            self.draft_model.get_untracked(),
            self.baseline.get_untracked(),
        ) else {
            self.baseline.set(Some(model));
            return;
        };
        let touched = self.touched.get_value();
        let changes = self.input_errors.with_untracked(|input_errors| {
            values(&draft)
                .into_iter()
                .filter(|(field, value)| {
                    touched.contains(field)
                        && (input_errors.contains_key(field) || {
                            // Values cannot be compared, models can.
                            let mut changed = baseline.clone();
                            field.set_value(&mut changed, value.clone());
                            changed != baseline
                        })
                })
                .collect::<HashMap<_, _>>()
        });
        let mut rebased = model.clone();
        for (field, value) in &changes {
            field.set_value(&mut rebased, value.clone());
        }
        let shown = self.fields.get_untracked();
        for (field, value) in values(&rebased) {
            if changes.contains_key(&field) {
                continue;
            }
            if let Some(reactive) = shown.with_value(|shown| shown.get(&field).copied()) {
                reactive.set(value);
            }
        }
        self.baseline.set(Some(model));
        self.draft_model.set(Some(rebased));
        self.clear_outdated();
    }

    /// Replaces draft, baseline, and field values with `model`, keeping the existing reactive
    /// values, so that rendered fields update in place.
    fn sync(&self, model: F::Model, fields: HashMap<F, ReactiveField>) {
        let current = self.fields.get_untracked();
        for (field, reactive) in fields {
            let existing = current.with_value(|current| current.get(&field).copied());
            match existing {
                Some(existing) => existing.set(reactive.get_untracked()),
                None => current.update_value(|current| {
                    current.insert(field, reactive);
                }),
            }
        }
        self.baseline.set(Some(model.clone()));
        self.draft_model.set(Some(model));
        self.input_errors.update(HashMap::clear);
        self.touched.update_value(HashSet::clear);
        self.record_save_error(None);
    }

    /// Records why the last save failed, or `None` after a successful save. The problems of fields
    /// that `error` reports replace those of earlier saves.
    fn record_save_error(&self, error: Option<RequestError>) {
        let problems = match &error {
            Some(RequestError::InvalidFields { problems, .. }) => problems.clone(),
            _ => Vec::new(),
        };
        let failure = SaveFailure { error, problems };
        if self.failure.with_untracked(|current| *current != failure) {
            self.failure.set(failure);
        }
    }

    /// Forgets a save refused as outdated, e.g. once the entity is reloaded.
    fn clear_outdated(&self) {
        let is_outdated = self
            .failure
            .with_untracked(|failure| matches!(failure.error, Some(RequestError::Outdated(_))));
        if is_outdated {
            self.failure.update(|failure| failure.error = None);
        }
    }

    /// Records that the field named `name` is shown, until the current owner is cleaned up. Field
    /// atoms call it, so that the form shows the problems of other fields with its failure.
    pub(crate) fn show_field(&self, name: String) {
        self.shown_fields
            .update(|shown| *shown.entry(name.clone()).or_default() += 1);
        let shown_fields = self.shown_fields;
        on_cleanup(move || {
            shown_fields.try_update(|shown| {
                if let Some(count) = shown.get_mut(&name) {
                    *count -= 1;
                    if *count == 0 {
                        shown.remove(&name);
                    }
                }
            });
        });
    }

    /// Returns the problems that the last save reported for fields that the form does not show.
    pub(crate) fn unshown_problems(&self) -> Signal<Vec<CrudFieldProblem>> {
        let (problems, shown_fields) = (self.problems, self.shown_fields);
        Signal::derive(move || {
            let shown = shown_fields.read();
            problems
                .read()
                .iter()
                .filter(|problem| !shown.contains_key(&problem.field))
                .cloned()
                .collect()
        })
    }
}

/// Why the last save of a form failed, and the problems of fields it reported that still apply.
#[derive(Debug, Clone, Default, PartialEq)]
struct SaveFailure {
    /// `None` after a successful save.
    error: Option<RequestError>,
    problems: Vec<CrudFieldProblem>,
}

/// Which outcomes of saving a form notify the user, through the instance's [`CrudNotifier`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum CrudSaveNotifications {
    /// Successes and failures.
    #[default]
    All,
    /// Failures only, e.g. when the application reports successes itself.
    Failures,
    /// None.
    None,
}

/// Input of [`use_crud_create_form`].
#[derive(Debug, Clone, Default)]
pub struct UseCrudCreateFormInput {
    /// Called after the entity was created, while the form is mounted.
    pub on_saved: Option<Callback<CrudSaved>>,
    /// Called when creating the entity failed, while the form is mounted.
    pub on_save_failed: Option<Callback<RequestError>>,
    /// Which outcomes notify the user. Notifications are sent even when the form is gone by then.
    pub notifications: CrudSaveNotifications,
}

/// Output of [`use_crud_create_form`].
#[derive(Debug, Clone, Copy)]
pub struct UseCrudCreateFormReturn {
    /// The draft of the entity to create.
    pub form: CrudFormState<DynCreateField>,
    /// Creates the drafted entity, then performs the given follow-up.
    pub save: Callback<CrudSaveFollowUp>,
    /// Whether a save request is in flight.
    pub is_saving: Signal<bool>,
    /// Whether [`Self::save`] is currently allowed. An unchanged default draft may be saved.
    pub can_save: Signal<bool>,
    /// Non-critical validation violations reported by the last successful save, if any.
    pub violations: Signal<Option<PartialSerializableAggregateViolations>>,
}

/// Creates a form for a new entity of the surrounding instance.
///
/// The draft starts from the create model's default, with the parent reference filled in for nested
/// instances. A changed draft guards the current navigation.
///
/// # Panics
///
/// Panics when called outside of a CrudKit instance.
#[must_use]
// Hooks take their input by value, following Leptonic's `use_x(UseXInput) -> UseXReturn` convention.
#[allow(clippy::needless_pass_by_value)]
pub fn use_crud_create_form(input: UseCrudCreateFormInput) -> UseCrudCreateFormReturn {
    let UseCrudCreateFormInput {
        on_saved,
        on_save_failed,
        notifications,
    } = input;
    let ctx = use_crud_instance();
    let navigation = ctx.navigation;
    let violations = RwSignal::new(None::<PartialSerializableAggregateViolations>);

    let save_action = create_action(&ctx, notifications);
    let is_saving: Signal<bool> = save_action.pending().into();
    let form = CrudFormState::<DynCreateField>::new(is_saving);
    let default_model = default_create_model(&ctx);
    let fields = reactive_fields(
        ctx.static_config
            .read_value()
            .model_handler
            .create_model_values(&default_model),
    );
    form.load(default_model, fields);
    navigation.guard(form.is_dirty);

    let save_result = save_action.value();
    let reset = move || {
        let model = default_create_model(&ctx);
        let fields = reactive_fields(
            ctx.static_config
                .read_value()
                .model_handler
                .create_model_values(&model),
        );
        form.sync(model, fields);
    };
    // The outcome was already notified by the request. The form applies it while it is mounted.
    Effect::new(move |_prev| {
        let Some((result, follow_up)) = save_result.get() else {
            return;
        };
        // Callbacks may read signals. They must not become dependencies of this effect, or a
        // change would replay the save result.
        untrack(move || match result {
            Ok(saved) => {
                form.record_save_error(None);
                violations.set(Some(saved.violations.clone()));
                let id = saved.entity.id();
                if let Some(on_saved) = on_saved {
                    on_saved.run(saved);
                }
                follow_up.perform(ctx.navigation, id, CrudEntityViewKind::Create, reset);
            }
            Err(request_error) => {
                form.record_save_error(Some(request_error.clone()));
                if let Some(on_save_failed) = on_save_failed {
                    on_save_failed.run(request_error);
                }
            }
        });
    });

    let save = Callback::new(move |follow_up: CrudSaveFollowUp| {
        if let Some(draft) = savable_draft(form) {
            save_action.dispatch((draft, follow_up));
        }
    });

    let can_save = Signal::derive(move || !is_saving.get() && !form.has_errors.get());

    UseCrudCreateFormReturn {
        form,
        save,
        is_saving,
        can_save,
        violations: violations.into(),
    }
}

/// Whether an edited or displayed entity can be shown.
#[derive(Debug, Clone, PartialEq)]
pub enum CrudEntityLoadStatus {
    /// The entity is being loaded, or its field values are being prepared.
    Loading,
    /// The entity is loaded and the form holds its field values.
    Ready,
    /// No entity matches the requested ID.
    NotFound,
    /// Loading the entity failed.
    Failed(RequestError),
}

/// Returns the status of `entity`, shown through `form`.
fn entity_status<F: TypeErasedField>(
    entity: Signal<LoadState<DynUpdateModel>>,
    form: CrudFormState<F>,
) -> Signal<CrudEntityLoadStatus> {
    // A memo, so that views keyed on the status re-render only when it changes.
    Memo::new(move |_| match &*entity.read() {
        LoadState::Loaded(_) if form.is_ready.get() => CrudEntityLoadStatus::Ready,
        LoadState::Loading | LoadState::Loaded(_) => CrudEntityLoadStatus::Loading,
        LoadState::NotFound => CrudEntityLoadStatus::NotFound,
        LoadState::Failed(error) => CrudEntityLoadStatus::Failed(error.clone()),
    })
    .into()
}

/// Result of a save request, together with the follow-up requested for it.
type SaveOutcome = (Result<CrudSaved, RequestError>, CrudSaveFollowUp);

/// Creates the action sending create requests for the instance of `ctx`. It notifies the outcome
/// as `notifications` ask, also when the form is gone by then.
// TODO: Can we get rid of new_local?
fn create_action(
    ctx: &CrudInstanceContext,
    notifications: CrudSaveNotifications,
) -> Action<(DynCreateModel, CrudSaveFollowUp), SaveOutcome> {
    let ctx = *ctx;
    let notify = SaveNotifier::new(&ctx, notifications);
    Action::new_local(
        move |(create_model, follow_up): &(DynCreateModel, CrudSaveFollowUp)| {
            let entity = create_model.clone();
            let follow_up = *follow_up;
            let data_source = ctx.data_source();
            let scope = ctx.scope.get_untracked();
            async move {
                let result = data_source
                    .create(&scope, CrudCreateRequest::new(entity))
                    .await;
                notify.outcome(
                    &result,
                    SaveTexts {
                        completed: |texts| &texts.created,
                        accepted: |texts| &texts.creation_accepted,
                        failed: |texts| &texts.create_failed,
                    },
                );
                (result, follow_up)
            }
        },
    )
}

/// Creates the action sending update requests for the instance of `ctx`. It notifies the outcome
/// as `notifications` ask, also when the form is gone by then.
fn update_action(
    ctx: &CrudInstanceContext,
    notifications: CrudSaveNotifications,
) -> Action<(CrudUpdateRequest, CrudSaveFollowUp), SaveOutcome> {
    let ctx = *ctx;
    let notify = SaveNotifier::new(&ctx, notifications);
    Action::new_local(
        move |(request, follow_up): &(CrudUpdateRequest, CrudSaveFollowUp)| {
            let request = request.clone();
            let follow_up = *follow_up;
            let data_source = ctx.data_source();
            let scope = ctx.scope.get_untracked();
            async move {
                let result = data_source.update(&scope, request).await;
                notify.outcome(
                    &result,
                    SaveTexts {
                        completed: |texts| &texts.saved,
                        accepted: |texts| &texts.change_accepted,
                        failed: |texts| &texts.update_failed,
                    },
                );
                (result, follow_up)
            }
        },
    )
}

/// The messages of the outcomes of one kind of save.
#[derive(Clone, Copy)]
struct SaveTexts {
    /// After a completed save.
    completed: fn(&CrudUiTexts) -> &str,
    /// After a save that the data source accepted, and still applies.
    accepted: fn(&CrudUiTexts) -> &str,
    /// After a failed save, followed by the error.
    failed: fn(&CrudUiTexts) -> &str,
}

/// Notifies the outcomes of save requests through the instance's notifier.
#[derive(Clone, Copy)]
struct SaveNotifier {
    notifier: CrudNotifier,
    texts: Signal<CrudUiTexts>,
    notifications: CrudSaveNotifications,
}

impl SaveNotifier {
    fn new(ctx: &CrudInstanceContext, notifications: CrudSaveNotifications) -> Self {
        Self {
            notifier: ctx.notifier,
            texts: use_crud_texts(),
            notifications,
        }
    }

    /// Notifies `result` with one of `messages`, as the notifications ask. A save refused as
    /// outdated is a warning, which asks the user to look at the current state.
    fn outcome(self, result: &Result<CrudSaved, RequestError>, messages: SaveTexts) {
        let notification = match result {
            Ok(saved) if self.notifications == CrudSaveNotifications::All => {
                let message = match saved.completion {
                    CrudCompletion::Completed => messages.completed,
                    CrudCompletion::Accepted => messages.accepted,
                };
                with_texts(self.texts, |texts| {
                    CrudNotification::success(texts.save.to_string(), message(texts))
                })
            }
            Err(error @ RequestError::Outdated(_))
                if self.notifications != CrudSaveNotifications::None =>
            {
                tracing::warn!(%error, "could not save the entity");
                with_texts(self.texts, |texts| {
                    CrudNotification::warning(texts.save.to_string(), texts.outdated.to_string())
                })
            }
            Err(error) if self.notifications != CrudSaveNotifications::None => {
                tracing::warn!(%error, "could not save the entity");
                with_texts(self.texts, |texts| {
                    CrudNotification::error(
                        texts.error.to_string(),
                        format!("{} {error}", (messages.failed)(texts)),
                    )
                })
            }
            Ok(_) => return,
            Err(error) => {
                tracing::warn!(%error, "could not save the entity");
                return;
            }
        };
        self.notifier.notify(notification);
    }
}

/// Returns the draft of `form`, unless it is missing or holds rejected input.
fn savable_draft<F: TypeErasedField>(form: CrudFormState<F>) -> Option<F::Model> {
    if form.has_errors.get_untracked() {
        tracing::warn!("not saving a draft with input errors");
        return None;
    }
    form.draft.get_untracked()
}

/// Returns the create model's default, with the IDs of the scope's parents in the fields that
/// refer to them. Data sources that place new entities below their parents, e.g. in a path, need
/// no such fields, so the create model may lack them.
fn default_create_model(ctx: &CrudInstanceContext) -> DynCreateModel {
    let model_handler = ctx.static_config.read_value().model_handler;
    let mut entity = model_handler.default_create_model();
    for parent in ctx.scope.get_untracked().parents {
        if let Some(field) = model_handler.create_model_field(&parent.field_name) {
            field.set_value(&mut entity, parent.value.into());
        } else {
            tracing::debug!(
                field = %parent.field_name,
                "the create model has no field referring to the parent; not prefilling it"
            );
        }
    }
    entity
}

/// Input of [`use_crud_edit_form`].
#[derive(Debug, Clone)]
pub struct UseCrudEditFormInput {
    /// ID of the edited entity.
    pub id: Signal<SerializableId>,
    /// Called after the entity was saved, while the form is mounted.
    pub on_saved: Option<Callback<CrudSaved>>,
    /// Called when saving the entity failed, while the form is mounted.
    pub on_save_failed: Option<Callback<RequestError>>,
    /// Which outcomes notify the user. Notifications are sent even when the form is gone by then.
    pub notifications: CrudSaveNotifications,
}

/// Output of [`use_crud_edit_form`].
#[derive(Debug, Clone, Copy)]
pub struct UseCrudEditFormReturn {
    /// The entity as loaded from the server.
    pub entity: Signal<LoadState<DynUpdateModel>>,
    /// Whether the entity can be shown, see [`CrudEntityLoadStatus`].
    pub status: Signal<CrudEntityLoadStatus>,
    /// Non-critical validation violations reported by the last successful save, if any.
    pub violations: Signal<Option<PartialSerializableAggregateViolations>>,
    /// The draft of the edited entity. Reset whenever the entity (re)loads, unless it holds
    /// unsaved changes of that entity.
    pub form: CrudFormState<DynUpdateField>,
    /// Saves the draft, then performs the given follow-up.
    pub save: Callback<CrudSaveFollowUp>,
    /// Whether a save request is in flight.
    pub is_saving: Signal<bool>,
    /// Whether [`Self::save`] is currently useful: the draft is changed and no save is in flight.
    pub can_save: Signal<bool>,
    /// Asks the user to confirm deleting the entity.
    pub delete: Callback<()>,
    /// Whether [`Self::delete`] is currently allowed.
    pub can_delete: Signal<bool>,
    /// Reloads the entity to show its current state, e.g. after a save was refused as
    /// [outdated](RequestError::Outdated). The draft keeps its unsaved changes, which then apply
    /// to the reloaded version.
    pub reload: Callback<()>,
}

/// Loads an entity of the surrounding instance and creates a form editing it.
///
/// A changed draft guards the current navigation. Saving sends the draft and the baseline it
/// started from, the version that was loaded or saved last. A failed save keeps the draft, and the
/// form's [`save_error`](CrudFormState::save_error) tells why: a save refused as
/// [outdated](RequestError::Outdated) asks for [`UseCrudEditFormReturn::reload`], and the problems
/// of [invalid fields](RequestError::InvalidFields) reach their fields.
///
/// # Panics
///
/// Panics when called outside of a CrudKit instance.
#[must_use]
// Hooks take their input by value, following Leptonic's `use_x(UseXInput) -> UseXReturn` convention.
#[allow(clippy::needless_pass_by_value)]
pub fn use_crud_edit_form(input: UseCrudEditFormInput) -> UseCrudEditFormReturn {
    let UseCrudEditFormInput {
        id,
        on_saved,
        on_save_failed,
        notifications,
    } = input;
    let ctx = use_crud_instance();
    let navigation = ctx.navigation;
    let violations = RwSignal::new(None::<PartialSerializableAggregateViolations>);

    let entity = use_entity(&ctx, id);
    let save_action = update_action(&ctx, notifications);
    let is_saving: Signal<bool> = save_action.pending().into();
    let form = CrudFormState::<DynUpdateField>::new(is_saving);
    load_into_form(&ctx, entity, form);
    navigation.guard(form.is_dirty);

    let save_result = save_action.value();
    // The outcome was already notified by the request. The form applies it while it is mounted.
    Effect::new(move |_prev| {
        let Some((result, follow_up)) = save_result.get() else {
            return;
        };
        // Callbacks may read signals. They must not become dependencies of this effect, or a
        // change would replay the save result.
        untrack(move || match result {
            Ok(saved) => {
                violations.set(Some(saved.violations.clone()));
                // The server may have normalized the entity. The form continues from its version.
                let fields = reactive_fields(
                    ctx.static_config
                        .read_value()
                        .model_handler
                        .update_model_values(&saved.entity),
                );
                let saved_id = saved.entity.id();
                form.sync(saved.entity.clone(), fields);
                if let Some(on_saved) = on_saved {
                    on_saved.run(saved);
                }
                follow_up.perform(navigation, saved_id, CrudEntityViewKind::Update, || {});
            }
            Err(request_error) => {
                // The draft stays as it is, also when the entity changed meanwhile: reloading it
                // applies the draft's changes to the current version.
                form.record_save_error(Some(request_error.clone()));
                if let Some(on_save_failed) = on_save_failed {
                    on_save_failed.run(request_error);
                }
            }
        });
    });

    let is_dirty = form.is_dirty;
    let save = Callback::new(move |follow_up: CrudSaveFollowUp| {
        let (Some(draft), Some(baseline)) = (savable_draft(form), form.baseline.get_untracked())
        else {
            return;
        };
        let request = CrudUpdateRequest::new(id.get_untracked(), draft, baseline);
        save_action.dispatch((request, follow_up));
    });
    // Deletes the entity as the data source last returned it, the baseline: the draft may hold
    // unsaved changes, even of its ID fields, and the baseline holds the version that the
    // deletion is based on, also after a save.
    let delete = Callback::new(move |()| {
        if let Some(entity) = form.baseline.get_untracked() {
            ctx.deletion.for_navigation(navigation).request(entity);
        }
    });
    let reload = Callback::new(move |()| {
        form.clear_outdated();
        ctx.reload();
    });

    UseCrudEditFormReturn {
        entity,
        status: entity_status(entity, form),
        violations: violations.into(),
        form,
        save,
        is_saving,
        can_save: Signal::derive(move || {
            !is_saving.get() && is_dirty.get() && !form.has_errors.get()
        }),
        delete,
        can_delete: Signal::derive(move || !is_saving.get() && entity.read().loaded().is_some()),
        reload,
    }
}

/// Output of [`use_crud_read`].
#[derive(Debug, Clone, Copy)]
pub struct UseCrudReadReturn {
    /// The entity as loaded from the server.
    pub entity: Signal<LoadState<DynUpdateModel>>,
    /// Whether the entity can be shown, see [`CrudEntityLoadStatus`].
    pub status: Signal<CrudEntityLoadStatus>,
    /// Reactive values of the entity's fields. Not meant to be edited.
    pub form: CrudFormState<DynUpdateField>,
    /// Asks the user to confirm deleting the entity.
    pub delete: Callback<()>,
    /// Whether [`Self::delete`] is currently allowed.
    pub can_delete: Signal<bool>,
    /// Reloads the entity to show its current state.
    pub reload: Callback<()>,
}

/// Loads an entity of the surrounding instance for read-only display.
///
/// # Panics
///
/// Panics when called outside of a CrudKit instance.
pub fn use_crud_read(id: impl Into<Signal<SerializableId>>) -> UseCrudReadReturn {
    let ctx = use_crud_instance();
    let navigation = ctx.navigation;

    let entity = use_entity(&ctx, id.into());
    let form = CrudFormState::<DynUpdateField>::new(Signal::stored(false));
    load_into_form(&ctx, entity, form);

    let delete = Callback::new(move |()| {
        if let Some(entity) = entity.get_untracked().into_loaded() {
            ctx.deletion.for_navigation(navigation).request(entity);
        }
    });

    UseCrudReadReturn {
        entity,
        status: entity_status(entity, form),
        form,
        delete,
        can_delete: Signal::derive(move || entity.read().loaded().is_some()),
        reload: Callback::new(move |()| ctx.reload()),
    }
}

/// Loads the entity identified by `id` in the instance's scope.
fn use_entity(
    ctx: &CrudInstanceContext,
    id: Signal<SerializableId>,
) -> Signal<LoadState<DynUpdateModel>> {
    let ctx = *ctx;
    // TODO: Do not use LocalResource, allow loading on the server.
    let resource = LocalResource::new(move || async move {
        let _ = ctx.reload.get();
        let id = id.get();
        let scope = ctx.scope.get();
        ctx.data_source()
            .read(&scope, CrudReadRequest::new(id))
            .await
    });

    Memo::new(move |_prev| match resource.get() {
        Some(result) => LoadState::from_optional_result(result).map(|read_model| {
            ctx.static_config
                .read_value()
                .model_handler
                .read_model_to_update_model(read_model)
        }),
        None => LoadState::Loading,
    })
    .into()
}

/// Resets `form` to every newly loaded version of `entity`.
///
/// A reload of the entity the form holds unsaved changes of keeps those changes: they apply to the
/// reloaded version, which becomes what they are compared against, and the fields the user did not
/// change show the reloaded values. Discarding the changes is up to the user, e.g. by leaving the
/// view.
fn load_into_form(
    ctx: &CrudInstanceContext,
    entity: Signal<LoadState<DynUpdateModel>>,
    form: CrudFormState<DynUpdateField>,
) {
    let ctx = *ctx;
    // The values of the fields outlive the run of the effect that creates them, which disposes
    // what it created when it runs again.
    let owner = Owner::current();
    Effect::new(move |_prev| {
        if let LoadState::Loaded(model) = entity.get() {
            let values_of = |model: &DynUpdateModel| {
                ctx.static_config
                    .read_value()
                    .model_handler
                    .update_model_values(model)
            };
            let shows_this_entity = form.draft.with_untracked(|draft| {
                draft.as_ref().is_some_and(|draft| draft.id() == model.id())
            });
            if shows_this_entity && form.is_dirty.get_untracked() {
                form.rebase(model, values_of);
            } else if shows_this_entity {
                // Rendered fields update in place.
                let fields = reactive_fields(values_of(&model));
                form.sync(model, fields);
            } else {
                let values = values_of(&model);
                let load = move || form.load(model, reactive_fields(values));
                match &owner {
                    Some(owner) => owner.with(load),
                    None => load(),
                }
            }
        }
    });
}
