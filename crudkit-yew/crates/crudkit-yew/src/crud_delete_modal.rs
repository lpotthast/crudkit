use std::{marker::PhantomData, rc::Rc};
use yew::prelude::*;
use yewdux::prelude::Dispatch;

use super::prelude::*;
use crate::stores::global_key_up::GlobalKeyUp;

pub enum Msg {
    OnCancel,
    OnDelete,
    GlobalKeyUp(Rc<GlobalKeyUp>),
}

#[derive(Properties, PartialEq)]
pub struct Props<T: CrudDataTrait + CrudIdTrait> {
    pub entity: T,
    pub on_cancel: Callback<()>,
    pub on_delete: Callback<()>,
}

pub struct CrudDeleteModal<T: CrudDataTrait + CrudIdTrait> {
    _global_key_up_dispatch: Dispatch<GlobalKeyUp>,
    phantom_data: PhantomData<T>,
}

impl<T: 'static + CrudDataTrait + CrudIdTrait> Component for CrudDeleteModal<T> {
    type Message = Msg;
    type Properties = Props<T>;

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            _global_key_up_dispatch: Dispatch::subscribe(ctx.link().callback(Msg::GlobalKeyUp)),
            phantom_data: PhantomData {},
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::OnCancel => {
                ctx.props().on_cancel.emit(());
            }
            Msg::OnDelete => {
                ctx.props().on_delete.emit(());
            }
            Msg::GlobalKeyUp(state) => {
                if let Some(event) = state.latest_event() {
                    if event.key().as_str() == "Escape" {
                        ctx.props().on_cancel.emit(());
                    }
                }
            }
        }
        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div class={"crud-modal"}>
                <div class={"crud-modal-header"}>
                    <div class={"crud-modal-title"}>
                        {format!("Löschen - {}", ctx.props().entity.get_id())}
                        //TODO: Consider using an "EntryVisualizer" of some sort...
                    </div>
                </div>

                <div class={"crud-modal-body"}>
                    {"Bist du dir sicher?"}<br />
                    {"Dieser Eintrag kann nicht wiederhergestellt werden!"}
                </div>

                <div class={"crud-modal-footer"}>
                    <div class={"crud-row"}>
                    <div class={"crud-col crud-col-flex-end"}>
                        <CrudBtnWrapper>
                            <CrudBtn name={"Zurück"} variant={Variant::Default} onclick={&ctx.link().callback(|_| Msg::OnCancel)}/>
                            <CrudBtn name={"Löschen"} variant={Variant::Danger} onclick={&ctx.link().callback(|_| Msg::OnDelete)}/>
                        </CrudBtnWrapper>
                    </div>
                    </div>
                </div>
            </div>
        }
    }
}
