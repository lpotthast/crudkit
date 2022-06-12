use log::error;
use wasm_bindgen::{prelude::Closure};

mod js {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen(raw_module = "./res/tiptap.js")]
    extern "C" {
        pub fn create(id: String, content: String, editable: bool, onchange: &Closure<dyn Fn(String)>);
        pub fn isEditable(id: String) -> bool;
        pub fn getHTML(id: String) -> JsValue;
        pub fn toggleBold(id: String) -> JsValue;
        pub fn toggleItalic(id: String) -> JsValue;
        pub fn toggleStrike(id: String) -> JsValue;
        pub fn toggleBlockquote(id: String) -> JsValue;
        pub fn setImage(id: String, src: String, alt: String, title: String) -> JsValue;
    }
}

pub fn create(id: String, content: String, editable: bool, onchange: &Closure<dyn Fn(String)>) {
    js::create(id, content, editable, onchange);
}

pub fn is_editable(id: String) -> bool {
    js::isEditable(id)
}

pub fn get_html(id: String) -> String {
    let value = js::getHTML(id);
    match value.as_string() {
        Some(string) => string,
        None => {
            error!("JS function initKeycloak returned {:?}, which was not of the expected type: bool", value);
            "error".to_owned()
        },
    }
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

pub fn set_image(id: String, src: String, alt: String, title: String) {
    js::setImage(id, src, alt, title);
}
