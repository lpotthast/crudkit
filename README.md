# TBD


LIFECYCLE:

User input
beforeSendCreate

beforeCreate
create (BE)
afterCreate

User input
beforeSendUpdate

update (BE)

delete




#[derive(Debug, Deserialize, DeriveCrudResources)]
pub enum Resource {
    #[crud(
        create_model = asv_api::news::CreateModel,
        model = asv_api::news::Model,
        active_model = asv_api::news::ActiveModel,
    )]
    #[serde(rename = "news")]
    News,

    #[crud(
        create_model = asv_api::news::CreateModel,
        model = asv_api::news::Model,
        active_model = asv_api::news::ActiveModel,
    )]
    #[serde(rename = "courses")]
    Courses
}

=> and we should get CRUD operations and event handling for free!

