//! UI-independent classification of deletion results.

use crudkit_core::DeletedMany;

/// Classification of a [`DeletedMany`] response for user feedback.
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
    /// Number of deletions aborted by lifecycle hooks.
    pub aborted: usize,
    /// Number of deletions prevented by critical validation errors.
    pub validation_failed: usize,
    /// Number of deletions that failed with an error.
    pub errors: usize,
}

impl From<&DeletedMany> for DeleteManyOutcome {
    fn from(result: &DeletedMany) -> Self {
        let counts = DeleteManyCounts {
            deleted: result.deleted_count,
            aborted: result.aborted.len(),
            validation_failed: result.validation_failed.len(),
            errors: result.errors.len(),
        };
        let has_failures = counts.aborted > 0 || counts.validation_failed > 0 || counts.errors > 0;
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

    fn deleted_many(deleted: u64, aborted: usize, errors: usize) -> DeletedMany {
        DeletedMany {
            deleted_count: deleted,
            deleted_ids: Vec::new(),
            aborted: (0..aborted)
                .map(|_| (SerializableId(Vec::new()), String::new()))
                .collect(),
            validation_failed: Vec::new(),
            errors: (0..errors)
                .map(|_| (SerializableId(Vec::new()), String::new()))
                .collect(),
        }
    }

    #[test]
    fn delete_many_outcomes_are_classified() {
        assert_that!(DeleteManyOutcome::from(&deleted_many(3, 0, 0)))
            .is_equal_to(DeleteManyOutcome::Complete { deleted: 3 });
        assert_that!(matches!(
            DeleteManyOutcome::from(&deleted_many(2, 1, 0)),
            DeleteManyOutcome::Partial(DeleteManyCounts {
                deleted: 2,
                aborted: 1,
                ..
            })
        ))
        .is_true();
        assert_that!(matches!(
            DeleteManyOutcome::from(&deleted_many(0, 0, 1)),
            DeleteManyOutcome::Failed(DeleteManyCounts { errors: 1, .. })
        ))
        .is_true();
        assert_that!(matches!(
            DeleteManyOutcome::from(&deleted_many(0, 0, 0)),
            DeleteManyOutcome::Failed(_)
        ))
        .is_true();
    }
}
