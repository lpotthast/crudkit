//! Single and mass deletion of entities.

use crate::config::CrudText;
use crate::hooks::instance::use_crud_instance;
use crate::hooks::notify::{CrudNotification, CrudNotifier};
use crate::hooks::texts::CrudTexts;
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
        texts: CrudTexts,
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
                    notifier.notify(delete_notification(texts, &result));
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
            let requested = entities.len() as u64;
            async move {
                let result = data_source.delete_many(&scope, request).await;
                pending_many.set(None);
                notifier.notify(delete_many_notification(texts, requested, &result));
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
    texts: CrudTexts,
    result: &Result<CrudDeleted, RequestError>,
) -> CrudNotification {
    let text = |text: CrudText<'_>| texts.get_untracked(text);
    match result {
        Ok(deleted) => {
            let count = deleted.entities_affected;
            let (title, message) = match deleted.completion {
                CrudCompletion::Completed => (
                    CrudText::DeleteCompletedTitle { count },
                    CrudText::DeleteCompleted { count },
                ),
                CrudCompletion::Accepted => (
                    CrudText::DeleteAcceptedTitle { count },
                    CrudText::DeleteAccepted { count },
                ),
            };
            CrudNotification::success(text(title), text(message))
        }
        Err(RequestError::Outdated(_)) => CrudNotification::warning(
            text(CrudText::DeleteRefusedTitle),
            text(CrudText::DeleteOutdated),
        ),
        Err(
            error @ (RequestError::Forbidden(_)
            | RequestError::UnprocessableEntity(_)
            | RequestError::CriticalValidationErrors { .. }
            | RequestError::InvalidFields { .. }),
        ) => CrudNotification::warning(
            text(CrudText::DeleteRefusedTitle),
            texts.request_error_untracked(error),
        ),
        Err(error) => CrudNotification::error(
            text(CrudText::DeleteFailedTitle { count: 1 }),
            texts.request_error_untracked(error),
        ),
    }
}

/// Describes the outcome of a mass deletion of `requested` entities.
pub(crate) fn delete_many_notification(
    texts: CrudTexts,
    requested: u64,
    result: &Result<CrudDeletedMany, RequestError>,
) -> CrudNotification {
    let text = |text: CrudText<'_>| texts.get_untracked(text);
    let failed_title = || text(CrudText::DeleteFailedTitle { count: requested });
    match result {
        Ok(deleted_many) => match DeleteManyOutcome::from(deleted_many) {
            DeleteManyOutcome::Complete { deleted, accepted } => {
                let title = if deleted == 0 && accepted > 0 {
                    CrudText::DeleteAcceptedTitle { count: accepted }
                } else {
                    CrudText::DeleteCompletedTitle {
                        count: deleted + accepted,
                    }
                };
                CrudNotification::success(
                    text(title),
                    with_accepted(
                        texts,
                        |count| CrudText::DeleteCompleted { count },
                        deleted,
                        accepted,
                    ),
                )
            }
            DeleteManyOutcome::Partial(counts) => CrudNotification::warning(
                text(CrudText::DeleteManyPartialTitle),
                with_failures(
                    texts,
                    with_accepted(
                        texts,
                        |count| CrudText::DeleteManyPartial { count },
                        counts.deleted,
                        counts.accepted,
                    ),
                    counts,
                ),
            ),
            DeleteManyOutcome::Failed(counts) => CrudNotification::error(
                failed_title(),
                with_failures(texts, text(CrudText::DeleteManyNone), counts),
            ),
        },
        Err(error) => CrudNotification::error(failed_title(), texts.request_error_untracked(error)),
    }
}

/// Reports `deleted` deleted entities with `deleted_text`, followed by the `accepted` entities
/// that are still being deleted. Leaves out the deleted entities when there are none, but some
/// accepted ones.
fn with_accepted(
    texts: CrudTexts,
    deleted_text: fn(u64) -> CrudText<'static>,
    deleted: u64,
    accepted: u64,
) -> String {
    let deleted_text = || texts.get_untracked(deleted_text(deleted));
    let accepted_text = || texts.get_untracked(CrudText::DeleteAccepted { count: accepted });
    match (deleted, accepted) {
        (_, 0) => deleted_text(),
        (0, _) => accepted_text(),
        _ => format!("{} {}", deleted_text(), accepted_text()),
    }
}

/// Appends the sentences reporting the failures in `counts` to `message`.
fn with_failures(texts: CrudTexts, message: String, counts: DeleteManyCounts) -> String {
    let failures = [
        (
            counts.aborted,
            CrudText::DeleteManyAborted {
                count: counts.aborted as u64,
            },
        ),
        (
            counts.validation_failed,
            CrudText::DeleteManyInvalid {
                count: counts.validation_failed as u64,
            },
        ),
        (
            counts.errors,
            CrudText::DeleteManyFailed {
                count: counts.errors as u64,
            },
        ),
        (
            counts.skipped,
            CrudText::DeleteManySkipped {
                count: counts.skipped as u64,
            },
        ),
    ];
    failures
        .into_iter()
        .filter(|(count, _)| *count > 0)
        .fold(message, |message, (_, text)| {
            format!("{message} {}", texts.get_untracked(text))
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hooks::notify::CrudNotificationKind;
    use assertr::prelude::*;
    use crudkit_core::id::SerializableId;
    use leptos::reactive::owner::Owner;

    #[test]
    fn single_deletion_outcomes_map_to_notification_kinds() {
        let texts = CrudTexts::default();
        let ok = delete_notification(texts, &Ok(CrudDeleted::new(1)));
        assert_that!(ok.kind).is_equal_to(CrudNotificationKind::Success);
        assert_that!(ok.title).is_equal_to("Entry deleted".to_owned());
        assert_that!(ok.message).is_equal_to("1 entry deleted.".to_owned());

        let forbidden =
            delete_notification(texts, &Err(RequestError::Forbidden("nope".to_owned())));
        assert_that!(forbidden.kind).is_equal_to(CrudNotificationKind::Warning);
        assert_that!(forbidden.title).is_equal_to("Entry not deleted".to_owned());
        assert_that!(forbidden.message).is_equal_to("Permission denied: nope".to_owned());

        let failed = delete_notification(texts, &Err(RequestError::Request("offline".to_owned())));
        assert_that!(failed.kind).is_equal_to(CrudNotificationKind::Error);
        assert_that!(failed.title).is_equal_to("Entry not deleted".to_owned());
        assert_that!(failed.message).is_equal_to("Network error: offline".to_owned());
    }

    /// A mass deletion of four entities that deleted two, refused one, and left one untried.
    fn partial_deletion() -> CrudDeletedMany {
        let id = || SerializableId(Vec::new());
        let mut outcome = CrudDeletedMany::new();
        outcome.deleted = vec![id(), id()];
        outcome.failed = vec![(id(), RequestError::Forbidden(String::new()))];
        outcome.skipped = vec![id()];
        outcome
    }

    #[test]
    fn mass_deletion_outcomes_map_to_notification_kinds() {
        let texts = CrudTexts::default();
        let partial = delete_many_notification(texts, 4, &Ok(partial_deletion()));
        assert_that!(partial.kind).is_equal_to(CrudNotificationKind::Warning);
        assert_that!(partial.title).is_equal_to("Some entries not deleted".to_owned());
        assert_that!(partial.message)
            .is_equal_to("2 entries deleted. 1 refused. 1 not attempted.".to_owned());

        let failed =
            delete_many_notification(texts, 4, &Err(RequestError::Request("offline".to_owned())));
        assert_that!(failed.kind).is_equal_to(CrudNotificationKind::Error);
        assert_that!(failed.title).is_equal_to("Entries not deleted".to_owned());
    }

    #[test]
    fn deletion_outcomes_are_translated_with_their_counts() {
        Owner::new().with(|| {
            // Translates some texts only, so the others show their English defaults.
            let texts = CrudTexts::new(|text: &CrudText<'_>| match *text {
                CrudText::DeleteManyPartialTitle => Some("Nicht alle gelöscht".to_owned()),
                CrudText::DeleteManyPartial { count } => Some(match count {
                    1 => "1 Eintrag gelöscht.".to_owned(),
                    _ => format!("{count} Einträge gelöscht."),
                }),
                CrudText::DeleteManyAborted { count } => Some(format!("{count} abgelehnt.")),
                CrudText::DeleteFailedTitle { count: 1 } => Some("Nicht gelöscht".to_owned()),
                CrudText::ErrorNetwork => Some("Netzwerkfehler".to_owned()),
                // Leaves out the technical detail, which is English.
                CrudText::ErrorDetail { error, .. } => Some(error.to_owned()),
                _ => None,
            });

            let partial = delete_many_notification(texts, 4, &Ok(partial_deletion()));
            let failed =
                delete_notification(texts, &Err(RequestError::Request("offline".to_owned())));

            assert_that!(partial.title).is_equal_to("Nicht alle gelöscht".to_owned());
            assert_that!(partial.message)
                .is_equal_to("2 Einträge gelöscht. 1 abgelehnt. 1 not attempted.".to_owned());
            assert_that!(failed.title).is_equal_to("Nicht gelöscht".to_owned());
            assert_that!(failed.message).is_equal_to("Netzwerkfehler".to_owned());
        });
    }
}
