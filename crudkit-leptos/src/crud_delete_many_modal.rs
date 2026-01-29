use crudkit_web::prelude::DynReadModel;
use leptonic::components::prelude::*;
use leptonic::prelude::*;
use leptos::prelude::*;
use std::sync::Arc;

#[component]
pub fn CrudDeleteManyModal(
    // Modal is shown when this Signal contains a Some value with entities to delete.
    #[prop(into)] entities: Signal<Option<Arc<Vec<DynReadModel>>>>,
    #[prop(into)] on_cancel: Callback<()>,
    #[prop(into)] on_accept: Callback<Arc<Vec<DynReadModel>>>,
) -> impl IntoView {
    let show_when = Signal::derive(move || entities.get().is_some());
    let count = Signal::derive(move || entities.get().map(|e| e.len()).unwrap_or(0));

    view! {
        <Modal show_when=show_when on_escape=move || on_cancel.run(())>
            <ModalHeader>
                <ModalTitle>
                    {move || format!("{} Einträge löschen", count.get())}
                </ModalTitle>
            </ModalHeader>

            <ModalBody>
                "Bist du dir sicher?" <br/>
                {move || {
                    let n = count.get();
                    if n == 1 {
                        "Dieser Eintrag kann nicht wiederhergestellt werden!".to_string()
                    } else {
                        format!("Diese {} Einträge können nicht wiederhergestellt werden!", n)
                    }
                }}
            </ModalBody>

            <ModalFooter>
                <Grid gap=Size::Em(0.6)>
                    <Row>
                        <Col h_align=ColAlign::End>
                            <ButtonWrapper>
                                <Button
                                    color=ButtonColor::Secondary
                                    on_press=move |_| {
                                        on_cancel.run(())
                                    }
                                >
                                    "Zurück"
                                </Button>
                                <Button
                                    color=ButtonColor::Danger
                                    on_press=move |_| {
                                        if let Some(entities) = entities.get() {
                                            on_accept.run(entities)
                                        }
                                    }
                                >
                                    {move || {
                                        let n = count.get();
                                        if n == 1 {
                                            "Löschen".to_string()
                                        } else {
                                            format!("{} Einträge löschen", n)
                                        }
                                    }}
                                </Button>
                            </ButtonWrapper>
                        </Col>
                    </Row>
                </Grid>
            </ModalFooter>
        </Modal>
    }
}
