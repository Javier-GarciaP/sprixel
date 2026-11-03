//! The sessions of the clients. A session starts with `initialize`, where the
//! client says its name and version, and is found again by the id the server
//! gave (the `Mcp-Session-Id` header).

use super::random::random_hex;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// How many clients may be connected at the same time.
pub const MAX_SESSIONS: usize = 16;
/// A session nobody uses for this long is forgotten when room is needed.
const IDLE: Duration = Duration::from_secs(10 * 60);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Session {
    /// The label the client gave itself. See `clean_name`.
    pub name: String,
    pub version: String,
}

struct Entry {
    session: Session,
    last_seen: Instant,
}

#[derive(Default)]
pub struct Sessions(HashMap<String, Entry>);

impl Sessions {
    /// Starts a session. `None` when too many clients are connected.
    pub fn start(&mut self, session: Session) -> Option<String> {
        if self.0.len() >= MAX_SESSIONS {
            self.0.retain(|_, entry| entry.last_seen.elapsed() < IDLE);
        }
        if self.0.len() >= MAX_SESSIONS {
            return None;
        }
        let id = random_hex(16);
        self.0.insert(id.clone(), Entry { session, last_seen: Instant::now() });
        Some(id)
    }

    /// The session of an id, marking it as used now.
    pub fn touch(&mut self, id: &str) -> Option<Session> {
        let entry = self.0.get_mut(id)?;
        entry.last_seen = Instant::now();
        Some(entry.session.clone())
    }

    /// Ends every session of a client (it was removed by the user).
    pub fn end_client(&mut self, name: &str, version: &str) {
        self.0.retain(|_, entry| !(entry.session.name == name && entry.session.version == version));
    }

    pub fn exists(&self, id: &str) -> bool {
        self.0.contains_key(id)
    }

    pub fn end(&mut self, id: &str) {
        self.0.remove(id);
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

/// Characters that draw nothing or change the direction of the text. A name
/// with them could look like another name, so they are removed.
fn is_invisible(c: char) -> bool {
    matches!(c,
        '\u{00AD}' | '\u{061C}' | '\u{180E}' | '\u{200B}'..='\u{200F}' | '\u{202A}'..='\u{202E}'
        | '\u{2060}'..='\u{2064}' | '\u{2066}'..='\u{2069}' | '\u{FEFF}')
}

/// Text from a client, safe to keep and show: no control, invisible or
/// direction characters, trimmed, at most `max` characters.
pub fn clean_text(raw: &str, max: usize) -> String {
    let text: String = raw.chars().filter(|c| !c.is_control() && !is_invisible(*c)).take(max).collect();
    text.trim().to_string()
}

/// The name of a client as the app keeps and shows it. It is only a label
/// that the client chose, so it is cleaned (see `clean_text`), at most 64
/// characters, never empty, and never `user`, which is the author of the
/// changes made by the person and must not be faked.
pub fn clean_name(raw: &str) -> String {
    let name = clean_text(raw, 64);
    if name.is_empty() {
        "unknown client".to_string()
    } else if name.eq_ignore_ascii_case(crate::history::USER) {
        format!("{name} (agent)")
    } else {
        name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn session(name: &str) -> Session {
        Session { name: name.to_string(), version: "1".to_string() }
    }

    // Checks that a started session is found again by its id, and that an
    // unknown id or an ended session is not.
    #[test]
    fn a_session_is_found_by_its_id_until_it_ends() {
        let mut sessions = Sessions::default();
        let id = sessions.start(session("Claude Code")).unwrap();
        assert_eq!(sessions.touch(&id), Some(session("Claude Code")));
        assert_eq!(sessions.touch("nope"), None);
        sessions.end(&id);
        assert_eq!(sessions.touch(&id), None);
    }

    // Checks the limit of clients connected at the same time.
    #[test]
    fn too_many_sessions_are_refused() {
        let mut sessions = Sessions::default();
        for _ in 0..MAX_SESSIONS {
            assert!(sessions.start(session("a")).is_some());
        }
        assert!(sessions.start(session("a")).is_none());
        assert_eq!(sessions.len(), MAX_SESSIONS);
    }

    // Checks that a client cannot fake the author of the person, hide in a
    // control character or send a huge name.
    #[test]
    fn names_are_cleaned() {
        assert_eq!(clean_name("  Claude Code \n"), "Claude Code");
        assert_eq!(clean_name("User"), "User (agent)");
        assert_eq!(clean_name("\u{0}\u{7}"), "unknown client");
        assert_eq!(clean_name(&"x".repeat(500)).chars().count(), 64);
        // A direction mark or a zero-width space cannot make a name look like another.
        assert_eq!(clean_name("Claude\u{202E} Code\u{200B}"), "Claude Code");
        assert_eq!(clean_text("1.0\u{0}\u{202E}", 64), "1.0");
    }
}
