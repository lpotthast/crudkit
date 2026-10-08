//! The user-facing texts of CrudKit's atoms and notifications.

use crate::config::CrudUiTexts;
use leptos::prelude::*;

/// The provided texts, see [`provide_crud_texts`].
#[derive(Clone, Copy)]
struct ProvidedTexts(Signal<CrudUiTexts>);

/// Makes `texts` the texts of CrudKit UI rendered below the current owner. A signal switches the
/// language of everything rendered, e.g. `Signal::derive(move || texts_for(locale.get()))`.
pub fn provide_crud_texts(texts: impl Into<Signal<CrudUiTexts>>) {
    provide_context(ProvidedTexts(texts.into()));
}

/// Applies `read` to `texts`, or to the default texts once `texts` is disposed, e.g. when a request
/// completes after the component that started it is gone.
pub(crate) fn with_texts<T>(texts: Signal<CrudUiTexts>, read: impl Fn(&CrudUiTexts) -> T) -> T {
    texts
        .try_with_untracked(&read)
        .unwrap_or_else(|| read(&CrudUiTexts::default()))
}

/// Returns the nearest provided texts, or the defaults. Read them where they are rendered, e.g.
/// `move || texts.read().save.to_string()`, so that rendered texts follow a change of language.
#[must_use]
pub fn use_crud_texts() -> Signal<CrudUiTexts> {
    use_context::<ProvidedTexts>()
        .map_or_else(|| Signal::stored(CrudUiTexts::default()), |texts| texts.0)
}
