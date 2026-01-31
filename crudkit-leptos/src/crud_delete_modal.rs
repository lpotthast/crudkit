use crudkit_web::prelude::DynReadOrUpdateModel;
use leptonic::components::prelude::*;
use leptonic::prelude::*;
use leptos::prelude::*;

#[component]
pub fn CrudDeleteModal(
    // Modal is shown when this Signal contains a Some value.
    #[prop(into)] entity: Signal<Option<DynReadOrUpdateModel>>,
    #[prop(into)] on_cancel: Callback<()>,
    #[prop(into)] on_accept: Callback<DynReadOrUpdateModel>,
) -> impl IntoView {
    let show_when = Signal::derive(move || entity.get().is_some());

    view! {
        <Modal show_when=show_when on_escape=move || on_cancel.run(())>
            <ModalHeader>
                <ModalTitle>
                    // TODO: Consider using an "EntryVisualizer" of some sort...
                    {move || {
                        entity
                            .get()
                            .map(|it| {
                                // TODO: Use other (customizable) stringify method.
                                let id = match it {
                                    DynReadOrUpdateModel::Read(model) => model.id(),
                                    DynReadOrUpdateModel::Update(model) => model.id(),
                                };
                                format!("Löschen - {id:?}")
                            })
                    }}

                </ModalTitle>
            </ModalHeader>

            <ModalBody>"Bist du dir sicher?" <br/> "Dieser Eintrag kann nicht wiederhergestellt werden!"</ModalBody>

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
                                        if let Some(model) = entity.get() { on_accept.run(model) }
                                    }
                                >
                                    "Löschen"
                                </Button>
                            </ButtonWrapper>
                        </Col>
                    </Row>
                </Grid>
            </ModalFooter>
        </Modal>
    }
}
