//! Single and mass deletion of entities.

use crate::config::CrudUiTexts;
use crate::hooks::instance::use_crud_instance;
use crate::hooks::notify::{CrudNotification, CrudNotifier};
use crate::hooks::texts::with_texts;
use crate::instance::{CommittedReturnDestination, CrudNavigation};
use crudkit_web::data_source::{
    CrudCompletion, CrudDataSource, CrudDeleteManyRequest, CrudDeleteRequest, CrudDeleted,
    CrudDeletedMany, CrudScope,
};
use crudkit_web::delete::{DeleteManyCounts, DeleteManyOutcome};
use crudkit_web::http::RequestError;
use crudkit_web::prelude::*;
use leptos::prelude::*;
use std::sync::Arc;

/// Deletion requests of one instance, awaiting the user's confirmation.
///
/// A request only records what should be deleted. Nothing is deleted until the request is
/// confirmed, typically through a confirmation dialog.
#[derive(Clone, Copy)]
pub struct CrudDeleteState {
    /// The entity awaiting confirmation of its deletion, if any.
    pub pending: Signal<Option<DynReadOrUpdateModel>>,
    /// The entities awaiting confirmation of their mass deletion, if any.
    pub pending_many: Signal<Option<Arc<Vec<DynReadModel>>>>,
    /// Whether a confirmed deletion is in flight.
    pub is_deleting: Signal<bool>,
    requested: RwSignal<Option<(DynReadOrUpdateModel, CrudNavigation)>>,
    requested_many: RwSignal<Option<Arc<Vec<DynReadModel>>>>,
    delete_action: Action<(DynReadOrUpdateModel, CrudNavigation), ()>,
    delete_many_action: Action<Arc<Vec<DynReadModel>>, ()>,
    /// Navigation of the view requesting deletions, which returns after a single deletion.
    navigation: CrudNavigation,
}

impl std::fmt::Debug for CrudDeleteState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CrudDeleteState").finish_non_exhaustive()
    }
}

impl CrudDeleteState {
    pub(crate) fn new(
        data_source: Arc<dyn CrudDataSource>,
        reload: Callback<()>,
        notifier: CrudNotifier,
        texts: Signal<CrudUiTexts>,
        navigation: CrudNavigation,
        scope: Signal<CrudScope>,
    ) -> Self {
        let pending = RwSignal::new(None::<(DynReadOrUpdateModel, CrudNavigation)>);
        let pending_many = RwSignal::new(None::<Arc<Vec<DynReadModel>>>);

        let single_data_source = data_source.clone();
        let delete_action = Action::new_local(
            move |(entity, navigation): &(DynReadOrUpdateModel, CrudNavigation)| {
                let data_source = single_data_source.clone();
                let request = CrudDeleteRequest::new(entity.clone());
                let id = request.id.clone();
                let navigation = *navigation;
                // Deletions stay within the instance's scope, like reads.
                let scope = scope.get_untracked();
                async move {
                    let result = data_source.delete(&scope, request).await;
                    pending.set(None);
                    notifier.notify(with_texts(texts, |texts| {
                        delete_notification(texts, &result)
                    }));
                    // Someone else changed the entity since the user saw it. Reloading shows its
                    // current state, so that the user can decide again.
                    if matches!(result, Err(RequestError::Outdated(_))) {
                        reload.try_run(());
                        return;
                    }
                    // The requesting view may have been unmounted while the request was in flight.
                    if result.is_err() || navigation.is_disposed() {
                        return;
                    }
                    // Only a view showing the deleted entity leaves it. Persistence resolved the
                    // entity, so follow-up navigation must not ask about now-obsolete input again.
                    // A return callback leaves refreshing to the application.
                    let shows_deleted =
                        navigation.current().get_untracked().subject.as_ref() == Some(&id);
                    if !shows_deleted
                        || navigation.return_committed() == CommittedReturnDestination::View
                    {
                        reload.try_run(());
                    }
                }
            },
        );

        let delete_many_action = Action::new_local(move |entities: &Arc<Vec<DynReadModel>>| {
            let data_source = data_source.clone();
            let request = CrudDeleteManyRequest::new(
                entities
                    .iter()
                    .cloned()
                    .map(DynReadOrUpdateModel::from)
                    .collect(),
            );
            let scope = scope.get_untracked();
            async move {
                let result = data_source.delete_many(&scope, request).await;
                pending_many.set(None);
                notifier.notify(with_texts(texts, |texts| {
                    delete_many_notification(texts, &result)
                }));
                // The instance may have been unmounted while the request was in flight.
                reload.try_run(());
            }
        });

        let (single, many) = (delete_action.pending(), delete_many_action.pending());
        Self {
            pending: Signal::derive(move || {
                pending.read().as_ref().map(|(entity, _)| entity.clone())
            }),
            pending_many: pending_many.into(),
            is_deleting: Signal::derive(move || single.get() || many.get()),
            requested: pending,
            requested_many: pending_many,
            delete_action,
            delete_many_action,
            navigation,
        }
    }

    /// Returns this state for requests made by the view of `navigation`.
    pub(crate) fn for_navigation(mut self, navigation: CrudNavigation) -> Self {
        self.navigation = navigation;
        self
    }

    /// Requests deleting `entity`. After a confirmed deletion, a requesting view showing `entity`
    /// (whose view [`subject`](crudkit_web::view::CrudView::subject) is its ID) returns, e.g. from
    /// an edit view to the list, also when the data source only accepted the deletion and the
    /// entity is still being deleted. Other views, such as the list, stay and reload. A deletion
    /// refused because someone else changed the entity reloads the instance, so that it shows the
    /// current state.
    pub fn request(&self, entity: impl Into<DynReadOrUpdateModel>) {
        self.requested.set(Some((entity.into(), self.navigation)));
    }

    /// Requests deleting `entities`. Requests without entities are ignored.
    pub fn request_many(&self, entities: impl Into<Arc<Vec<DynReadModel>>>) {
        let entities = entities.into();
        if !entities.is_empty() {
            self.requested_many.set(Some(entities));
        }
    }

    /// Deletes the entity of the pending request. Ignored while a deletion is in flight.
    pub fn confirm(&self) {
        if self.delete_action.pending().get_untracked() {
            return;
        }
        if let Some(requested) = self.requested.get_untracked() {
            self.delete_action.dispatch(requested);
        }
    }

    /// Discards the pending request.
    pub fn cancel(&self) {
        self.requested.set(None);
    }

    /// Deletes the entities of the pending mass deletion request. Ignored while a deletion is in
    /// flight.
    pub fn confirm_many(&self) {
        if self.delete_many_action.pending().get_untracked() {
            return;
        }
        if let Some(entities) = self.requested_many.get_untracked() {
            self.delete_many_action.dispatch(entities);
        }
    }

    /// Discards the pending mass deletion request.
    pub fn cancel_many(&self) {
        self.requested_many.set(None);
    }

    /// Discards all pending requests.
    pub(crate) fn clear(&self) {
        self.cancel();
        self.cancel_many();
    }
}

/// Returns the deletion requests of the surrounding instance.
///
/// # Panics
///
/// Panics when called outside of a CrudKit instance.
#[must_use]
pub fn use_crud_delete() -> CrudDeleteState {
    let ctx = use_crud_instance();
    ctx.deletion.for_navigation(ctx.navigation)
}

/// Describes the outcome of a single deletion.
pub(crate) fn delete_notification(
    texts: &CrudUiTexts,
    result: &Result<CrudDeleted, RequestError>,
) -> CrudNotification {
    let title = texts.deletion.to_string();
    match result {
        Ok(deleted) => {
            let message = match deleted.completion {
                CrudCompletion::Completed => (texts.deleted)(deleted.entities_affected),
                CrudCompletion::Accepted => (texts.deletion_accepted)(deleted.entities_affected),
            };
            CrudNotification::success(title, message)
        }
        Err(RequestError::Forbidden(reason)) => {
            CrudNotification::warning(title, (texts.deletion_forbidden)(reason))
        }
        Err(RequestError::Outdated(_)) => {
            CrudNotification::warning(title, texts.deletion_outdated.to_string())
        }
        Err(
            RequestError::UnprocessableEntity(reason)
            | RequestError::CriticalValidationErrors {
                message: reason, ..
            }
            | RequestError::InvalidFields {
                message: reason, ..
            },
        ) => CrudNotification::warning(title, (texts.deletion_rejected)(reason)),
        Err(error) => CrudNotification::error(title, (texts.deletion_failed)(&error.to_string())),
    }
}

/// Describes the outcome of a mass deletion.
pub(crate) fn delete_many_notification(
    texts: &CrudUiTexts,
    result: &Result<CrudDeletedMany, RequestError>,
) -> CrudNotification {
    let title = texts.deletion.to_string();
    match result {
        Ok(deleted_many) => match DeleteManyOutcome::from(deleted_many) {
            DeleteManyOutcome::Complete { deleted, accepted } => CrudNotification::success(
                title,
                with_accepted(texts, texts.deleted.as_ref(), deleted, accepted),
            ),
            DeleteManyOutcome::Partial(counts) => CrudNotification::warning(
                title,
                with_failures(
                    texts,
                    with_accepted(
                        texts,
                        texts.deleted_partially.as_ref(),
                        counts.deleted,
                        counts.accepted,
                    ),
                    counts,
                ),
            ),
            DeleteManyOutcome::Failed(counts) => CrudNotification::error(
                title,
                with_failures(texts, texts.nothing_deleted.to_string(), counts),
            ),
        },
        Err(err) => CrudNotification::error(title, (texts.deletion_failed)(&err.to_string())),
    }
}

/// Reports `deleted` deleted entities with `deleted_text`, followed by the `accepted` entities that
/// are still being deleted. Leaves out the deleted entities when there are none, but some accepted
/// ones.
fn with_accepted(
    texts: &CrudUiTexts,
    deleted_text: &(dyn Fn(u64) -> String + Send + Sync),
    deleted: u64,
    accepted: u64,
) -> String {
    match (deleted, accepted) {
        (_, 0) => deleted_text(deleted),
        (0, accepted) => (texts.deletion_accepted)(accepted),
        (deleted, accepted) => format!(
            "{} {}",
            deleted_text(deleted),
            (texts.deletion_accepted)(accepted)
        ),
    }
}

/// Appends the sentences reporting the failures in `counts` to `message`.
fn with_failures(texts: &CrudUiTexts, message: String, counts: DeleteManyCounts) -> String {
    let failures = [
        (counts.aborted, &texts.deletions_aborted),
        (counts.validation_failed, &texts.deletions_invalid),
        (counts.errors, &texts.deletions_failed),
        (counts.skipped, &texts.deletions_skipped),
    ];
    failures
        .into_iter()
        .filter(|(count, _)| *count > 0)
        .fold(message, |message, (count, text)| {
            format!("{message} {}", text(count as u64))
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hooks::notify::CrudNotificationKind;
    use assertr::prelude::*;
    use crudkit_core::id::SerializableId;

    #[test]
    fn single_deletion_outcomes_map_to_notification_kinds() {
        let ok = delete_notification(&CrudUiTexts::default(), &Ok(CrudDeleted::new(1)));
        assert_that!(ok.kind).is_equal_to(CrudNotificationKind::Success);
        assert_that!(ok.message).is_equal_to("1 Eintrag erfolgreich gelöscht.".to_owned());

        let forbidden = delete_notification(
            &CrudUiTexts::default(),
            &Err(RequestError::Forbidden("nope".to_owned())),
        );
        assert_that!(forbidden.kind).is_equal_to(CrudNotificationKind::Warning);

        let failed = delete_notification(
            &CrudUiTexts::default(),
            &Err(RequestError::Request("offline".to_owned())),
        );
        assert_that!(failed.kind).is_equal_to(CrudNotificationKind::Error);
    }

    #[test]
    fn mass_deletion_outcomes_map_to_notification_kinds() {
        let id = || SerializableId(Vec::new());
        let mut outcome = CrudDeletedMany::new();
        outcome.deleted = vec![id(), id()];
        outcome.failed = vec![(id(), RequestError::Forbidden(String::new()))];
        outcome.skipped = vec![id()];
        let partial = delete_many_notification(&CrudUiTexts::default(), &Ok(outcome));
        assert_that!(partial.kind).is_equal_to(CrudNotificationKind::Warning);
        assert_that!(partial.message)
            .is_equal_to("2 Einträge gelöscht. 1 abgebrochen. 1 nicht versucht.".to_owned());

        let failed = delete_many_notification(
            &CrudUiTexts::default(),
            &Err(RequestError::Request("offline".to_owned())),
        );
        assert_that!(failed.kind).is_equal_to(CrudNotificationKind::Error);
    }
}
