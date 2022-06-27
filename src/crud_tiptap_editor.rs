use crate::{prelude::*, types::files::FileResource, js_tiptap::State};
use wasm_bindgen::prelude::Closure;
use yew::prelude::*;
use yewbi::Bi;

use super::js_tiptap;

pub enum Msg {
    SelectionChanged,
    Changed(String),
    H1,
    H2,
    H3,
    Paragraph,
    Bold,
    Italic,
    Strike,
    Blockquote,
    Highlight,
    AlignLeft,
    AlignCenter,
    AlignRight,
    AlignJustify,
    SetImage,
    ChooseImageCanceled,
    ImageChosen(FileResource),
}

#[derive(Properties, PartialEq)]
pub struct Props {
    pub api_base_url: String,
    pub id: String,
    pub value: String,
    pub class: String,
    pub disabled: bool,
    pub onchange: Option<Callback<String>>,
}

//Msg::Extract => {
//    self.extracted = js_tiptap::get_html("test".to_owned());
//    true
//},

pub struct CrudTipTapEditor {
    on_change: Closure<dyn Fn(String)>,
    on_selection: Closure<dyn Fn()>,
    choose_image: bool,
    state: State,
}

impl Component for CrudTipTapEditor {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let yew_callback = ctx.link().callback(|text| Msg::Changed(text));
        let changed =
            Closure::wrap(Box::new(move |text| yew_callback.emit(text)) as Box<dyn Fn(String)>);

        let yew_callback = ctx.link().callback(|_| Msg::SelectionChanged);
        let selected =
            Closure::wrap(Box::new(move || yew_callback.emit(())) as Box<dyn Fn()>);
        Self {
            on_change: changed,
            on_selection: selected,
            choose_image: false,
            state: Default::default(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::SelectionChanged => {
                self.state = js_tiptap::get_state(ctx.props().id.clone());
                true
            }
            Msg::Changed(text) => {
                if let Some(onchange) = &ctx.props().onchange {
                    onchange.emit(text);
                }
                self.state = js_tiptap::get_state(ctx.props().id.clone());
                true
            }
            Msg::H1 => {
                js_tiptap::toggle_heading(ctx.props().id.clone(), js_tiptap::HeadingLevel::H1);
                true
            }
            Msg::H2 => {
                js_tiptap::toggle_heading(ctx.props().id.clone(), js_tiptap::HeadingLevel::H2);
                true
            }
            Msg::H3 => {
                js_tiptap::toggle_heading(ctx.props().id.clone(), js_tiptap::HeadingLevel::H3);
                true
            }
            Msg::Paragraph => {
                js_tiptap::set_paragraph(ctx.props().id.clone());
                true
            }
            Msg::Bold => {
                js_tiptap::toggle_bold(ctx.props().id.clone());
                true
            }
            Msg::Italic => {
                js_tiptap::toggle_italic(ctx.props().id.clone());
                true
            }
            Msg::Strike => {
                js_tiptap::toggle_strike(ctx.props().id.clone());
                true
            }
            Msg::Blockquote => {
                js_tiptap::toggle_blockquote(ctx.props().id.clone());
                true
            }
            Msg::Highlight => {
                js_tiptap::toggle_highlight(ctx.props().id.clone());
                true
            }
            Msg::AlignLeft => {
                js_tiptap::set_text_align_left(ctx.props().id.clone());
                true
            }
            Msg::AlignCenter => {
                js_tiptap::set_text_align_center(ctx.props().id.clone());
                true
            }
            Msg::AlignRight => {
                js_tiptap::set_text_align_right(ctx.props().id.clone());
                true
            }
            Msg::AlignJustify => {
                js_tiptap::set_text_align_justify(ctx.props().id.clone());
                true
            }
            Msg::SetImage => {
                // Enables the chooser modal!
                self.choose_image = true;
                true
            }
            Msg::ChooseImageCanceled => {
                self.choose_image = false;
                true
            }
            Msg::ImageChosen(resource) => {
                self.choose_image = false;
                js_tiptap::set_image(
                    ctx.props().id.clone(),
                    // TODO: Consolidate /public resource url creations in one place...
                    format!(
                        "{}/public/{}",
                        ctx.props().api_base_url,
                        urlencoding::encode(resource.name.as_str())
                    ),
                    // TODO: Define proper
                    resource.name.clone(),
                    resource.name.clone(),
                );
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div class={classes!("tiptap-editor", ctx.props().disabled.then(|| "disabled"))}>

                <div class={"tiptap-menu"}>

                    <div class={classes!("tiptap-btn", self.state.h1.then(|| "active"))} onclick={ctx.link().callback(|_| Msg::H1)}>
                        <CrudIcon variant={Bi::TypeH1}/>
                        {"h1"}
                    </div>

                    <div class={classes!("tiptap-btn", self.state.h2.then(|| "active"))} onclick={ctx.link().callback(|_| Msg::H2)}>
                        <CrudIcon variant={Bi::TypeH2}/>
                        {"h2"}
                    </div>

                    <div class={classes!("tiptap-btn", self.state.h3.then(|| "active"))} onclick={ctx.link().callback(|_| Msg::H3)}>
                        <CrudIcon variant={Bi::TypeH3}/>
                        {"h3"}
                    </div>

                    <div class={classes!("tiptap-btn", self.state.paragraph.then(|| "active"))} onclick={ctx.link().callback(|_| Msg::Paragraph)}>
                        <CrudIcon variant={Bi::Paragraph}/>
                        {"paragraph"}
                    </div>

                    <div class={classes!("tiptap-btn", self.state.bold.then(|| "active"))} onclick={ctx.link().callback(|_| Msg::Bold)}>
                        <CrudIcon variant={Bi::TypeBold}/>
                        {"bold"}
                    </div>

                    <div class={classes!("tiptap-btn", self.state.italic.then(|| "active"))} onclick={ctx.link().callback(|_| Msg::Italic)}>
                        <CrudIcon variant={Bi::TypeItalic}/>
                        {"italic"}
                    </div>

                    <div class={classes!("tiptap-btn", self.state.strike.then(|| "active"))} onclick={ctx.link().callback(|_| Msg::Strike)}>
                        <CrudIcon variant={Bi::TypeStrikethrough}/>
                        {"strike"}
                    </div>

                    <div class={classes!("tiptap-btn", self.state.blockquote.then(|| "active"))} onclick={ctx.link().callback(|_| Msg::Blockquote)}>
                        <CrudIcon variant={Bi::BlockquoteLeft}/>
                        {"blockquote"}
                    </div>

                    <div class={classes!("tiptap-btn", self.state.highlight.then(|| "active"))} onclick={ctx.link().callback(|_| Msg::Highlight)}>
                        <CrudIcon variant={Bi::BrightnessAltHigh}/>
                        {"highlight"}
                    </div>

                    <div class={classes!("tiptap-btn", self.state.align_left.then(|| "active"))} onclick={ctx.link().callback(|_| Msg::AlignLeft)}>
                        <CrudIcon variant={Bi::TextLeft}/>
                        {"left"}
                    </div>
                    
                    <div class={classes!("tiptap-btn", self.state.align_center.then(|| "active"))} onclick={ctx.link().callback(|_| Msg::AlignCenter)}>
                        <CrudIcon variant={Bi::TextCenter}/>
                        {"center"}
                    </div>
                    
                    <div class={classes!("tiptap-btn", self.state.align_right.then(|| "active"))} onclick={ctx.link().callback(|_| Msg::AlignRight)}>
                        <CrudIcon variant={Bi::TextRight}/>
                        {"right"}
                    </div>

                    <div class={classes!("tiptap-btn", self.state.align_justify.then(|| "active"))} onclick={ctx.link().callback(|_| Msg::AlignJustify)}>
                        <CrudIcon variant={Bi::Justify}/>
                        {"justify"}
                    </div>

                    <div class={"tiptap-btn"} onclick={ctx.link().callback(|_| Msg::SetImage)}>
                        <CrudIcon variant={Bi::Image}/>
                        {"image"}
                    </div>

                </div>

                // This is our TipTap instance!
                <div id={ctx.props().id.clone()} class={"tiptap-instance"}></div>

                {
                    match &self.choose_image {
                        true => html! {
                            <CrudModal>
                                <CrudImageChooserModal
                                    api_base_url={ctx.props().api_base_url.clone()}
                                    on_cancel={ctx.link().callback(|_| Msg::ChooseImageCanceled)}
                                    on_choose={ctx.link().callback(|resource| Msg::ImageChosen(resource))}
                                />
                            </CrudModal>
                        },
                        false => html! {}
                    }
                }
            </div>
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            js_tiptap::create(
                ctx.props().id.clone(),
                ctx.props().value.clone(),
                !ctx.props().disabled,
                &self.on_change,
                &self.on_selection,
            );
        }
    }
}
