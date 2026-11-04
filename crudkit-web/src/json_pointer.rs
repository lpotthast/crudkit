//! JSON pointers (RFC 6901), which name a member of a JSON document, e.g. `/groups/0/count`.

/// Escapes `token` as a reference token of a JSON pointer: `~` becomes `~0`, and `/` becomes `~1`.
#[must_use]
pub fn escape_token(token: &str) -> String {
    token.replace('~', "~0").replace('/', "~1")
}

/// Returns the first reference token of `pointer`, unescaped, and the rest of the pointer, e.g.
/// `groups` and `/0/count` for `/groups/0/count`. `None` for the pointer to the whole document,
/// `""`, and for text that is no pointer.
#[must_use]
pub fn split_first_token(pointer: &str) -> Option<(String, &str)> {
    let rest = pointer.strip_prefix('/')?;
    let (token, rest) = rest.find('/').map_or((rest, ""), |end| rest.split_at(end));
    Some((token.replace("~1", "/").replace("~0", "~"), rest))
}

#[cfg(test)]
mod tests {
    use super::*;
    use assertr::prelude::*;

    #[test]
    fn pointers_split_into_their_first_token_and_the_rest() {
        assert_that!(split_first_token("/groups/0/count"))
            .is_equal_to(Some(("groups".to_owned(), "/0/count")));
        assert_that!(split_first_token("/a~1b~0c")).is_equal_to(Some(("a/b~c".to_owned(), "")));
        assert_that!(split_first_token("")).is_none();
        assert_that!(split_first_token("name")).is_none();
        assert_that!(escape_token("a/b~c")).is_equal_to("a~1b~0c".to_owned());
    }
}
