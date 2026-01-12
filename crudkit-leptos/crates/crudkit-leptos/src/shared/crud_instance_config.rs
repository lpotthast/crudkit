use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ItemsPerPage(pub u64);

impl Default for ItemsPerPage {
    fn default() -> Self {
        Self(10)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct PageNr(pub u64); // TODO: NonZero type?

impl Default for PageNr {
    fn default() -> Self {
        Self::first()
    }
}

impl PageNr {
    pub fn first() -> Self {
        Self(1)
    }
}
