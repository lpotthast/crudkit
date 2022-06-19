use std::collections::HashMap;

use yew::html::Scope;
use yewdux::prelude::*;

use crate::{CrudDataTrait, prelude::CrudInstance};

#[derive(Clone)]
struct Link<T: 'static + CrudDataTrait>(Scope<CrudInstance<T>>);

impl<T: CrudDataTrait> PartialEq for Link<T> {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}

#[derive(Default, Clone, PartialEq)]
pub struct InstanceLinksStore<T: 'static + CrudDataTrait> {
    instance_links: HashMap<String, Link<T>>,
}

impl<T: CrudDataTrait> InstanceLinksStore<T> {
    pub fn get(&self, instance_name: &str) -> Option<Scope<CrudInstance<T>>> {
        self.instance_links.get(instance_name).cloned().map(|link| link.0)
    }

    pub fn save(&mut self, instance_name: String, instance_link: Option<Scope<CrudInstance<T>>>) {
        match instance_link {
            Some(link) => self.instance_links.insert(instance_name, Link(link)),
            None => self.instance_links.remove(&instance_name),
        };
    }
}

impl<T: 'static + CrudDataTrait> Store for InstanceLinksStore<T> {
    fn new() -> Self {
        InstanceLinksStore::default()
    }

    fn changed(&mut self) {
        // not storable...
    }
}
