use std::{cell::RefCell, rc::Rc};

use crudkit_web::SerializableCrudView;
use leptos::*;

#[derive(Debug, Clone)]
pub struct InstanceState {
    pub name: &'static str,
    pub view: Signal<SerializableCrudView>,
}

#[derive(Debug, Clone)]
pub struct InstanceStates {
    states: Rc<RefCell<Vec<InstanceState>>>,
}

impl Default for InstanceStates {
    fn default() -> Self {
        Self {
            states: Rc::new(RefCell::new(Vec::new())),
        }
    }
}

impl InstanceStates {
    pub fn get_by_name(&self, name: &'static str) -> Option<InstanceState> {
        self.states
            .borrow()
            .iter()
            .find(|state| state.name == name)
            .cloned()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CrudInstanceMgrContext {
    pub instances: ReadSignal<InstanceStates>,
    set_instances: WriteSignal<InstanceStates>,
}

impl CrudInstanceMgrContext {
    /// Panics when a state is already registered for this name!
    pub fn register(&self, name: &'static str, state: InstanceState) {
        self.set_instances.update(|instances| {
            let mut states = instances.states.borrow_mut();
            match states.iter_mut().find(|it| it.name == name) {
                Some(elem) => *elem = state,
                None => states.push(state),
            }
        })
    }
}

/// Manages instances in a dynamic way. Must be rendered before any instance is rendered!
#[component]
pub fn CrudInstanceMgr(cx: Scope, children: Children) -> impl IntoView {
    let (instances, set_instances) = create_signal(cx, InstanceStates::default());
    provide_context(
        cx,
        CrudInstanceMgrContext {
            instances,
            set_instances,
        },
    );

    children(cx)
}
