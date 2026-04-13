use dashmap::DashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
struct ChallengeEntry {
    nonce: Vec<u8>,
    expires_at: Instant,
}

#[derive(Debug)]
pub struct ChallengeStore {
    entries: DashMap<String, ChallengeEntry>,
    ttl: Duration,
}

pub type SharedChallengeStore = Arc<ChallengeStore>;

impl ChallengeStore {
    pub fn new(ttl: Duration) -> Self {
        Self {
            entries: DashMap::new(),
            ttl,
        }
    }

    pub fn insert(&self, pubkey: String, nonce: Vec<u8>) {
        self.entries.insert(
            pubkey,
            ChallengeEntry {
                nonce,
                expires_at: Instant::now() + self.ttl,
            },
        );
    }

    pub fn get(&self, pubkey: &str) -> Option<Vec<u8>> {
        let entry = self.entries.get(pubkey)?;
        if entry.expires_at <= Instant::now() {
            drop(entry);
            self.entries.remove(pubkey);
            return None;
        }
        Some(entry.nonce.clone())
    }

    pub fn take(&self, pubkey: &str) -> Option<Vec<u8>> {
        let (.., entry) = self.entries.remove(pubkey)?;
        if entry.expires_at <= Instant::now() {
            return None;
        }
        Some(entry.nonce)
    }

    pub fn purge_expired(&self) -> usize {
        let now = Instant::now();
        let mut removed = 0usize;
        self.entries.retain(|_, entry| {
            let keep = entry.expires_at > now;
            if !keep {
                removed += 1;
            }
            keep
        });
        removed
    }
}

#[cfg(test)]
mod tests {
    use super::ChallengeStore;
    use std::time::Duration;

    #[test]
    fn challenge_store_creates_and_retrieves_challenges() {
        let store = ChallengeStore::new(Duration::from_secs(300));
        let nonce = vec![1u8; 32];
        store.insert("pk1".to_string(), nonce.clone());

        let got = store.get("pk1");
        assert_eq!(got, Some(nonce));
    }

    #[test]
    fn purge_expired_removes_stale_entries() {
        let store = ChallengeStore::new(Duration::from_millis(1));
        store.insert("pk1".to_string(), vec![7u8; 32]);
        std::thread::sleep(Duration::from_millis(5));

        let removed = store.purge_expired();
        assert_eq!(removed, 1);
        assert!(store.get("pk1").is_none());
    }

    #[test]
    fn challenges_are_single_use() {
        let store = ChallengeStore::new(Duration::from_secs(300));
        store.insert("pk1".to_string(), vec![9u8; 32]);

        assert!(store.take("pk1").is_some());
        assert!(store.take("pk1").is_none());
    }
}
