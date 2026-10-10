//! User notifications emitted by CrudKit and application actions.

use leptonic::hooks::{ToastOptions, ToastQueue};
use leptos::prelude::*;
use std::time::Duration;
use uuid::Uuid;

/// Severity of a [`CrudNotification`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CrudNotificationKind {
    /// An operation succeeded.
    Success,
    /// Neutral information.
    Info,
    /// An operation partially succeeded or was refused.
    Warning,
    /// An operation failed.
    Error,
}

/// A message CrudKit wants to show to the user.
///
/// CrudKit only describes notifications. Rendering them, e.g. as toasts, belongs to whatever the
/// current [`CrudNotifier`] forwards them to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrudNotification {
    /// Severity of the notification.
    pub kind: CrudNotificationKind,
    /// Short heading.
    pub title: String,
    /// Message body.
    pub message: String,
    /// The instance that emitted the notification, set by CrudKit for every notification emitted
    /// through an instance, including action outcomes. `None` for notifications emitted outside of
    /// an instance.
    pub origin: Option<CrudNotificationOrigin>,
}

/// The CrudKit instance that emitted a [`CrudNotification`].
///
/// Lets one application-wide notifier group, filter, or place notifications by instance or
/// resource.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrudNotificationOrigin {
    /// Volatile identifier of the instance's mount, see
    /// [`CrudInstanceContext::id`](crate::instance::CrudInstanceContext::id).
    pub instance_id: Uuid,
    /// Name of the instance, see
    /// [`CrudInstanceContext::name`](crate::instance::CrudInstanceContext::name).
    pub instance_name: &'static str,
    /// Name of the instance's resource on the wire.
    pub resource_name: String,
}

impl CrudNotification {
    /// Creates a notification without an origin.
    pub fn new(
        kind: CrudNotificationKind,
        title: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            title: title.into(),
            message: message.into(),
            origin: None,
        }
    }

    /// Creates a [`CrudNotificationKind::Success`] notification.
    pub fn success(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(CrudNotificationKind::Success, title, message)
    }

    /// Creates a [`CrudNotificationKind::Info`] notification.
    pub fn info(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(CrudNotificationKind::Info, title, message)
    }

    /// Creates a [`CrudNotificationKind::Warning`] notification.
    pub fn warning(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(CrudNotificationKind::Warning, title, message)
    }

    /// Creates a [`CrudNotificationKind::Error`] notification.
    pub fn error(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(CrudNotificationKind::Error, title, message)
    }
}

/// Destination of [`CrudNotification`]s.
///
/// Provide one with [`provide_crud_notifier`]: typically [`Self::toasts`], or [`Self::new`] to route
/// CrudKit's notifications into an application's own notification system.
#[derive(Debug, Clone, Copy)]
pub struct CrudNotifier {
    notify: Callback<CrudNotification>,
    /// The origin given to notifications without one, see [`Self::with_origin`].
    origin: Option<StoredValue<CrudNotificationOrigin>>,
}

impl CrudNotifier {
    /// Creates a notifier forwarding every notification to `notify`.
    pub fn new(notify: impl Fn(CrudNotification) + Send + Sync + 'static) -> Self {
        Self {
            notify: Callback::new(notify),
            origin: None,
        }
    }

    /// Creates a notifier showing notifications as toasts of `queue`, e.g. rendered with
    /// Leptonic's `ToastRegion` and `Toast` atoms.
    ///
    /// Successes and information close after five seconds. Warnings and errors stay until closed,
    /// so they cannot be missed.
    #[must_use]
    pub fn toasts(queue: ToastQueue<CrudNotification>) -> Self {
        Self::new(move |notification: CrudNotification| {
            let timeout = match notification.kind {
                CrudNotificationKind::Success | CrudNotificationKind::Info => {
                    Some(Duration::from_secs(5))
                }
                CrudNotificationKind::Warning | CrudNotificationKind::Error => None,
            };
            queue.add(
                notification,
                ToastOptions {
                    timeout,
                    ..ToastOptions::default()
                },
            );
        })
    }

    /// Emits `notification`.
    ///
    /// Requests may complete after the component that started them is gone, e.g. an instance
    /// unmounted while deleting. Their notifications are still delivered, without an origin once
    /// the instance is gone, and dropped with a warning once the notifier itself is gone.
    pub fn notify(&self, mut notification: CrudNotification) {
        if notification.origin.is_none() {
            notification.origin = self.origin.and_then(|origin| origin.try_get_value());
        }
        if self.notify.try_run(notification).is_none() {
            tracing::warn!(
                "dropped a CrudKit notification emitted after its notifier was disposed"
            );
        }
    }

    /// Returns a notifier forwarding to this one, setting `origin` on notifications that have
    /// none. The origin lives as long as the calling owner, e.g. an instance.
    pub(crate) fn with_origin(self, origin: CrudNotificationOrigin) -> Self {
        Self {
            notify: self.notify,
            origin: Some(StoredValue::new(origin)),
        }
    }
}

/// Makes `notifier` the destination of CrudKit notifications for the current owner and its
/// descendants.
///
/// Provide one notifier at the application's root, e.g. [`CrudNotifier::toasts`] of a toast queue
/// rendered once for the whole page. Every instance, at any depth, then reports to it, and
/// [`CrudNotification::origin`] tells the instances apart. A notifier provided further down
/// overrides the root one for its subtree.
pub fn provide_crud_notifier(notifier: CrudNotifier) {
    provide_context(notifier);
}

/// Returns the nearest provided [`CrudNotifier`].
///
/// Without a provided notifier, notifications are logged as warnings.
#[must_use]
pub fn use_crud_notifier() -> CrudNotifier {
    use_context::<CrudNotifier>().unwrap_or_else(|| {
        CrudNotifier::new(|notification: CrudNotification| {
            tracing::warn!(
                ?notification,
                "CrudKit notification without a notifier; provide one with `provide_crud_notifier`"
            );
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use assertr::prelude::*;
    use leptos::reactive::owner::Owner;
    use std::sync::{Arc, Mutex};

    fn origin(instance_name: &'static str) -> CrudNotificationOrigin {
        CrudNotificationOrigin {
            instance_id: Uuid::new_v4(),
            instance_name,
            resource_name: "clubs".to_owned(),
        }
    }

    #[test]
    fn instance_notifiers_annotate_notifications_without_overriding_an_origin() {
        Owner::new().with(|| {
            let received = Arc::new(Mutex::new(Vec::<CrudNotification>::new()));
            let recorded = received.clone();
            let sink = CrudNotifier::new(move |notification| {
                recorded.lock().expect("lock").push(notification);
            });
            let instance = origin("clubs-table");
            let nested = origin("nested");
            let notifier = sink.with_origin(instance.clone());

            notifier.notify(CrudNotification::info("Saved", "The club was saved."));
            let mut preset = CrudNotification::error("Failed", "The club was not saved.");
            preset.origin = Some(nested.clone());
            notifier.notify(preset);

            let origins = received
                .lock()
                .expect("lock")
                .iter()
                .map(|notification| notification.origin.clone())
                .collect::<Vec<_>>();
            assert_that!(origins).is_equal_to(vec![Some(instance), Some(nested)]);
        });
    }
}
