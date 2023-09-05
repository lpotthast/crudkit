use leptonic::prelude::*;
use leptos::*;

#[component]
pub fn CrudLeaveModal(
    #[prop(into)] show_when: Signal<bool>,
    on_cancel: Callback<()>,
    on_accept: Callback<()>,
) -> impl IntoView {
    let g_keyboard_event: GlobalKeyboardEvent = expect_context::<GlobalKeyboardEvent>();
    create_effect(move |_old| {
        if let Some(e) = g_keyboard_event.read_signal.get() {
            if show_when.get_untracked() && e.key().as_str() == "Escape" {
                on_cancel.call(());
            }
        }
    });

    view! {
        <ModalFn show_when=show_when>
            <ModalHeader>
                <ModalTitle>"Ungespeicherte Änderungen"</ModalTitle>
            </ModalHeader>

            <ModalBody style="text-align: center;">
                "Du hast deine Änderungen noch nicht gespeichert." <br/> "Möchtest du den Bereich wirklich verlassen?"
                <br/> "Ungespeicherte Änderungen gehen verloren!"
            </ModalBody>

            <ModalFooter>
                <Grid spacing=Size::Em(0.6)>
                    <Row>
                        <Col h_align=ColAlign::End>
                            <ButtonWrapper>
                                <Button
                                    color=ButtonColor::Secondary
                                    on_click=move |_| {
                                        tracing::info!("cancel");
                                        on_cancel.call(());
                                    }
                                >

                                    "Zurück"
                                </Button>
                                <Button
                                    color=ButtonColor::Warn
                                    on_click=move |_| {
                                        tracing::info!("leave");
                                        on_accept.call(());
                                    }
                                >

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
