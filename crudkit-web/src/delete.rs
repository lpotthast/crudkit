//! UI-independent classification of deletion results.

use crate::data_source::CrudDeletedMany;
use crate::http::RequestError;

/// Classification of a [`CrudDeletedMany`] outcome for user feedback.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeleteManyOutcome {
    /// Every targeted entity was deleted.
    Complete {
        /// Number of deleted entities.
        deleted: u64,
    },
    /// Some, but not all, targeted entities were deleted.
    Partial(DeleteManyCounts),
    /// No entity was deleted.
    Failed(DeleteManyCounts),
}

/// Counts reported by a mass deletion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeleteManyCounts {
    /// Number of deleted entities.
    pub deleted: u64,
    /// Number of deletions refused or rejected, e.g. by lifecycle hooks.
    pub aborted: usize,
    /// Number of deletions prevented by critical validation errors.
    pub validation_failed: usize,
    /// Number of deletions that failed with another error.
    pub errors: usize,
    /// Number of entities not tried, because a failure stopped the deletion.
    pub skipped: usize,
}

impl From<&CrudDeletedMany> for DeleteManyOutcome {
    fn from(result: &CrudDeletedMany) -> Self {
        let mut counts = DeleteManyCounts {
            deleted: result.deleted.len() as u64,
            aborted: 0,
            validation_failed: 0,
            errors: 0,
            skipped: result.skipped.len(),
        };
        for (_, error) in &result.failed {
            match error {
                RequestError::Forbidden(_) | RequestError::UnprocessableEntity(_) => {
                    counts.aborted += 1;
                }
                RequestError::CriticalValidationErrors { .. } => counts.validation_failed += 1,
                _ => counts.errors += 1,
            }
        }
        let has_failures = !result.failed.is_empty() || counts.skipped > 0;
        match (counts.deleted, has_failures) {
            (0, _) => Self::Failed(counts),
            (deleted, false) => Self::Complete { deleted },
            (_, true) => Self::Partial(counts),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assertr::prelude::*;
    use crudkit_core::id::SerializableId;

    fn deleted_many(deleted: usize, failed: Vec<RequestError>, skipped: usize) -> CrudDeletedMany {
        let id = || SerializableId(Vec::new());
        let mut outcome = CrudDeletedMany::new();
        outcome.deleted = (0..deleted).map(|_| id()).collect();
        outcome.failed = failed.into_iter().map(|error| (id(), error)).collect();
        outcome.skipped = (0..skipped).map(|_| id()).collect();
        outcome
    }

    #[test]
    fn delete_many_outcomes_are_classified() {
        assert_that!(DeleteManyOutcome::from(&deleted_many(3, Vec::new(), 0)))
            .is_equal_to(DeleteManyOutcome::Complete { deleted: 3 });
        assert_that!(DeleteManyOutcome::from(&deleted_many(
            2,
            vec![
                RequestError::Forbidden(String::new()),
                RequestError::NotFound(String::new()),
            ],
            1,
        )))
        .is_equal_to(DeleteManyOutcome::Partial(DeleteManyCounts {
            deleted: 2,
            aborted: 1,
            validation_failed: 0,
            errors: 1,
            skipped: 1,
        }));
        assert_that!(matches!(
            DeleteManyOutcome::from(&deleted_many(
                0,
                vec![RequestError::Request(String::new())],
                0
            )),
            DeleteManyOutcome::Failed(DeleteManyCounts { errors: 1, .. })
        ))
        .is_true();
        assert_that!(matches!(
            DeleteManyOutcome::from(&deleted_many(0, Vec::new(), 0)),
            DeleteManyOutcome::Failed(_)
        ))
        .is_true();
    }
}
