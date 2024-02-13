use std::marker::PhantomData;

use crudkit_web::{CrudIdTrait, CrudMainTrait, DeletableModel};
use leptonic::prelude::*;
use leptonic::components::prelude::*;
use leptos::*;

#[component]
pub fn CrudDeleteModal<T>(
    _phantom: PhantomData<T>,
    // Modal is shown when this Signal contains a Some value.
    #[prop(into)] entity: Signal<Option<DeletableModel<T::ReadModel, T::UpdateModel>>>, // TODO: Do not take Option
    #[prop(into)] on_cancel: Callback<()>,
    #[prop(into)] on_accept: Callback<DeletableModel<T::ReadModel, T::UpdateModel>>,
) -> impl IntoView
where
    T: CrudMainTrait + 'static,
{
    let show_when = Signal::derive(move || entity.get().is_some());

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
                <ModalTitle>
                    // TODO: Consider using an "EntryVisualizer" of some sort...
                    {move || {
                        entity
                            .get()
                            .map(|it| {
                                format!(
                                    "Löschen - {}", match it { DeletableModel::Read(model) => model.get_id()
                                    .to_string(), DeletableModel::Update(model) => model.get_id().to_string(), }
                                )
                            })
                            .unwrap_or_default()
                    }}

                </ModalTitle>
            </ModalHeader>

            <ModalBody>"Bist du dir sicher?" <br/> "Dieser Eintrag kann nicht wiederhergestellt werden!"</ModalBody>

            <ModalFooter>
                <Grid spacing=Size::Em(0.6)>
                    <Row>
                        <Col h_align=ColAlign::End>
                            <ButtonWrapper>
                                <Button
                                    color=ButtonColor::Secondary
                                    on_press=move |_| {
                                        on_cancel.call(())
                                    }
                                >
                                    "Zurück"
                                </Button>
                                <Button
                                    color=ButtonColor::Danger
                                    on_press=move |_| {
                                        match entity.get() {
                                            Some(model) => on_accept.call(model),
                                            None => {}
                                        }
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
