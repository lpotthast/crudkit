use yew::prelude::*;

use std::fmt::Debug;

pub enum Msg<T> {
    ToggleOptionsMenu,
    OptionSelected(T),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Selection<T> {
    None,
    Single(T),
    Multiple(Vec<T>),
}

#[derive(Properties, PartialEq)]
pub struct Props<T: Debug + Clone + PartialEq> {
    pub options: Vec<T>,
    pub selected: Selection<T>,
    #[prop_or(true)]
    pub close_options_menu_on_selection: bool,
    pub selection_changed: Callback<Selection<T>>,
}

pub struct CrudSelect<T: Debug + Clone + PartialEq> {
    show_options: bool,
    selected: Selection<T>,
}

impl<T: 'static + Debug + Clone + PartialEq> Component for CrudSelect<T> {
    type Message = Msg<T>;
    type Properties = Props<T>;

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            show_options: false,
            selected: ctx.props().selected.clone(),
        }
    }
    
    fn changed(&mut self, ctx: &Context<Self>) -> bool {
        self.selected = ctx.props().selected.clone();
        true
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ToggleOptionsMenu => {
                self.show_options = !self.show_options;
                true
            }
            Msg::OptionSelected(option) => {
                let selected = Selection::Single(option);
                self.selected = selected.clone();
                if ctx.props().close_options_menu_on_selection {
                    self.show_options = false;
                }
                ctx.props().selection_changed.emit(selected);
                true
            },
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div class={"crud-select"}>
                <div class={"selected"} onclick={ctx.link().callback(move |_| Msg::ToggleOptionsMenu)}>
                    {format!("{:?}", self.selected)}
                </div>
                <div class={classes!("options", self.show_options.then(|| "shown"))}>
                    { ctx.props().options.iter()
                        .map(|option| {
                            let cloned = option.clone();
                            html! {
                                <div class={"option"} onclick={ctx.link().callback(move |_| Msg::OptionSelected(cloned.clone()))}>
                                    {format!("{:?},\n", option)}
                                </div>
                            }
                        })
                        .collect::<Html>() }
                </div>
            </div>
        }
    }
}
