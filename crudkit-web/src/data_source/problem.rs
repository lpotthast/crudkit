//! The problems with which a data source refuses an entity, field by field.

use crate::json_pointer::{escape_token, split_first_token};

/// A problem with one field of an entity, e.g. an entry of the `errors` of problem details
/// (RFC 9457) that names the field by a JSON pointer.
///
/// A data source refuses an entity with its problems in
/// [`RequestError::InvalidFields`](crate::http::RequestError::InvalidFields). A form shows each
/// problem at the field it names, where the field can show it, and lists every problem in its
/// error summary.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct CrudFieldProblem {
    /// The name of the field, as the model's field enum names it (`Named::name`), e.g. `groups`.
    /// A name that the model does not have is shown in the form's error summary only.
    pub field: String,
    /// The JSON pointer (RFC 6901) of the problem into the entity as the model serializes it,
    /// e.g. `/groups/0/count`. It starts with the field.
    pub pointer: String,
    /// What is wrong with the field, and what to do.
    pub detail: String,
}

impl CrudFieldProblem {
    /// A problem with the field `field` as a whole, which `detail` describes.
    pub fn new(field: impl Into<String>, detail: impl Into<String>) -> Self {
        let field = field.into();
        Self {
            pointer: format!("/{}", escape_token(&field)),
            field,
            detail: detail.into(),
        }
    }

    /// A problem at the JSON pointer `pointer` into the entity, e.g. `/groups/0/count`, which
    /// `detail` describes. The pointer's first reference token names the field. `None` for the
    /// pointer to the whole entity, `""`, and for text that is no JSON pointer.
    pub fn at(pointer: impl Into<String>, detail: impl Into<String>) -> Option<Self> {
        let pointer = pointer.into();
        let (field, _) = split_first_token(&pointer)?;
        Some(Self {
            field,
            pointer,
            detail: detail.into(),
        })
    }

    /// Returns the part of the pointer below the field, e.g. `/0/count` for `/groups/0/count`.
    /// Empty for a problem with the field as a whole.
    #[must_use]
    pub fn path_in_field(&self) -> &str {
        split_first_token(&self.pointer).map_or("", |(_, rest)| rest)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assertr::prelude::*;

    #[test]
    fn problems_name_their_field_and_where_in_it_they_are() {
        let nested =
            CrudFieldProblem::at("/groups/0/count", "must be positive").expect("a problem");
        let whole = CrudFieldProblem::new("a/b", "is taken");

        assert_that!((nested.field.as_str(), nested.path_in_field()))
            .is_equal_to(("groups", "/0/count"));
        assert_that!((whole.pointer.as_str(), whole.path_in_field())).is_equal_to(("/a~1b", ""));
        assert_that!(CrudFieldProblem::at("", "the body is invalid")).is_none();
    }
}
