use crate::{prelude::*, types::files::FileResource};
use wasm_bindgen::prelude::Closure;
use yew::prelude::*;
use yewbi::Bi;

use super::js_tiptap;

pub enum Msg {
    Changed(String),
    Bold,
    Italic,
    Strike,
    Blockquote,
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
    changed: Closure<dyn Fn(String)>,
    choose_image: bool,
}

impl Component for CrudTipTapEditor {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let yew_callback = ctx.link().callback(|text| Msg::Changed(text));
        let changed =
            Closure::wrap(Box::new(move |text| yew_callback.emit(text)) as Box<dyn Fn(String)>);
        Self {
            changed,
            choose_image: false,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Changed(text) => {
                if let Some(onchange) = &ctx.props().onchange {
                    onchange.emit(text);
                }
                false
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

                    <div class={"tiptap-btn"} onclick={ctx.link().callback(|_| Msg::Bold)}>
                        <CrudIcon variant={Bi::TypeBold}/>
                        {"bold"}
                    </div>

                    <div class={"tiptap-btn"} onclick={ctx.link().callback(|_| Msg::Italic)}>
                        <CrudIcon variant={Bi::TypeItalic}/>
                        {"italic"}
                    </div>

                    <div class={"tiptap-btn"} onclick={ctx.link().callback(|_| Msg::Strike)}>
                        <CrudIcon variant={Bi::TypeStrikethrough}/>
                        {"strike"}
                    </div>

                    <div class={"tiptap-btn"} onclick={ctx.link().callback(|_| Msg::Blockquote)}>
                        <CrudIcon variant={Bi::BlockquoteLeft}/>
                        {"blockquote"}
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
                &self.changed,
            );
        }
    }
}
