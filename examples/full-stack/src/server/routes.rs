//! The CrudKit REST API of this example.

use crate::server::collaboration::NoopCollaborationService;
use crate::server::resources::{
    ClubResource, ClubResourceContext, PersonResource, PersonResourceContext,
};
use crate::server::validation::PersonGenderValidator;
use axum::{Extension, Router};
use crudkit_rs::impl_add_crud_routes;
use crudkit_rs::prelude::CrudContext;
use crudkit_rs::validate::GlobalValidationState;
use crudkit_sea_orm::repo::SeaOrmRepo;
use crudkit_sea_orm::validation::unified::repository::UnifiedValidationRepository;
use sea_orm::DatabaseConnection;
use std::sync::Arc;

/// URL prefix of all API routes.
pub const API_ROOT: &str = "/api";

impl_add_crud_routes!(crate::server::resources::ClubResource, club);
impl_add_crud_routes!(crate::server::resources::PersonResource, person);

/// Creates the router serving `POST /api/{resource}/crud/v1/{operation}` for every resource.
pub fn api_router(db: Arc<DatabaseConnection>) -> Router {
    let repository = Arc::new(SeaOrmRepo::new(db.clone()));
    let validation_result_repository = Arc::new(UnifiedValidationRepository { db });
    let collab_service = Arc::new(NoopCollaborationService);

    let club_context = Arc::new(CrudContext::<ClubResource> {
        res_context: Arc::new(ClubResourceContext),
        repository: repository.clone(),
        validators: vec![],
        resource_validators: vec![],
        validation_result_repository: validation_result_repository.clone(),
        collab_service: collab_service.clone(),
        global_validation_state: Arc::new(GlobalValidationState::new()),
    });

    let person_context = Arc::new(CrudContext::<PersonResource> {
        res_context: Arc::new(PersonResourceContext),
        repository,
        validators: vec![Arc::new(PersonGenderValidator)],
        resource_validators: vec![],
        validation_result_repository,
        collab_service,
        global_validation_state: Arc::new(GlobalValidationState::new()),
    });

    let mut router = Router::new();
    router = axum_club_crud_routes::add_crud_routes(API_ROOT, router);
    router = axum_person_crud_routes::add_crud_routes(API_ROOT, router);
    router
        .layer(Extension(club_context))
        .layer(Extension(person_context))
}
