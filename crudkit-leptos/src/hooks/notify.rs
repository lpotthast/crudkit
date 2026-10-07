//! User notifications emitted by CrudKit and application actions.

use leptos::prelude::*;
use std::time::Duration;

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
}

impl CrudNotification {
    /// Creates a notification.
    pub fn new(
        kind: CrudNotificationKind,
        title: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            title: title.into(),
            message: message.into(),
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
}

/// Makes `notifier` the destination of CrudKit notifications for the current owner and its
/// descendants.
pub fn provide_crud_notifier(notifier: CrudNotifier) {
    provide_context(notifier);
}

/// Returns the nearest provided [`CrudNotifier`].
///
/// [`crate::instance::CrudInstanceMgr`] provides one showing notifications in a
/// [`crate::components::notifications::CrudNotificationRegion`], unless the application provided
/// its own. Outside of a manager, notifications are only logged.
#[must_use]
pub fn use_crud_notifier() -> CrudNotifier {
    use_context::<CrudNotifier>().unwrap_or_else(|| {
        CrudNotifier::new(|notification: CrudNotification| {
            tracing::info!(?notification, "CrudKit notification without a notifier");
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
