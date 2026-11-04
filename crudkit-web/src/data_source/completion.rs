//! How far a change or a deletion got when its data source answered.

/// How far a change or a deletion got when its data source answered.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum CrudCompletion {
    /// The change or deletion is complete.
    #[default]
    Completed,
    /// The data source accepted the change or deletion, and the entity is still changing, e.g.
    /// because an API answered `202 Accepted` and reconciles the entity in the background. The
    /// outcome shows the entity in its state at the time of the answer, e.g. `deleting`.
    Accepted,
}
