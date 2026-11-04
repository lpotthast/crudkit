//! The texts of CrudKit's atoms and notifications, in the user's language.

use crate::config::{CrudText, CrudTranslator};
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
            assert_that!(texts.get(CrudText::FormReturn)).is_equal_to("Back".to_owned());
        });
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
