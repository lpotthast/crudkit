use log::error;
use serde::Deserialize;
use wasm_bindgen::prelude::Closure;

mod js {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen(raw_module = "./js/tiptap.js")]
    extern "C" {
        pub fn create(
            id: String,
            content: String,
            editable: bool,
            onChange: &Closure<dyn Fn(String)>,
            onSelection: &Closure<dyn Fn()>,
        );
        pub fn isEditable(id: String) -> bool;
        pub fn getHTML(id: String) -> JsValue;
        pub fn toggleHeading(id: String, level: i32) -> JsValue;
        pub fn setParagraph(id: String) -> JsValue;
        pub fn toggleBold(id: String) -> JsValue;
        pub fn toggleItalic(id: String) -> JsValue;
        pub fn toggleStrike(id: String) -> JsValue;
        pub fn toggleBlockquote(id: String) -> JsValue;
        pub fn toggleHighlight(id: String) -> JsValue;
        pub fn setTextAlignLeft(id: String) -> JsValue;
        pub fn setTextAlignCenter(id: String) -> JsValue;
        pub fn setTextAlignRight(id: String) -> JsValue;
        pub fn setTextAlignJustify(id: String) -> JsValue;
        pub fn setImage(id: String, src: String, alt: String, title: String) -> JsValue;
        pub fn getState(id: String) -> JsValue;
    }
}

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
pub struct State {
    pub h1: bool,
    pub h2: bool,
    pub h3: bool,
    pub h4: bool,
    pub h5: bool,
    pub h6: bool,
    pub paragraph: bool,
    pub bold: bool,
    pub italic: bool,
    pub strike: bool,
    pub blockquote: bool,
    pub highlight: bool,
    pub align_left: bool,
    pub align_center: bool,
    pub align_right: bool,
    pub align_justify: bool,
}

pub fn create(
    id: String,
    content: String,
    editable: bool,
    on_change: &Closure<dyn Fn(String)>,
    on_selection: &Closure<dyn Fn()>,
) {
    js::create(id, content, editable, on_change, on_selection);
}

pub fn is_editable(id: String) -> bool {
    js::isEditable(id)
}

pub fn get_html(id: String) -> String {
    let value = js::getHTML(id);
    match value.as_string() {
        Some(string) => string,
        None => {
            error!(
                "JS function initKeycloak returned {:?}, which was not of the expected type: bool",
                value
            );
            "error".to_owned()
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeadingLevel {
    H1,
    H2,
    H3,
    H4,
    H5,
    H6,
}

impl Into<i32> for HeadingLevel {
    fn into(self) -> i32 {
        match self {
            HeadingLevel::H1 => 1,
            HeadingLevel::H2 => 2,
            HeadingLevel::H3 => 3,
            HeadingLevel::H4 => 4,
            HeadingLevel::H5 => 5,
            HeadingLevel::H6 => 6,
        }
    }
}

pub fn toggle_heading(id: String, level: HeadingLevel) {
    js::toggleHeading(id, level.into());
}

pub fn set_paragraph(id: String) {
    js::setParagraph(id);
}

pub fn toggle_bold(id: String) {
    js::toggleBold(id);
}

pub fn toggle_italic(id: String) {
    js::toggleItalic(id);
}

pub fn toggle_strike(id: String) {
    js::toggleStrike(id);
}

pub fn toggle_blockquote(id: String) {
    js::toggleBlockquote(id);
}

pub fn toggle_highlight(id: String) {
    js::toggleHighlight(id);
}

pub fn set_text_align_left(id: String) {
    js::setTextAlignLeft(id);
}

pub fn set_text_align_center(id: String) {
    js::setTextAlignCenter(id);
}

pub fn set_text_align_right(id: String) {
    js::setTextAlignRight(id);
}

pub fn set_text_align_justify(id: String) {
    js::setTextAlignJustify(id);
}

pub fn set_image(id: String, src: String, alt: String, title: String) {
    js::setImage(id, src, alt, title);
}

pub fn get_state(id: String) -> State {
    let state: State = js::getState(id).into_serde().unwrap();
    state
}
