use crudkit_core::id::SerializableId;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CrudView<ReadId, UpdateId>
where
    ReadId: crudkit_core::id::Id + Serialize + DeserializeOwned,
    UpdateId: crudkit_core::id::Id + Serialize + DeserializeOwned,
{
    #[default]
    List,
    Create,
    #[serde(bound = "")]
    Read(ReadId),
    #[serde(bound = "")]
    Edit(UpdateId),
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SerializableCrudView {
    #[default]
    List,
    Create,
    #[serde(bound = "")]
    Read(SerializableId),
    #[serde(bound = "")]
    Edit(SerializableId),
}

impl<ReadId, UpdateId> From<CrudView<ReadId, UpdateId>> for SerializableCrudView
where
    ReadId: crudkit_core::id::Id + Serialize + DeserializeOwned,
    UpdateId: crudkit_core::id::Id + Serialize + DeserializeOwned,
{
    fn from(value: CrudView<ReadId, UpdateId>) -> Self {
        match value {
            CrudView::List => Self::List,
            CrudView::Create => Self::Create,
            CrudView::Read(id) => Self::Read(id.to_serializable_id()),
            CrudView::Edit(id) => Self::Edit(id.to_serializable_id()),
        }
    }
}
