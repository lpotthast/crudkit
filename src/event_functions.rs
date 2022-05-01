use wasm_bindgen::JsCast;
use web_sys::Element;
use yew::prelude::*;

pub trait EventFunctions {
    fn has_target_with_class(&self, class_name: &str) -> bool;
}

pub trait MouseEventFunctions {
    fn comp_rel_pos_percent_target_parent_with_class(&self, class: &str) -> Option<(f64, f64)>;
    fn comp_rel_pos_percent_target_element_with_id<S: AsRef<str>>(
        &self,
        id_query: S,
    ) -> Option<(f64, f64)>;
}

impl EventFunctions for Event {
    fn has_target_with_class(&self, class_name: &str) -> bool {
        let optional_target = self
            .target()
            .and_then(|event_target: web_sys::EventTarget| {
                event_target.dyn_into::<web_sys::Element>().ok()
            });
        if let Some(target) = optional_target {
            target.class_list().contains(class_name)
        } else {
            false
        }
    }
}

impl MouseEventFunctions for MouseEvent {
    fn comp_rel_pos_percent_target_parent_with_class(&self, class: &str) -> Option<(f64, f64)> {
        let optional_target = self
            .target()
            .and_then(|event_target: web_sys::EventTarget| {
                event_target.dyn_into::<web_sys::Element>().ok()
            });
        if let Some(mut target) = optional_target {
            while !target.class_list().contains(class) {
                match target.parent_element() {
                    Some(parent) => target = parent,
                    None => break,
                }
            }
            assert!(target.class_list().contains(class));
            Some(comp_rel_pos_percent_target_element(
                self.client_x() as f64,
                self.client_y() as f64,
                target,
            ))
        } else {
            None
        }
    }

    fn comp_rel_pos_percent_target_element_with_id<S: AsRef<str>>(
        &self,
        id: S,
    ) -> Option<(f64, f64)> {
        web_sys::window()
            .and_then(|window| window.document())
            .and_then(|document| document.get_element_by_id(id.as_ref()))
            .map(|target| {
                comp_rel_pos_percent_target_element(
                    self.client_x() as f64,
                    self.client_y() as f64,
                    target,
                )
            })
    }
}

fn comp_rel_pos_percent_target_element(
    client_x: f64,
    client_y: f64,
    target: Element,
) -> (f64, f64) {
    let rect = target.get_bounding_client_rect();
    let x = client_x - rect.left();
    let y = client_y - rect.top();
    // Using custom x,y instead of event.offset_x/y,
    // because event.offset was computed for the direct target, which must not be the target we got now.
    let mut px = x / rect.width();
    let mut py = y / rect.height();
    px = f64::max(0.0, f64::min(1.0, px));
    py = f64::max(0.0, f64::min(1.0, py));
    (px, py)
}
