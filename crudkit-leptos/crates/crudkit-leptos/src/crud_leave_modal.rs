use std::rc::Rc;

use yew::prelude::*;
use yewdux::prelude::Dispatch;

use crate::prelude::*;
use crate::stores::global_key_up::GlobalKeyUp;

pub enum Msg {
    OnCancel,
    OnLeave,
    GlobalKeyUp(Rc<GlobalKeyUp>),
}

#[derive(Properties, PartialEq)]
pub struct Props {
    pub on_cancel: Callback<()>,
    pub on_leave: Callback<()>,
}

pub struct CrudLeaveModal {
    _global_key_up_dispatch: Dispatch<GlobalKeyUp>,
}

impl Component for CrudLeaveModal {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            _global_key_up_dispatch: Dispatch::subscribe(ctx.link().callback(Msg::GlobalKeyUp)),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::OnCancel => {
                ctx.props().on_cancel.emit(());
            }
            Msg::OnLeave => {
                ctx.props().on_leave.emit(());
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
                        {"Ungespeicherte Änderungen"}
                    </div>
                </div>

                <div class={"crud-modal-body"} style={"text-align: center;"}>
                    {"Du hast deine Änderungen noch nicht gespeichert."}<br/>
                    {"Möchtest du den Bereich wirklich verlassen?"}<br/>
                    {"Ungespeicherte Änderungen gehen verloren!"}
                </div>

                <div class={"crud-modal-footer"}>
                    <div class={"crud-row"}>
                        <div class={"crud-col crud-col-flex-end"}>
                            <CrudBtnWrapper>
                                <CrudBtn name={"Zurück"} variant={Variant::Default} onclick={&ctx.link().callback(|_| Msg::OnCancel)}/>
                                <CrudBtn name={"Verlassen"} variant={Variant::Warn} onclick={&ctx.link().callback(|_| Msg::OnLeave)}/>
                            </CrudBtnWrapper>
                        </div>
                    </div>
                </div>
            </div>
        }
    }
}
