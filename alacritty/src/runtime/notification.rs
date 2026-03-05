use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub struct NotificationTarget {
    pub workspace_id: String,
    pub surface_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NotificationRecord {
    pub id: u64,
    pub target: NotificationTarget,
    pub title: String,
    pub body: String,
    pub created_at_millis: u128,
    pub read: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewNotification {
    pub target: NotificationTarget,
    pub title: String,
    pub body: String,
}

#[derive(Default, Debug)]
pub struct NotificationStore {
    next_id: u64,
    notifications: Vec<NotificationRecord>,
}

impl NotificationStore {
    pub fn insert(&mut self, notification: NewNotification) -> NotificationRecord {
        // Keep only the latest attention event for each target.
        self.notifications.retain(|existing| existing.target != notification.target);

        let record = NotificationRecord {
            id: self.next_id,
            target: notification.target,
            title: notification.title,
            body: notification.body,
            created_at_millis: now_millis(),
            read: false,
        };
        self.next_id += 1;

        self.notifications.push(record.clone());
        record
    }

    pub fn list(&self) -> &[NotificationRecord] {
        &self.notifications
    }

    pub fn latest_for_workspace(&self, workspace_id: &str) -> Option<&NotificationRecord> {
        self.notifications
            .iter()
            .rev()
            .find(|notification| notification.target.workspace_id == workspace_id)
    }

    pub fn latest_unread_for_workspace(&self, workspace_id: &str) -> Option<&NotificationRecord> {
        self.notifications.iter().rev().find(|notification| {
            notification.target.workspace_id == workspace_id && !notification.read
        })
    }

    pub fn recent(&self, limit: usize) -> Vec<&NotificationRecord> {
        self.notifications.iter().rev().take(limit).collect()
    }

    pub fn unread_count_for_workspace(&self, workspace_id: &str) -> usize {
        self.notifications
            .iter()
            .filter(|notification| {
                !notification.read && notification.target.workspace_id == workspace_id
            })
            .count()
    }

    pub fn mark_read(&mut self, id: u64) -> Option<&NotificationRecord> {
        let notification =
            self.notifications.iter_mut().find(|notification| notification.id == id)?;
        notification.read = true;
        Some(notification)
    }
}

fn now_millis() -> u128 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or(Duration::ZERO).as_millis()
}

#[cfg(test)]
mod tests {
    use super::{NewNotification, NotificationStore, NotificationTarget};

    #[test]
    fn dedupe_target_on_insert() {
        let target = NotificationTarget {
            workspace_id: String::from("workspace:1"),
            surface_id: Some(String::from("surface:1")),
        };

        let mut store = NotificationStore::default();
        store.insert(NewNotification {
            target: target.clone(),
            title: String::from("first"),
            body: String::from("1"),
        });
        store.insert(NewNotification {
            target,
            title: String::from("second"),
            body: String::from("2"),
        });

        assert_eq!(store.list().len(), 1);
        assert_eq!(store.list()[0].title, "second");
    }

    #[test]
    fn unread_count_ignores_read_notifications() {
        let mut store = NotificationStore::default();
        let created = store.insert(NewNotification {
            target: NotificationTarget {
                workspace_id: String::from("workspace:7"),
                surface_id: None,
            },
            title: String::from("attention"),
            body: String::from("body"),
        });

        assert_eq!(store.unread_count_for_workspace("workspace:7"), 1);
        let _ = store.mark_read(created.id);
        assert_eq!(store.unread_count_for_workspace("workspace:7"), 0);
    }

    #[test]
    fn latest_for_workspace_picks_last_insert() {
        let mut store = NotificationStore::default();
        store.insert(NewNotification {
            target: NotificationTarget {
                workspace_id: String::from("workspace:2"),
                surface_id: None,
            },
            title: String::from("first"),
            body: String::from("a"),
        });
        store.insert(NewNotification {
            target: NotificationTarget {
                workspace_id: String::from("workspace:2"),
                surface_id: Some(String::from("surface:22")),
            },
            title: String::from("second"),
            body: String::from("b"),
        });

        assert_eq!(
            store.latest_for_workspace("workspace:2").map(|n| n.title.as_str()),
            Some("second")
        );
    }

    #[test]
    fn latest_unread_for_workspace_skips_read() {
        let mut store = NotificationStore::default();
        let first = store.insert(NewNotification {
            target: NotificationTarget {
                workspace_id: String::from("workspace:3"),
                surface_id: None,
            },
            title: String::from("first"),
            body: String::from("a"),
        });
        let _ = store.mark_read(first.id);
        store.insert(NewNotification {
            target: NotificationTarget {
                workspace_id: String::from("workspace:3"),
                surface_id: Some(String::from("surface:31")),
            },
            title: String::from("second"),
            body: String::from("b"),
        });

        assert_eq!(
            store.latest_unread_for_workspace("workspace:3").map(|n| n.title.as_str()),
            Some("second")
        );
    }
}
