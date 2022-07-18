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



## Add validation_status column to all reads

Assuming that we have an entity/table called "News"
And validation results stored in table "ValidationResults,
a boolean column, indicating whether or not validation errors exist for each entry,
can be added like this 

    SELECT N.*, CASE WHEN bool.entity_id IS NOT NULL THEN 1 ELSE 0 END AS validation_status
    FROM "News" as N
    LEFT OUTER JOIN 
    (
        SELECT VR.entity_name, VR.entity_id
        FROM "ValidationResults" AS VR
        WHERE VR.entity_name = 'news'
    ) bool ON bool.entity_id = N.id
    ORDER BY validation_status DESC
    LIMIT 100;