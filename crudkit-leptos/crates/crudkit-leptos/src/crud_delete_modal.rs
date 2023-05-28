use crudkit_web::{CrudDataTrait, CrudIdTrait};
use leptonic::{prelude::*, root::GlobalKeyboardEvent};
use leptos::*;

#[component]
pub fn CrudDeleteModal<T, C, A>(
    cx: Scope,
    // Modal is shown when this Signal contains a Some value.
    entity: Signal<Option<T>>, // TODO: Take DeletableModel instead? Do not take Option
    on_cancel: C,
    on_accept: A,
) -> impl IntoView
where
    T: CrudDataTrait + CrudIdTrait + 'static,
    C: Fn() + Clone + 'static,
    A: Fn(T) + Clone + 'static,
{
    let on_cancel = store_value(cx, on_cancel);
    let on_accept = store_value(cx, on_accept);

    let show_when = Signal::derive(cx, move || entity.get().is_some());

    let g_keyboard_event: GlobalKeyboardEvent = expect_context::<GlobalKeyboardEvent>(cx);
    create_effect(cx, move |_old| {
        let is_shown = show_when.get_untracked();
        if let Some(e) = g_keyboard_event.read_signal.get() {
            if is_shown && e.key().as_str() == "Escape" {
                (on_cancel.get_value())();
            }
        }
    });

    view! {cx,
        <ModalFn show_when=show_when>
            <ModalHeader>
                <ModalTitle>
                    //TODO: Consider using an "EntryVisualizer" of some sort...
                    { move || format!("Löschen - {}", entity.get().unwrap().get_id()) }
                </ModalTitle>
            </ModalHeader>

            <ModalBody>
                "Bist du dir sicher?"<br />
                "Dieser Eintrag kann nicht wiederhergestellt werden!"
            </ModalBody>

            <ModalFooter>
                <Grid spacing=6>
                    <Row>
                        <Col h_align=ColAlign::End>
                            <ButtonWrapper>
                                <Button color=ButtonColor::Secondary on_click=move |_| {
                                    tracing::info!("cancel");
                                    (on_cancel.get_value())()
                                }>
                                    "Zurück"
                                </Button>
                                <Button color=ButtonColor::Danger on_click=move |_| {
                                    tracing::info!("cancel");
                                    (on_accept.get_value())(entity.get().unwrap())
                                }>
                                    "Löschen"
                                </Button>
                            </ButtonWrapper>
                        </Col>
                    </Row>
                </Grid>
            </ModalFooter>
        </ModalFn>
    }
}
