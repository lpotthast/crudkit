//! User notifications emitted by CrudKit and application actions.

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
/// Provide one with [`provide_crud_notifier`] to route CrudKit's notifications into an
/// application's own notification system.
#[derive(Debug, Clone, Copy)]
pub struct CrudNotifier {
    notify: Callback<CrudNotification>,
}

impl CrudNotifier {
    /// Creates a notifier forwarding every notification to `notify`.
    pub fn new(notify: impl Fn(CrudNotification) + Send + Sync + 'static) -> Self {
        Self {
            notify: Callback::new(notify),
        }
    }

    /// Emits `notification`.
    pub fn notify(&self, notification: CrudNotification) {
        self.notify.run(notification);
    }

    /// Returns a notifier forwarding to this one, setting `origin` on notifications that have
    /// none.
    pub(crate) fn with_origin(self, origin: CrudNotificationOrigin) -> Self {
        Self::new(move |mut notification: CrudNotification| {
            if notification.origin.is_none() {
                notification.origin = Some(origin.clone());
            }
            self.notify(notification);
        })
    }
}

/// Makes `notifier` the destination of CrudKit notifications for the current owner and its
/// descendants.
///
/// Provide one notifier at the application's root, e.g. the notifier of a
/// [`CrudNotificationQueue`] rendered once for the whole page. Every instance, at any depth, then
/// reports to it, and [`CrudNotification::origin`] tells the instances apart. A notifier provided
/// further down overrides the root one for its subtree.
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

/// Identifies a notification in a [`CrudNotificationQueue`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CrudNotificationId(u64);

/// The notifications currently shown. Each disappears after a while unless dismissed earlier.
///
/// Feed it with [`Self::notifier`], e.g. through [`provide_crud_notifier`], and render
/// [`Self::entries`] in any markup. CrudKit's `CrudNotificationRegion` is one such rendering.
#[derive(Debug, Clone, Copy)]
pub struct CrudNotificationQueue {
    /// The shown notifications, oldest first.
    pub entries: Signal<Vec<(CrudNotificationId, CrudNotification)>>,
    writable: RwSignal<Vec<(CrudNotificationId, CrudNotification)>>,
    next_id: StoredValue<u64>,
    visible_for: Duration,
}

impl Default for CrudNotificationQueue {
    /// Creates a queue showing notifications for six seconds.
    fn default() -> Self {
        Self::new(Duration::from_secs(6))
    }
}

impl CrudNotificationQueue {
    /// Creates an empty queue showing each notification for `visible_for`.
    #[must_use]
    pub fn new(visible_for: Duration) -> Self {
        let writable = RwSignal::new(Vec::new());
        Self {
            entries: writable.read_only().into(),
            writable,
            next_id: StoredValue::new(0),
            visible_for,
        }
    }

    /// Shows `notification`.
    pub fn push(&self, notification: CrudNotification) {
        let id = CrudNotificationId(self.next_id.get_value());
        self.next_id
            .update_value(|next| *next = next.wrapping_add(1));
        self.writable
            .update(|entries| entries.push((id, notification)));
        let this = *self;
        set_timeout(move || this.dismiss(id), self.visible_for);
    }

    /// Removes the notification `id`.
    pub fn dismiss(&self, id: CrudNotificationId) {
        // The queue may already be disposed when a timeout fires after unmounting.
        _ = self
            .writable
            .try_update(|entries| entries.retain(|(entry, _)| *entry != id));
    }

    /// Returns a notifier showing notifications in this queue.
    #[must_use]
    pub fn notifier(&self) -> CrudNotifier {
        let this = *self;
        CrudNotifier::new(move |notification| this.push(notification))
    }
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
