use leptonic::{prelude::*, root::GlobalKeyboardEvent};
use leptos::*;

#[component]
pub fn CrudLeaveModal<C, A>(
    cx: Scope,
    #[prop(into)] show_when: Signal<bool>,
    on_cancel: C,
    on_accept: A,
) -> impl IntoView
where
    C: Fn() + Clone + 'static,
    A: Fn() + Clone + 'static,
{
    let on_cancel = store_value(cx, on_cancel);
    let on_accept = store_value(cx, on_accept);

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
                    "Ungespeicherte Änderungen"
                </ModalTitle>
            </ModalHeader>

            <ModalBody style="text-align: center;">
                "Du hast deine Änderungen noch nicht gespeichert."<br />
                "Möchtest du den Bereich wirklich verlassen?"<br />
                "Ungespeicherte Änderungen gehen verloren!"
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
                                <Button color=ButtonColor::Warn on_click=move |_| {
                                    tracing::info!("leave");
                                    (on_accept.get_value())()
                                }>
                                    "Verlassen"
                                </Button>
                            </ButtonWrapper>
                        </Col>
                    </Row>
                </Grid>
            </ModalFooter>
        </ModalFn>
    }
}
