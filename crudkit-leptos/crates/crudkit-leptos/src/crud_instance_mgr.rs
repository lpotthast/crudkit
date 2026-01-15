use crudkit_web::view::SerializableCrudView;
use leptos::prelude::*;
use std::ops::Deref;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct InstanceState {
    pub name: &'static str,
    pub view: Signal<SerializableCrudView>,
}

#[derive(Debug, Clone)]
pub struct InstanceStates {
    states: Arc<Mutex<Vec<InstanceState>>>,
}

impl Default for InstanceStates {
    fn default() -> Self {
        Self {
            states: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl InstanceStates {
    pub fn get_by_name(&self, name: &'static str) -> Option<InstanceState> {
        self.states
            .lock()
            .unwrap()
            .deref()
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
            let mut states = instances.states.lock().unwrap();
            match states.iter_mut().find(|it| it.name == name) {
                Some(elem) => *elem = state,
                None => states.push(state),
            }
        })
    }
}

/// Manages instances in a dynamic way. Must be rendered before any instance is rendered!
#[component]
pub fn CrudInstanceMgr(children: Children) -> impl IntoView {
    let (instances, set_instances) = signal(InstanceStates::default());
    provide_context(CrudInstanceMgrContext {
        instances,
        set_instances,
    });
    children()
}
