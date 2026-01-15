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

/// Request specific context passed to lifecycle hooks (before_create, after_create, etc.).
///
/// This struct contains authentication data allowing for custom authorization logic inside
/// lifecycle hooks.
///
/// The `auth` field is `Option<A>` because whether authentication is required depends on
/// the resource's [`AuthPolicy`](CrudAuthPolicy). For public operations (where
/// [`AuthRequirement::None`] is specified), `auth` may be `None` (or some if the user is still
/// authenticated). For operations requiring authentication ([`AuthRequirement::Authenticated`]),
/// `auth` will always be `Some(...)`.
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
///         // Auth should be present for delete operations (enforced by AuthPolicy).
///         let Some(auth) = &request.auth else {
///             return Ok(Abort::Yes { reason: "Authentication required".into() });
///         };
///
///         if entity.author_subject != auth.subject {
///             return Ok(Abort::Yes { reason: "Only the author can delete".into() });
///         }
///         Ok(Abort::No)
///     }
/// }
/// ```
#[derive(Clone, Debug)]
pub struct RequestContext<A: Auth = NoAuth> {
    /// The authentication data, if present.
    ///
    /// This is `Some` when the request included valid authentication and `None` otherwise.
    /// Whether authentication is required for a given operation is determined by the
    /// resource's [`AuthPolicy`](CrudAuthPolicy), not by this field's presence.
    ///
    /// Lifetime hooks should check this field when implementing custom authorization
    /// logic (e.g., role or entity-ownership checks).
    pub auth: Option<A>,
}

impl<A: Auth> RequestContext<A> {
    /// Create a new request context with authentication data.
    pub fn authenticated(auth: A) -> Self {
        Self { auth: Some(auth) }
    }

    /// Create a new request context without authentication data.
    ///
    /// Use this for public operations where authentication is not required.
    pub fn unauthenticated() -> Self {
        Self { auth: None }
    }
}

impl<A: Auth> Default for RequestContext<A> {
    fn default() -> Self {
        Self { auth: None }
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
    fn extract(extension: Option<axum::Extension<Self>>) -> Result<Self, axum::response::Response>;
}

impl AuthExtractor for NoAuth {
    fn extract(
        _extension: Option<axum::Extension<Self>>,
    ) -> Result<Self, axum::response::Response> {
        Ok(NoAuth)
    }
}

impl<A: RequiresAuth> AuthExtractor for A {
    fn extract(extension: Option<axum::Extension<Self>>) -> Result<Self, axum::response::Response> {
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
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AuthRequirement {
    /// No authentication required (public access).
    #[default]
    None,

    /// User must be authenticated (any valid auth).
    ///
    /// This requirement only checks that authentication is present - it does not
    /// verify any specific claims, roles, or permissions. For fine-grained authorization
    /// (such as role-based access control or ownership checks), use lifecycle hooks
    /// to inspect the authentication data in [`RequestContext::auth`].
    ///
    /// # Example
    ///
    /// ```ignore
    /// // In your CrudLifetime implementation:
    /// async fn before_delete(
    ///     model: &Article,
    ///     context: &ArticleContext,
    ///     request: RequestContext<KeycloakToken<Role>>,
    ///     data: HookData,
    /// ) -> Result<(Abort, HookData), Self::Error> {
    ///     let Some(auth) = &request.auth else {
    ///         return Ok((Abort::Yes { reason: "Authentication required".into() }, data));
    ///     };
    ///
    ///     // Check roles, ownership, or other claims
    ///     if !auth.has_role("admin") && model.creator_id != auth.subject {
    ///         return Ok((Abort::Yes { reason: "Not authorized".into() }, data));
    ///     }
    ///
    ///     Ok((Abort::No, data))
    /// }
    /// ```
    Authenticated,
}

/// Defines authorization policy per CRUD operation.
///
/// Implement this trait to customize which operations require authentication
/// for a given resource. This trait only controls whether authentication is
/// **required** - for fine-grained authorization (roles, ownership, etc.),
/// use lifecycle hooks.
///
/// # Built-in Policies
///
/// - [`OpenAuthPolicy`]: All operations are public
/// - [`DefaultAuthPolicy`]: Reads are public, writes require authentication
/// - [`RestrictedAuthPolicy`]: All operations require authentication
///
/// # Custom Policy Example
///
/// ```ignore
/// struct ArticleAuthPolicy;
///
/// impl CrudAuthPolicy for ArticleAuthPolicy {
///     // Reads are public
///     fn read_requirement() -> AuthRequirement { AuthRequirement::None }
///     // All writes require authentication
///     fn create_requirement() -> AuthRequirement { AuthRequirement::Authenticated }
///     fn update_requirement() -> AuthRequirement { AuthRequirement::Authenticated }
///     fn delete_requirement() -> AuthRequirement { AuthRequirement::Authenticated }
/// }
/// ```
///
/// For role-based authorization, implement checks in your [`CrudLifetime`] hooks.
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
