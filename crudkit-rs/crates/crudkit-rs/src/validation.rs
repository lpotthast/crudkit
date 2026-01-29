#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrudAction {
    Create,
    Read,
    Update,
    Delete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum When {
    Before,
    After,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ValidationContext {
    /// The CRUD action that lead to the validation.
    pub action: CrudAction,
    /// Whether or not the validation occurs before or after applying the CRUD action.
    /// Critical violations created before the action is applied will prevent its application.
    pub when: When,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationTrigger {
    CrudAction(ValidationContext),
    GlobalValidation,
}
