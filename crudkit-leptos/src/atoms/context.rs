//! The instance boundary: what an instance hides from the atoms inside it.

use crate::hooks::entity::CrudEntityHandle;
use crate::hooks::form::CrudFormHandle;
use crate::hooks::table::RowEntity;
use leptos::context::Provider;
use leptos::prelude::*;

/// Hides the table row, form, and entity of an outer instance from the descendants of an instance,
/// so atoms of a nested instance act on their own instance only. The hook counterpart of
/// [`InstanceBoundary`], for `provide_crud_instance`; both hide the same contexts.
pub(crate) fn provide_instance_boundary() {
    provide_context(None::<CrudEntityHandle>);
    provide_context(None::<CrudFormHandle>);
    provide_context(None::<RowEntity>);
}

/// Renders `children` behind an instance boundary, see [`provide_instance_boundary`].
#[component]
pub(crate) fn InstanceBoundary(children: Children) -> impl IntoView {
    let (no_entity, no_form, no_row) = (
        None::<CrudEntityHandle>,
        None::<CrudFormHandle>,
        None::<RowEntity>,
    );
    view! {
        <Provider value=no_entity>
            <Provider value=no_form>
                <Provider value=no_row>{children()}</Provider>
            </Provider>
        </Provider>
    }
}
