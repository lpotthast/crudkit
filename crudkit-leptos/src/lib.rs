#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]
#![deny(missing_docs)]
#![recursion_limit = "512"]

#![doc = include_str!("../docs/guide.md")]

pub mod atoms;
pub mod config;
pub mod hooks;
pub mod instance;
#[cfg(all(test, not(target_family = "wasm")))]
mod test_support;

// Re-exported, so that applications can `use crudkit_leptos::prelude::*` and use CrudKit's derive
// macros, whose generated code refers to these crates, without depending on them directly.
pub use crudkit_core;
pub use crudkit_web;

/// Defines the `clubs` and `players` resources used by the examples of the crate documentation.
/// Not part of the API.
#[doc(hidden)]
#[macro_export]
macro_rules! __doc_example {
    () => {
        use ::leptos::prelude::*;
        use ::serde::{Deserialize, Serialize};
        use $crate::prelude::*;

        #[derive(
            Clone, PartialEq, Eq, Debug, CkId, CkField, CkResource, Serialize, Deserialize,
        )]
        #[ck_resource(resource_name = "clubs")]
        #[ck_field(model = Update)]
        pub struct Club {
            pub id: i64,
            pub name: String,
            #[serde(skip_deserializing)]
            pub players: (),
        }

        #[derive(Clone, PartialEq, Eq, Debug, Default, CkField, Serialize, Deserialize)]
        #[ck_field(model = Create)]
        pub struct CreateClub {
            pub name: String,
        }

        impl ErasedIdentifiable for CreateClub {
            fn id(&self) -> SerializableId {
                unreachable!("create models have no id")
            }
        }

        #[derive(Clone, PartialEq, Eq, Debug, CkId, CkField, Serialize, Deserialize)]
        #[ck_field(model = Read)]
        pub struct ReadClub {
            pub id: i64,
            pub name: String,
        }

        impl From<ReadClub> for Club {
            fn from(read: ReadClub) -> Self {
                Self {
                    id: read.id,
                    name: read.name,
                    players: (),
                }
            }
        }

        fn club_config() -> CrudInstanceConfig {
            CrudInstanceConfig::new::<CrudClubResource>(
                "/api",
                ::std::sync::Arc::new($crate::crudkit_web::http::NewClientPerRequestExecutor),
            )
        }

        #[derive(
            Clone, PartialEq, Eq, Debug, CkId, CkField, CkResource, Serialize, Deserialize,
        )]
        #[ck_resource(resource_name = "players")]
        #[ck_field(model = Update)]
        pub struct Player {
            pub id: i64,
            pub name: String,
            pub club_id: i64,
        }

        #[derive(Clone, PartialEq, Eq, Debug, Default, CkField, Serialize, Deserialize)]
        #[ck_field(model = Create)]
        pub struct CreatePlayer {
            pub name: String,
            pub club_id: i64,
        }

        impl ErasedIdentifiable for CreatePlayer {
            fn id(&self) -> SerializableId {
                unreachable!("create models have no id")
            }
        }

        #[derive(Clone, PartialEq, Eq, Debug, CkId, CkField, Serialize, Deserialize)]
        #[ck_field(model = Read)]
        pub struct ReadPlayer {
            pub id: i64,
            pub name: String,
            pub club_id: i64,
        }

        impl From<ReadPlayer> for Player {
            fn from(read: ReadPlayer) -> Self {
                Self {
                    id: read.id,
                    name: read.name,
                    club_id: read.club_id,
                }
            }
        }

        fn player_config() -> CrudInstanceConfig {
            CrudInstanceConfig::new::<CrudPlayerResource>(
                "/api",
                ::std::sync::Arc::new($crate::crudkit_web::http::NewClientPerRequestExecutor),
            )
        }
    };
}

/// Common imports of CrudKit applications: the shared contracts, derive macros, and the public
/// types, hooks, and atoms of this crate.
pub mod prelude {
    pub use crudkit_core;
    pub use crudkit_core::collaboration;
    pub use crudkit_core::condition;
    pub use crudkit_core::id;
    pub use crudkit_core::id::*;
    pub use crudkit_core::validation;
    pub use crudkit_core::*;
    pub use crudkit_web;
    pub use crudkit_web::prelude::*;

    // Explicitly re-export Model from crudkit_web to resolve ambiguity
    // (both crudkit_core and crudkit_web export Model).
    pub use crudkit_web::model::Model;

    pub use crudkit_core_macros::CkId;
    pub use crudkit_web_macros::{CkActionPayload, CkField, CkResource};

    pub use crudkit_web::http::ReqwestExecutor;
    pub use crudkit_web::view::{CREATE_VIEW, EDIT_VIEW, READ_VIEW, TABLE_VIEW};

    pub use super::atoms::*;
    pub use super::config::*;
    pub use super::hooks::*;
    pub use super::instance::*;
}
