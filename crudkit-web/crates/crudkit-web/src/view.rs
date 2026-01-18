use crudkit_id::SerializableId;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CrudView<ReadId, UpdateId>
where
    ReadId: crudkit_id::Id + Serialize + DeserializeOwned,
    UpdateId: crudkit_id::Id + Serialize + DeserializeOwned,
{
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

impl<ReadId, UpdateId> Into<SerializableCrudView> for CrudView<ReadId, UpdateId>
where
    ReadId: crudkit_id::Id + Serialize + DeserializeOwned,
    UpdateId: crudkit_id::Id + Serialize + DeserializeOwned,
{
    fn into(self) -> SerializableCrudView {
        match self {
            CrudView::List => SerializableCrudView::List,
            CrudView::Create => SerializableCrudView::Create,
            CrudView::Read(id) => SerializableCrudView::Read(id.to_serializable_id()),
            CrudView::Edit(id) => SerializableCrudView::Edit(id.to_serializable_id()),
        }
    }
}

impl<ReadId, UpdateId> Default for CrudView<ReadId, UpdateId>
where
    ReadId: crudkit_id::Id + Serialize + DeserializeOwned,
    UpdateId: crudkit_id::Id + Serialize + DeserializeOwned,
{
    fn default() -> Self {
        Self::List
    }
}
