use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

use tokio::sync::mpsc;

#[derive(Debug, Default)]
struct RegistryInner {
    senders: HashMap<String, mpsc::UnboundedSender<String>>,
    connection_users: HashMap<String, String>,
    user_connections: HashMap<String, HashSet<String>>,
    channel_subscriptions: HashMap<String, HashSet<String>>,
}

#[derive(Debug, Clone, Default)]
pub struct ConnectionRegistry {
    inner: Arc<RwLock<RegistryInner>>,
}

impl ConnectionRegistry {
    pub fn register(
        &self,
        connection_id: String,
        user_id: String,
        sender: mpsc::UnboundedSender<String>,
    ) {
        let mut inner = self.inner.write().expect("registry lock poisoned");
        inner.senders.insert(connection_id.clone(), sender);
        inner
            .connection_users
            .insert(connection_id.clone(), user_id.clone());
        inner
            .user_connections
            .entry(user_id)
            .or_default()
            .insert(connection_id);
    }

    pub fn unregister(&self, connection_id: &str) {
        let mut inner = self.inner.write().expect("registry lock poisoned");
        inner.senders.remove(connection_id);

        if let Some(user_id) = inner.connection_users.remove(connection_id) {
            if let Some(connections) = inner.user_connections.get_mut(&user_id) {
                connections.remove(connection_id);
                if connections.is_empty() {
                    inner.user_connections.remove(&user_id);
                }
            }
        }

        inner.channel_subscriptions.retain(|_, members| {
            members.remove(connection_id);
            !members.is_empty()
        });
    }

    pub fn subscribe(&self, connection_id: &str, channel_id: &str) {
        let mut inner = self.inner.write().expect("registry lock poisoned");
        inner
            .channel_subscriptions
            .entry(channel_id.to_string())
            .or_default()
            .insert(connection_id.to_string());
    }

    pub fn unsubscribe(&self, connection_id: &str, channel_id: &str) {
        let mut inner = self.inner.write().expect("registry lock poisoned");
        if let Some(members) = inner.channel_subscriptions.get_mut(channel_id) {
            members.remove(connection_id);
            if members.is_empty() {
                inner.channel_subscriptions.remove(channel_id);
            }
        }
    }

    pub fn broadcast_to_channel(&self, channel_id: &str, payload: &str) {
        let inner = self.inner.read().expect("registry lock poisoned");
        if let Some(members) = inner.channel_subscriptions.get(channel_id) {
            for connection_id in members {
                if let Some(sender) = inner.senders.get(connection_id) {
                    let _ = sender.send(payload.to_string());
                }
            }
        }
    }

    pub fn send_to_user(&self, user_id: &str, payload: &str) {
        let inner = self.inner.read().expect("registry lock poisoned");
        if let Some(connections) = inner.user_connections.get(user_id) {
            for connection_id in connections {
                if let Some(sender) = inner.senders.get(connection_id) {
                    let _ = sender.send(payload.to_string());
                }
            }
        }
    }

    pub fn broadcast_all(&self, payload: &str) {
        let inner = self.inner.read().expect("registry lock poisoned");
        for sender in inner.senders.values() {
            let _ = sender.send(payload.to_string());
        }
    }

    #[cfg(test)]
    fn counts(&self) -> (usize, usize, usize) {
        let inner = self.inner.read().expect("registry lock poisoned");
        (
            inner.senders.len(),
            inner.user_connections.len(),
            inner.channel_subscriptions.len(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::ConnectionRegistry;
    use tokio::sync::mpsc;

    #[test]
    fn registry_tracks_registration_subscriptions_and_cleanup() {
        let registry = ConnectionRegistry::default();
        let (tx_a, mut rx_a) = mpsc::unbounded_channel::<String>();
        let (tx_b, mut rx_b) = mpsc::unbounded_channel::<String>();

        registry.register("conn_a".to_string(), "user_1".to_string(), tx_a);
        registry.register("conn_b".to_string(), "user_1".to_string(), tx_b);

        registry.subscribe("conn_a", "chan_1");
        registry.subscribe("conn_b", "chan_2");

        registry.broadcast_to_channel("chan_1", "msg1");
        assert_eq!(rx_a.try_recv().unwrap(), "msg1");
        assert!(rx_b.try_recv().is_err());

        registry.send_to_user("user_1", "direct");
        assert_eq!(rx_a.try_recv().unwrap(), "direct");
        assert_eq!(rx_b.try_recv().unwrap(), "direct");

        registry.unsubscribe("conn_a", "chan_1");
        registry.broadcast_to_channel("chan_1", "ignored");
        assert!(rx_a.try_recv().is_err());

        registry.unregister("conn_a");
        registry.unregister("conn_b");
        assert_eq!(registry.counts(), (0, 0, 0));
    }
}
