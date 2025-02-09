use leptonic::components::prelude::*;
use leptonic::prelude::*;
use leptos::prelude::*;

#[component]
pub fn CrudLeaveModal(
    #[prop(into)] show_when: Signal<bool>,
    #[prop(into)] on_cancel: Callback<(), ()>,
    #[prop(into)] on_accept: Callback<(), ()>,
) -> impl IntoView {
    view! {
        <Modal show_when=show_when on_escape=move || on_cancel.run(())>
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
