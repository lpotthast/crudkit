// use yew::prelude::*;

use std::fmt::Debug;

//pub enum Msg<T> {
//    ToggleOptionsMenu,
//    OptionSelected(T),
//}

// TODO: add to prelude?, replace usage in derive.field-value macro!
#[derive(Debug, Clone, PartialEq)]
pub enum Selection<T> {
    None,
    Single(T),
    Multiple(Vec<T>),
}

// #[derive(Properties, PartialEq)]
// pub struct Props<T: Debug + Clone + PartialEq> {
//     // TODO: add option to forbid clearing (selecting None)
//     // TODO: rename to "selectable"
//     pub options: Vec<T>,
//     #[prop_or(None)]
//     pub option_renderer: Option<OptionRenderer<T>>,
//     pub selected: Selection<T>,
//     #[prop_or(true)]
//     pub close_options_menu_on_selection: bool,
//     pub selection_changed: Callback<Selection<T>>,
// }
// 
// /// Note: PartialEq is implemented by only comparing the name!
// /// If you want to update the renderer of a select, it is important to supply one with a different name.
// pub struct OptionRenderer<T> {
//     pub name: &'static str,
//     pub renderer: fn(&T) -> Html,
// }
// 
// impl<T> PartialEq for OptionRenderer<T> {
//     fn eq(&self, other: &Self) -> bool {
//         self.name == other.name
//     }
// }
// 
// pub struct CrudSelect<T: Debug + Clone + PartialEq> {
//     show_options: bool,
//     selected: Selection<T>,
// }
// 
// impl<T: 'static + Debug + Clone + PartialEq> CrudSelect<T> {
//     pub fn render_option(&self, ctx: &Context<Self>, option: &T) -> Html {
//         ctx.props()
//             .option_renderer
//             .as_ref()
//             .map(|it| (it.renderer)(option))
//             .unwrap_or_else(|| html! { format!("{option:?}") })
//     }
// }
// 
// impl<T: 'static + Debug + Clone + PartialEq> Component for CrudSelect<T> {
//     type Message = Msg<T>;
//     type Properties = Props<T>;
// 
//     fn create(ctx: &Context<Self>) -> Self {
//         Self {
//             show_options: false,
//             selected: ctx.props().selected.clone(),
//         }
//     }
// 
//     fn changed(&mut self, ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
//         self.selected = ctx.props().selected.clone();
//         true
//     }
// 
//     fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
//         match msg {
//             Msg::ToggleOptionsMenu => {
//                 self.show_options = !self.show_options;
//                 true
//             }
//             Msg::OptionSelected(option) => {
//                 let selected = Selection::Single(option);
//                 self.selected = selected.clone();
//                 if ctx.props().close_options_menu_on_selection {
//                     self.show_options = false;
//                 }
//                 ctx.props().selection_changed.emit(selected);
//                 true
//             }
//         }
//     }
// 
//     fn view(&self, ctx: &Context<Self>) -> Html {
//         html! {
//             <div class={"crud-select"}>
//                 <div class={"selected"} onclick={ctx.link().callback(move |_| Msg::ToggleOptionsMenu)}>
//                     {
//                         match &self.selected {
//                             Selection::None => html! { "" },
//                             Selection::Single(selected) => self.render_option(ctx, selected),
//                             Selection::Multiple(selected) => selected.iter()
//                                 .map(|it| html! {
//                                     <>
//                                         { self.render_option(ctx, it) }
//                                         {", "}
//                                     </>
//                                 })
//                                 .collect::<Html>(),
//                         }
//                     }
//                 </div>
//                 <div class={classes!("options", self.show_options.then(|| "shown"))}>
//                     { ctx.props().options.iter()
//                         .map(|option| {
//                             let cloned = option.clone();
//                             html! {
//                                 <div class={"option"} onclick={ctx.link().callback(move |_| Msg::OptionSelected(cloned.clone()))}>
//                                     { self.render_option(ctx, option) }
//                                 </div>
//                             }
//                         })
//                         .collect::<Html>() }
//                 </div>
//             </div>
//         }
//     }
// }
// 