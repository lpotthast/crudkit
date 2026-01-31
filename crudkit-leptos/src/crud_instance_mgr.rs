use crudkit_web::view::SerializableCrudView;
use leptos::prelude::*;

#[derive(Debug, Clone)]
pub struct InstanceState {
    pub name: &'static str,
    pub view: Signal<SerializableCrudView>,
}

#[derive(Debug, Clone, Copy)]
pub struct CrudInstanceMgrContext {
    instances: StoredValue<Vec<InstanceState>>,
}

impl CrudInstanceMgrContext {
    pub fn get_by_name(&self, name: &'static str) -> Option<InstanceState> {
        self.instances
            .read_value()
            .iter()
            .find(|instance| instance.name == name)
            .cloned()
    }

    pub fn register(&self, name: &'static str, instance: InstanceState) {
        self.instances.update_value(|instances| {
            match instances.iter_mut().find(|it| it.name == name) {
                Some(elem) => *elem = instance,
                None => instances.push(instance),
            }
        });
    }
}

/// Manages instances in a dynamic way. Must be rendered before any instance is rendered!
#[component]
pub fn CrudInstanceMgr(children: Children) -> impl IntoView {
    let states = StoredValue::new(Vec::<InstanceState>::new());
    provide_context(CrudInstanceMgrContext { instances: states });
    children()
}
