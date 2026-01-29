use std::collections::HashMap;

use yew::html::;
use yewdux::prelude::*;

use crate::prelude::*;

#[derive(Clone)]
struct Link<T: 'static + CrudMainTrait>(<CrudInstance<T>>);

impl<T: CrudMainTrait> PartialEq for Link<T> {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}

#[derive(Default, Clone, PartialEq, Store)]
pub struct InstanceLinksStore<T: 'static + CrudMainTrait> {
    instance_links: HashMap<String, Link<T>>,
}

impl<T: CrudMainTrait> InstanceLinksStore<T> {
    pub fn get(&self, instance_name: &str) -> Option<<CrudInstance<T>>> {
        self.instance_links
            .get(instance_name)
            .cloned()
            .map(|link| link.0)
    }

    pub fn save(&mut self, instance_name: String, instance_link: Option<<CrudInstance<T>>>) {
        match instance_link {
            Some(link) => self.instance_links.insert(instance_name, Link(link)),
            None => self.instance_links.remove(&instance_name),
        };
    }
}
