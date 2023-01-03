use tracing::info;
use yew::prelude::*;

use chrono_utc_date_time::UtcDateTime;

use super::prelude::*;

#[derive(Debug, Clone, PartialEq)]
pub struct Year {
    pub number: i32,
    pub is_now: bool,
    pub disabled: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Month {
    /// base 1
    pub index: u32,
    pub name: String,
    pub is_now: bool,
    pub disabled: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Week {
    pub days: Vec<Day>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Day {
    pub index: u32,
    pub display_name: String,
    pub in_previous_month: bool,
    pub in_current_month: bool,
    pub in_next_month: bool,
    pub utc_date_time: UtcDateTime,
    pub disabled: bool,
    pub highlighted: bool,
    pub selected: bool,
    pub is_now: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GuideMode {
    CalendarFirst,
    YearFirst,
}

impl Default for GuideMode {
    fn default() -> Self {
        Self::CalendarFirst
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Type {
    Date,
    Time,
    DateTime,
}

impl Default for Type {
    fn default() -> Self {
        Self::DateTime
    }
}

pub enum Msg {
    ToggleMenu,
    OpenMenu,
    CloseMenu,
    SetFocus(bool),
    KeyDown(KeyboardEvent),
    UpdateValue(UtcDateTime),
}

#[derive(Debug, Properties, PartialEq)]
pub struct Props {
    pub id: String,
    #[prop_or_default]
    pub value: Option<UtcDateTime>,
    #[prop_or_default]
    pub min: Option<UtcDateTime>,
    #[prop_or_default]
    pub max: Option<UtcDateTime>,
    #[prop_or_default]
    pub input_type: Type,
    #[prop_or_default]
    pub guide_mode: GuideMode,
    #[prop_or_default]
    pub placeholder: String,
    #[prop_or_default]
    pub disabled: bool,
    #[prop_or_default]
    pub onchange: Option<Callback<Option<UtcDateTime>>>,
    #[prop_or_default]
    pub onopen: Option<Callback<()>>,
    #[prop_or_default]
    pub onclose: Option<Callback<()>>,
}

pub struct CrudDatetime {
    initial_value: Option<UtcDateTime>,
    value: Option<UtcDateTime>,
    open: bool,
    in_focus: bool,
}

impl Component for CrudDatetime {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            initial_value: ctx.props().value.clone(),
            value: ctx.props().value.clone(),
            open: false,
            in_focus: false,
        }
    }

    fn changed(&mut self, ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        if self.initial_value != ctx.props().value {
            self.initial_value = ctx.props().value.clone();
            self.value = ctx.props().value.clone();
        }
        true
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ToggleMenu => {
                if !ctx.props().disabled {
                    match self.open {
                        true => ctx.link().send_message(Msg::CloseMenu),
                        false => ctx.link().send_message(Msg::OpenMenu),
                    }
                }
                true
            }
            Msg::OpenMenu => {
                if !self.open {
                    self.open = true;
                    if let Some(onopen) = &ctx.props().onopen {
                        onopen.emit(());
                    }
                    true
                } else {
                    false
                }
            }
            Msg::CloseMenu => {
                if self.open {
                    // Opening the menu puts the focus on the search input.
                    // On close, the focus on the select itself should be restored, as the user might still want to interact with it
                    // or want to tab further down the focusable elements on the site.
                    // If the menu is closed by pressing escape,
                    // the search input is still focused and the focus can be restored safely.
                    // If the menu is closed with a click outside the select menu and onto a focusable element, restoring focus
                    // to the select would probably be against the users intention / will.

                    // TODO: Implement this via js calls...
                    //if (!!this.input && !!this.input.nativeElement
                    //&& !!this.document.activeElement
                    //&& (this.document.activeElement === this.document.body
                    //    || this.document.activeElement === this.document.body.parentElement)) {
                    //this.input.nativeElement.focus();
                    //}

                    self.open = false;
                    if let Some(onclose) = &ctx.props().onclose {
                        onclose.emit(());
                    }
                    true
                } else {
                    false
                }
            }
            Msg::SetFocus(focus) => {
                self.in_focus = focus;
                false
            }
            Msg::KeyDown(event) => {
                if self.in_focus {
                    if !self.open
                        && (event.key().as_str() == "ArrowDown"
                            || event.key().as_str() == "Enter"
                            || event.key().as_str() == " ")
                    {
                        event.prevent_default();
                        ctx.link().send_message(Msg::OpenMenu);
                    } else if self.open
                        && (event.key().as_str() == "Escape" || event.key().as_str() == "Tab")
                    {
                        ctx.link().send_message(Msg::CloseMenu);
                    } else if event.key().as_str() == "Tab" {
                        // Do nothing.
                    } else {
                        event.prevent_default();
                        event.stop_propagation();
                    }
                }
                true
            }
            Msg::UpdateValue(value) => {
                info!("received new value {:?}", value);
                self.value = Some(value);
                if let Some(onchange) = &ctx.props().onchange {
                    onchange.emit(self.value.clone());
                }
                true
            },
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        //        Type::Date => rsx!{
        //            CrudDatetimeDateSelector {
        //                //selected: *selected.clone(),
        //                //(closeRequest)="closeMenu()"
        //                //(selection)="onDateSelection($event)"
        //                //[guideSelectionMode]="guideSelectionMode"
        //                //[maxDate]="maxDate"
        //                //[minDate]="minDate"
        //                //[sendCloseRequestOnDaySelection]="sendCloseRequestOnDaySelection ? sendCloseRequestOnDaySelection : true"
        //            }
        //        },
        //        Type::Time => rsx!{
        //            CrudDatetimeTimeSelector {
        //                //(closeRequest)="closeMenu()"
        //                //(selection)="onDateSelection($event)"
        //                //[currentlySelected]="selected"
        //            }
        //        },
        fn date_selector(this: &CrudDatetime, ctx: &Context<CrudDatetime>) -> Html {
            html! {
                <CrudDatetimeDateSelector
                    value={this.value.clone().unwrap_or_else(|| UtcDateTime::now())}
                    min={ctx.props().min.clone()}
                    max={ctx.props().max.clone()}
                    onchange={ctx.link().callback(Msg::UpdateValue)}
                    guide_mode={ctx.props().guide_mode}
                />
            }
        }

        fn time_selector(this: &CrudDatetime, _ctx: &Context<CrudDatetime>) -> Html {
            html! {
                <CrudDatetimeTimeSelector
                    value={this.value.clone().unwrap_or_else(|| UtcDateTime::now())}
                />
            }
        }

        html! {
            <div class={"crud-datetime"}>
                <input
                    type={"text"}
                    id={ctx.props().id.clone()}
                    class={classes!("crud-input-field", "datetime-selected", ctx.props().disabled.then(|| "disabled"))}
                    onclick={ctx.link().callback(|_| Msg::ToggleMenu)}
                    onfocusin={ctx.link().callback(|_| Msg::SetFocus(true))}
                    onfocusout={ctx.link().callback(|_| Msg::SetFocus(false))}
                    onkeydown={ctx.link().callback(|event| Msg::KeyDown(event))}
                    placeholder={ctx.props().placeholder.clone()}
                    value={self.value.clone().map(|it| it.to_rfc3339()).unwrap_or_default()}
                    tabindex={"0"}
                />
                <div class={"datetime-dropdown-menu-ref"}>
                    if self.open {
                        <div class={"datetime-dropdown-menu"}>
                            {
                                match ctx.props().input_type {
                                    Type::Date => date_selector(self, ctx),
                                    Type::Time => time_selector(self, ctx),
                                    Type::DateTime => html! {
                                        <>
                                            {date_selector(self, ctx)}
                                            {time_selector(self, ctx)}
                                        </>
                                    },
                                }
                            }
                        </div>
                    }
                </div>
            </div>
        }
    }
}
