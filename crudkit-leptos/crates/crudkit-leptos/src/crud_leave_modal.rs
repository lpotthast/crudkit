use leptonic::prelude::*;
use leptonic::components::prelude::*;
use leptos::*;

#[component]
pub fn CrudLeaveModal(
    #[prop(into)] show_when: Signal<bool>,
    #[prop(into)] on_cancel: Producer<()>,
    #[prop(into)] on_accept: Producer<()>,
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
        <Modal show_when=show_when>
            <ModalHeader>
                <ModalTitle>"Ungespeicherte Änderungen"</ModalTitle>
            </ModalHeader>

            <ModalBody style="text-align: center;">
                "Du hast deine Änderungen noch nicht gespeichert." <br/> "Möchtest du den Bereich wirklich verlassen?"
                <br/> "Ungespeicherte Änderungen gehen verloren!"
            </ModalBody>

            <ModalFooter>
                <Grid gap=Size::Em(0.6)>
                    <Row>
                        <Col h_align=ColAlign::End>
                            <ButtonWrapper>
                                <Button
                                    color=ButtonColor::Secondary
                                    on_press=move |_| on_cancel.call(())
                                >
                                    "Zurück"
                                </Button>
                                <Button
                                    color=ButtonColor::Warn
                                    on_press=move |_| on_accept.call(())
                                >
                                    "Verlassen"
                                </Button>
                            </ButtonWrapper>
                        </Col>
                    </Row>
                </Grid>
            </ModalFooter>
        </Modal>
    }
}
