//! Authentication and authorization abstractions for CRUD operations.
//!
//! This module provides a framework-agnostic way to handle authentication in CRUD operations.
//! It supports any authentication provider (Keycloak, JWT, session, API key, etc.) as well as
//! public resources that require no authentication.

/// Marker trait for types usable as authentication context.
///
/// Any type satisfying `Clone + Send + Sync + 'static` qualifies.
/// This trait is automatically implemented for all such types.
///
/// # Axum Integration
///
/// The implementing type must be injectable as an Axum [`Extension<A>`](axum::Extension).
/// Ensure your router includes middleware that provides this Extension.
///
/// # Examples
///
/// - `KeycloakToken<Ro>` from `axum_keycloak_auth`
/// - Custom JWT/session types
/// - [`NoAuth`] for public resources
pub trait Auth: Clone + Send + Sync + 'static {}

impl<A: Clone + Send + Sync + 'static> Auth for A {}

/// Unit type for resources requiring no authentication.
///
/// Use this as the `Auth` type for public resources that don't require any authentication.
///
/// # Example
///
/// ```ignore
/// impl CrudResource for PublicArticle {
///     type Auth = NoAuth;
///     type AuthPolicy = OpenAuthPolicy;
///     // ...
/// }
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct NoAuth;

/// Context passed to lifecycle hooks containing authentication data.
///
/// This struct wraps the authentication data and is passed to lifecycle hooks
/// (before_create, after_create, etc.) so they can perform custom authorization logic.
///
/// # Example
///
/// ```ignore
/// impl CrudLifetime<Article> for ArticleLifetime {
///     async fn before_delete(
///         entity: &Article,
///         context: &ArticleContext,
///         request: RequestContext<KeycloakToken<MyRoles>>,
///     ) -> Result<Abort, Self::Error> {
///         // Access auth fields directly
///         let user_subject = &request.auth.subject;
///
///         if entity.author_subject != *user_subject {
///             return Ok(Abort::WithReason("Only the author can delete".into()));
///         }
///         Ok(Abort::No)
///     }
/// }
/// ```
#[derive(Clone, Debug)]
pub struct RequestContext<A: Auth = NoAuth> {
    /// The authentication data. Access fields directly based on your auth type.
    pub auth: A,
}

impl<A: Auth> RequestContext<A> {
    /// Create a new request context with the given authentication data.
    pub fn new(auth: A) -> Self {
        Self { auth }
    }
}

impl Default for RequestContext<NoAuth> {
    fn default() -> Self {
        Self { auth: NoAuth }
    }
}

/// Marker trait for auth types that require the Extension to be present.
///
/// Implement this for any authentication type (e.g., `KeycloakToken<Role>`)
/// to indicate that requests must include the auth Extension.
///
/// `NoAuth` does NOT implement this trait, allowing compile-time differentiation
/// between public and authenticated resources.
///
/// # Example
///
/// ```ignore
/// use crudkit_rs::auth::RequiresAuth;
/// use axum_keycloak_auth::decode::KeycloakToken;
///
/// impl RequiresAuth for KeycloakToken<MyRoles> {}
/// ```
///
/// # Feature: `keycloak-auth`
///
/// When the `keycloak-auth` feature is enabled, `RequiresAuth` is automatically
/// implemented for `KeycloakToken<R>` where `R: axum_keycloak_auth::role::Role`.
pub trait RequiresAuth: Auth {}

/// Blanket implementation of `RequiresAuth` for Keycloak tokens.
///
/// This is enabled by the `keycloak-auth` feature flag.
#[cfg(feature = "keycloak-auth")]
impl<R> RequiresAuth for axum_keycloak_auth::decode::KeycloakToken<R> where
    R: axum_keycloak_auth::role::Role + Clone + Send + Sync + 'static
{
}

/// Trait for extracting authentication from Axum optional Extension.
///
/// Returns the auth value itself, not `RequestContext` - this keeps concerns separated
/// and allows `RequestContext` to be extended with additional fields in the future.
///
/// # Implementations
///
/// - `NoAuth`: Always returns `Ok(NoAuth)`, ignores the Extension
/// - Types implementing `RequiresAuth`: Requires Extension present, returns 401 if missing
pub trait AuthExtractor: Auth + Sized {
    /// Extract auth from an optional Extension.
    fn extract(
        extension: Option<axum::Extension<Self>>,
    ) -> Result<Self, axum::response::Response>;
}

impl AuthExtractor for NoAuth {
    fn extract(
        _extension: Option<axum::Extension<Self>>,
    ) -> Result<Self, axum::response::Response> {
        Ok(NoAuth)
    }
}

impl<A: RequiresAuth> AuthExtractor for A {
    fn extract(
        extension: Option<axum::Extension<Self>>,
    ) -> Result<Self, axum::response::Response> {
        use axum::{http::StatusCode, response::IntoResponse, Json};
        match extension {
            Some(axum::Extension(auth)) => Ok(auth),
            None => Err((
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({"error": "Authentication required"})),
            )
                .into_response()),
        }
    }
}

/// Authorization requirement for a CRUD operation.
///
/// Used by [`CrudAuthPolicy`] to specify what level of authentication
/// is required for each CRUD operation.
#[derive(Clone, Debug, Default)]
pub enum AuthRequirement {
    /// No authentication required (public access).
    #[default]
    None,
    /// User must be authenticated (any valid auth).
    Authenticated,
    /// User must have one of the specified roles.
    /// Role checking is delegated to middleware or custom logic.
    Roles(Vec<String>),
}

/// Defines authorization policy per CRUD operation.
///
/// Implement this trait to customize which operations require authentication
/// for a given resource.
///
/// # Default Implementation
///
/// The default implementation ([`DefaultAuthPolicy`]) allows public reads
/// but requires authentication for create, update, and delete operations.
///
/// # Example
///
/// ```ignore
/// struct ArticleAuthPolicy;
///
/// impl CrudAuthPolicy for ArticleAuthPolicy {
///     fn read_requirement() -> AuthRequirement { AuthRequirement::None }
///     fn create_requirement() -> AuthRequirement { AuthRequirement::Roles(vec!["author".into()]) }
///     fn update_requirement() -> AuthRequirement { AuthRequirement::Roles(vec!["author".into(), "editor".into()]) }
///     fn delete_requirement() -> AuthRequirement { AuthRequirement::Roles(vec!["admin".into()]) }
/// }
/// ```
pub trait CrudAuthPolicy: Send + Sync + 'static {
    /// Authorization requirement for read operations (count, read_one, read_many).
    fn read_requirement() -> AuthRequirement {
        AuthRequirement::None
    }

    /// Authorization requirement for create operations.
    fn create_requirement() -> AuthRequirement {
        AuthRequirement::Authenticated
    }

    /// Authorization requirement for update operations.
    fn update_requirement() -> AuthRequirement {
        AuthRequirement::Authenticated
    }

    /// Authorization requirement for delete operations.
    fn delete_requirement() -> AuthRequirement {
        AuthRequirement::Authenticated
    }
}

/// Default authorization policy: reads are public, writes require authentication.
///
/// This is the default policy used when no `AuthPolicy` is specified on a resource.
pub struct DefaultAuthPolicy;

impl CrudAuthPolicy for DefaultAuthPolicy {}

/// Fully open authorization policy: all operations are public.
///
/// Use this for resources that should be accessible without any authentication.
pub struct OpenAuthPolicy;

impl CrudAuthPolicy for OpenAuthPolicy {
    fn create_requirement() -> AuthRequirement {
        AuthRequirement::None
    }

    fn update_requirement() -> AuthRequirement {
        AuthRequirement::None
    }

    fn delete_requirement() -> AuthRequirement {
        AuthRequirement::None
    }
}

/// Fully restricted authorization policy: all operations require authentication.
///
/// Use this for resources where even read operations should be protected.
pub struct RestrictedAuthPolicy;

impl CrudAuthPolicy for RestrictedAuthPolicy {
    fn read_requirement() -> AuthRequirement {
        AuthRequirement::Authenticated
    }
}
