//! The texts of CrudKit's atoms and notifications, in the user's language.

use crate::config::{CrudText, CrudTranslator};
use crudkit_web::http::RequestError;
use leptos::prelude::*;
use std::fmt;
use std::sync::Arc;

/// CrudKit's texts in the user's language: translated by the [`CrudTranslator`] that
/// [`provide_crud_translator`] provided, else in English. Returned by [`use_crud_texts`].
///
/// The default shows the English texts.
#[derive(Clone, Copy, Default)]
pub struct CrudTexts {
    translator: Option<StoredValue<Arc<dyn CrudTranslator>>>,
}

impl fmt::Debug for CrudTexts {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CrudTexts")
            .field("is_translated", &self.translator.is_some())
            .finish()
    }
}

impl CrudTexts {
    /// Texts translated by `translator`, which the current owner keeps.
    pub(crate) fn new(translator: impl CrudTranslator + 'static) -> Self {
        let translator: Arc<dyn CrudTranslator> = Arc::new(translator);
        Self {
            translator: Some(StoredValue::new(translator)),
        }
    }

    /// Returns `text` in the user's language. Read texts where they are rendered, e.g.
    /// `move || texts.get(CrudText::FormSave)`, because this tracks the reactive state that the
    /// translator reads, so that rendered texts follow a change of language.
    ///
    /// Returns the English default when the translator has no translation, without a translator,
    /// and once the owner that provided the translator is gone, e.g. for a request that completes
    /// after its view.
    #[must_use]
    pub fn get(&self, text: CrudText<'_>) -> String {
        // Clone the translator out of the arena, so that it does not translate while holding
        // the arena, e.g. when it reads signals.
        self.translator
            .and_then(|translator| translator.try_get_value())
            .and_then(|translator| translator.translate(&text))
            .unwrap_or_else(|| text.english().into_owned())
    }

    /// Returns `text` in the user's language, like [`Self::get`], but without tracking, e.g. for
    /// the message of a notification.
    #[must_use]
    pub fn get_untracked(&self, text: CrudText<'_>) -> String {
        untrack(|| self.get(text))
    }

    /// Returns the text explaining `error` to the user: the text of its kind, such as
    /// [`CrudText::ErrorForbidden`], followed by its message ([`CrudText::ErrorMessage`]) or its
    /// technical detail ([`CrudText::ErrorDetail`]), unless the error has none. Tracks like
    /// [`Self::get`].
    #[must_use]
    pub fn request_error(&self, error: &RequestError) -> String {
        let (kind, addition) = explain(error);
        let kind = self.get(kind);
        match addition {
            Addition::Message(message) if !message.trim().is_empty() => {
                self.get(CrudText::ErrorMessage {
                    error: &kind,
                    message: message.trim(),
                })
            }
            Addition::Detail(detail) if !detail.trim().is_empty() => {
                self.get(CrudText::ErrorDetail {
                    error: &kind,
                    detail: detail.trim(),
                })
            }
            Addition::Message(_) | Addition::Detail(_) => kind,
        }
    }

    /// Returns the text explaining `error`, like [`Self::request_error`], but without tracking.
    #[must_use]
    pub fn request_error_untracked(&self, error: &RequestError) -> String {
        untrack(|| self.request_error(error))
    }
}

/// What a request error adds to the text of its kind.
enum Addition<'a> {
    /// A message of the data source, meant for the user.
    Message(&'a str),
    /// A technical detail in English, for diagnosing the error.
    Detail(&'a str),
}

/// Returns the text of the kind of `error`, and what the error adds to it.
fn explain(error: &RequestError) -> (CrudText<'static>, Addition<'_>) {
    match error {
        RequestError::BadRequest(message) => {
            (CrudText::ErrorBadRequest, Addition::Message(message))
        }
        RequestError::Unauthorized(message) => {
            (CrudText::ErrorUnauthorized, Addition::Message(message))
        }
        RequestError::Forbidden(message) => (CrudText::ErrorForbidden, Addition::Message(message)),
        RequestError::NotFound(message) => (CrudText::ErrorNotFound, Addition::Message(message)),
        RequestError::UnprocessableEntity(message) => {
            (CrudText::ErrorUnprocessable, Addition::Message(message))
        }
        RequestError::CriticalValidationErrors { message, .. } => {
            (CrudText::ErrorValidation, Addition::Message(message))
        }
        RequestError::Outdated(message) => (CrudText::ErrorOutdated, Addition::Message(message)),
        RequestError::InvalidFields { message, .. } => {
            (CrudText::ErrorInvalidFields, Addition::Message(message))
        }
        RequestError::InternalServerError(message) => {
            (CrudText::ErrorServer, Addition::Message(message))
        }
        RequestError::InvalidRequest(detail) => {
            (CrudText::ErrorInvalidRequest, Addition::Detail(detail))
        }
        RequestError::Request(detail) => (CrudText::ErrorNetwork, Addition::Detail(detail)),
        RequestError::Deserialize(detail) => {
            (CrudText::ErrorInvalidResponse, Addition::Detail(detail))
        }
    }
}

/// Makes `translator` translate the texts of the CrudKit UI rendered below the current owner.
/// Provide it once, at the root of the application. Without a translator, CrudKit shows its
/// English texts.
pub fn provide_crud_translator(translator: impl CrudTranslator + 'static) {
    provide_context(CrudTexts::new(translator));
}

/// Returns CrudKit's texts below the current owner: translated by the nearest translator that
/// [`provide_crud_translator`] provided, else in English.
#[must_use]
pub fn use_crud_texts() -> CrudTexts {
    use_context::<CrudTexts>().unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use assertr::prelude::*;
    use leptos::reactive::owner::Owner;

    /// Translates the save button's label only.
    fn german(text: &CrudText<'_>) -> Option<String> {
        matches!(text, CrudText::FormSave).then(|| "Speichern".to_owned())
    }

    #[test]
    fn texts_without_a_translator_are_english() {
        Owner::new().with(|| {
            let texts = use_crud_texts();

            assert_that!(texts.get(CrudText::FormSave)).is_equal_to("Save".to_owned());
            assert_that!(texts.get_untracked(CrudText::SelectionCount { count: 2 }))
                .is_equal_to("2 selected".to_owned());
        });
    }

    #[test]
    fn texts_the_translator_does_not_know_are_english() {
        Owner::new().with(|| {
            provide_crud_translator(german);
            let texts = use_crud_texts();

            assert_that!(texts.get(CrudText::FormSave)).is_equal_to("Speichern".to_owned());
            assert_that!(texts.get(CrudText::FormReturn)).is_equal_to("Return".to_owned());
        });
    }

    #[test]
    fn untracked_texts_do_not_follow_the_translator() {
        Owner::new().with(|| {
            let german = RwSignal::new(false);
            provide_crud_translator(move |text: &CrudText<'_>| match *text {
                CrudText::FormSave if german.get() => Some("Speichern".to_owned()),
                _ => None,
            });
            let texts = use_crud_texts();
            let tracked = Memo::new(move |_| texts.get(CrudText::FormSave));
            let untracked = Memo::new(move |_| texts.get_untracked(CrudText::FormSave));
            assert_that!(untracked.get_untracked()).is_equal_to("Save".to_owned());
            assert_that!(tracked.get_untracked()).is_equal_to("Save".to_owned());

            german.set(true);

            assert_that!(tracked.get_untracked()).is_equal_to("Speichern".to_owned());
            assert_that!(untracked.get_untracked()).is_equal_to("Save".to_owned());
        });
    }

    #[test]
    fn request_errors_add_their_message_or_detail_only_when_they_have_one() {
        let texts = CrudTexts::default();
        let explain = |error: RequestError| texts.request_error(&error);

        assert_that!(explain(RequestError::Forbidden("Not yours.".to_owned())))
            .is_equal_to("Permission denied: Not yours.".to_owned());
        assert_that!(explain(RequestError::Forbidden(String::new())))
            .is_equal_to("Permission denied".to_owned());
        assert_that!(explain(RequestError::InternalServerError(" \n".to_owned())))
            .is_equal_to("Server error".to_owned());
        assert_that!(explain(RequestError::Request(
            "connection refused".to_owned()
        )))
        .is_equal_to("Network error: connection refused".to_owned());
    }

    #[test]
    fn texts_are_english_once_the_owner_of_their_translator_is_gone() {
        let owner = Owner::new();
        let texts = owner.with(|| {
            provide_crud_translator(german);
            use_crud_texts()
        });
        assert_that!(texts.get_untracked(CrudText::FormSave)).is_equal_to("Speichern".to_owned());

        drop(owner);

        assert_that!(texts.get_untracked(CrudText::FormSave)).is_equal_to("Save".to_owned());
    }
}
