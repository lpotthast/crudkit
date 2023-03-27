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
        #[ck_resource(
            create_model = asv_api::news::CreateModel,
            model = asv_api::news::Model,
            active_model = asv_api::news::ActiveModel,
        )]
        #[serde(rename = "news")]
        News,

        #[ck_resource(
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
can be added like this:

    SELECT N.*, CASE WHEN bool.entity_id IS NOT NULL THEN true ELSE false END AS has_validation_errors
    FROM "News" as N
    LEFT OUTER JOIN
    (
        SELECT VR.entity_name, VR.entity_id
        FROM "ValidationResults" AS VR
        WHERE VR.entity_name = 'news'
    ) bool ON bool.entity_id = N.id;

A view with this additional column can be created like this:

    DROP VIEW IF EXISTS "NewsReadView";

    CREATE VIEW "NewsReadView" AS
    SELECT N.*, CASE WHEN bool.entity_id IS NOT NULL THEN true ELSE false END AS has_validation_errors
    FROM "News" as N
    LEFT OUTER JOIN
    (
        SELECT VR.entity_name, VR.entity_id
        FROM "ValidationResults" AS VR
        WHERE VR.entity_name = 'news'
    ) bool ON bool.entity_id = N.id;
