//! The limits of the server: the size of a request, the time it may take,
//! the requests per second of each client and the connections open at once.

use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy)]
pub struct Limits {
    /// The largest body accepted, in bytes (4 MB).
    pub body: usize,
    /// The time to receive a whole request. A client that sends slowly, or
    /// stops, is cut off (`408`).
    pub request_time: Duration,
    /// Requests per second for each client. Generous: an agent that draws
    /// fast makes tens of calls a second at most.
    pub requests_per_second: usize,
    /// Connections served at the same time.
    pub connections: usize,
}

impl Default for Limits {
    fn default() -> Self {
        Limits {
            body: 4 * 1024 * 1024,
            request_time: Duration::from_secs(10),
            requests_per_second: 100,
            connections: 32,
        }
    }
}

/// Counts the requests of each key (a session) in the last second.
#[derive(Default)]
pub struct RateLimiter {
    seen: HashMap<String, VecDeque<Instant>>,
}

impl RateLimiter {
    /// True when the request is inside the limit. It counts the request.
    pub fn allow(&mut self, key: &str, per_second: usize) -> bool {
        self.allow_at(key, per_second, Instant::now())
    }

    fn allow_at(&mut self, key: &str, per_second: usize, now: Instant) -> bool {
        // Forget keys that are quiet, so the map cannot grow without end.
        if self.seen.len() > 64 {
            self.seen.retain(|_, times| {
                times.back().is_some_and(|t| now.duration_since(*t) < Duration::from_secs(1))
            });
        }
        let times = self.seen.entry(key.to_string()).or_default();
        while times.front().is_some_and(|t| now.duration_since(*t) >= Duration::from_secs(1)) {
            times.pop_front();
        }
        if times.len() >= per_second {
            return false;
        }
        times.push_back(now);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Checks the per-second limit: the extra request is refused, another
    // client is not affected, and the window slides after one second.
    #[test]
    fn the_rate_limit_is_per_client_and_per_second() {
        let mut limiter = RateLimiter::default();
        let start = Instant::now();
        assert!(limiter.allow_at("a", 2, start));
        assert!(limiter.allow_at("a", 2, start));
        assert!(!limiter.allow_at("a", 2, start));
        assert!(limiter.allow_at("b", 2, start));
        assert!(limiter.allow_at("a", 2, start + Duration::from_millis(1001)));
    }
}
