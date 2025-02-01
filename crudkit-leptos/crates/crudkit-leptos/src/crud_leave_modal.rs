use leptonic::components::prelude::*;
use leptonic::prelude::*;
use leptos::prelude::*;

#[component]
pub fn CrudLeaveModal(
    #[prop(into)] show_when: Signal<bool>,
    #[prop(into)] on_cancel: Callback<(), ()>,
    #[prop(into)] on_accept: Callback<(), ()>,
) -> impl IntoView {
    let g_keyboard_event: GlobalKeyboardEvent = expect_context::<GlobalKeyboardEvent>();
    Effect::new(move |_old| {
        if let Some(e) = g_keyboard_event.read_signal.get() {
            if show_when.get_untracked() && e.key().as_str() == "Escape" {
                on_cancel.run(());
            }
        }
    });

    view! {
        <Modal show_when=show_when>
            <ModalHeader>
                <ModalTitle>"Ungespeicherte Änderungen"</ModalTitle>
            </ModalHeader>

            <ModalBody attr:style="text-align: center;">
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
                                    on_press=move |_| on_cancel.run(())
                                >
                                    "Zurück"
                                </Button>
                                <Button
                                    color=ButtonColor::Warn
                                    on_press=move |_| on_accept.run(())
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
