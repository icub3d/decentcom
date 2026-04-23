use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

/// Records of recent messages and links for a single user.
#[derive(Default)]
struct UserRecord {
    /// Ring buffer of (sent_at, content_hash) for deduplication.
    recent_messages: VecDeque<(Instant, u64)>,
    /// Ring buffer of link-post timestamps.
    recent_links: VecDeque<Instant>,
}

impl UserRecord {
    fn prune_messages(&mut self, window: Duration) {
        let cutoff = Instant::now() - window;
        while self.recent_messages.front().is_some_and(|(t, _)| *t < cutoff) {
            self.recent_messages.pop_front();
        }
    }

    fn prune_links(&mut self, window: Duration) {
        let cutoff = Instant::now() - window;
        while self.recent_links.front().is_some_and(|t| *t < cutoff) {
            self.recent_links.pop_front();
        }
    }
}

pub struct Detector {
    state: HashMap<String, UserRecord>,
    pub duplicate_threshold: usize,
    pub duplicate_window: Duration,
    pub link_threshold: usize,
    pub link_window: Duration,
}

#[derive(Debug, PartialEq)]
pub enum Violation {
    DuplicateMessages,
    LinkFlood,
}

impl Detector {
    pub fn new(
        duplicate_threshold: usize,
        duplicate_window_secs: u64,
        link_threshold: usize,
        link_window_secs: u64,
    ) -> Self {
        Self {
            state: HashMap::new(),
            duplicate_threshold,
            duplicate_window: Duration::from_secs(duplicate_window_secs),
            link_threshold,
            link_window: Duration::from_secs(link_window_secs),
        }
    }

    /// Record a message for `user_id` and return any violations detected.
    pub fn record_message(&mut self, user_id: &str, content: &str) -> Vec<Violation> {
        let record = self.state.entry(user_id.to_string()).or_default();
        let now = Instant::now();

        record.prune_messages(self.duplicate_window);
        record.prune_links(self.link_window);

        let content_hash = hash_content(content);
        record.recent_messages.push_back((now, content_hash));

        let link_count = count_links(content);
        for _ in 0..link_count {
            record.recent_links.push_back(now);
        }

        let mut violations = Vec::new();

        // Check duplicate threshold.
        let dup_count = record
            .recent_messages
            .iter()
            .filter(|(_, h)| *h == content_hash)
            .count();
        if dup_count >= self.duplicate_threshold {
            violations.push(Violation::DuplicateMessages);
        }

        // Check link flood threshold.
        if record.recent_links.len() >= self.link_threshold {
            violations.push(Violation::LinkFlood);
        }

        violations
    }

    /// Clear state for a user (e.g. after they've been kicked/banned).
    pub fn clear_user(&mut self, user_id: &str) {
        self.state.remove(user_id);
    }
}

fn hash_content(content: &str) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    content.to_lowercase().trim().hash(&mut hasher);
    hasher.finish()
}

fn count_links(content: &str) -> usize {
    // Count non-overlapping occurrences of http:// or https://.
    let lower = content.to_lowercase();
    lower.matches("http://").count() + lower.matches("https://").count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duplicate_detection() {
        let mut d = Detector::new(3, 30, 5, 60);
        let uid = "user1";
        // Two duplicates — no violation yet.
        assert!(d.record_message(uid, "hello").is_empty());
        assert!(d.record_message(uid, "hello").is_empty());
        // Third duplicate — violation.
        let v = d.record_message(uid, "hello");
        assert!(v.contains(&Violation::DuplicateMessages));
    }

    #[test]
    fn link_flood_detection() {
        let mut d = Detector::new(5, 30, 3, 60);
        let uid = "user2";
        assert!(d.record_message(uid, "check http://a.com").is_empty());
        assert!(d.record_message(uid, "check http://b.com").is_empty());
        // Third link triggers flood.
        let v = d.record_message(uid, "check http://c.com");
        assert!(v.contains(&Violation::LinkFlood));
    }

    #[test]
    fn no_violation_for_varied_messages() {
        let mut d = Detector::new(5, 30, 5, 60);
        let uid = "user3";
        for i in 0..10 {
            let msg = format!("message {i}");
            let v = d.record_message(uid, &msg);
            assert!(v.is_empty(), "unexpected violation at message {i}");
        }
    }

    #[test]
    fn count_links_finds_http_and_https() {
        assert_eq!(count_links("visit http://a.com and https://b.com"), 2);
        assert_eq!(count_links("no links here"), 0);
        assert_eq!(count_links("HTTP://UPPER.COM"), 1);
    }
}
