use super::atomic::FileWriter;
use super::session::{SaveStatus, Session};
use std::sync::{Arc, Condvar, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

/// The quiet time after the last change before the project is saved.
pub const DEFAULT_DEBOUNCE: Duration = Duration::from_millis(500);

/// Saves a session in the background. Every change restarts a timer; when
/// the timer ends without new changes, the pending files are written. When
/// the autosave is dropped (the app closes) a last save is done at once.
pub struct Autosave {
    shared: Arc<Shared>,
    status: Arc<Mutex<SaveStatus>>,
    worker: Option<JoinHandle<()>>,
}

struct Shared {
    session: Mutex<Session>,
    signal: Mutex<Signal>,
    wake: Condvar,
    debounce: Duration,
}

struct Signal {
    /// When the last change happened, if it is not saved yet.
    last_change: Option<Instant>,
    stop: bool,
}

impl Autosave {
    pub fn start(session: Session, debounce: Duration) -> Autosave {
        let status = session.status_handle();
        let shared = Arc::new(Shared {
            session: Mutex::new(session),
            signal: Mutex::new(Signal {
                last_change: None,
                stop: false,
            }),
            wake: Condvar::new(),
            debounce,
        });
        let worker = thread::spawn({
            let shared = shared.clone();
            move || run(&shared)
        });
        Autosave {
            shared,
            status,
            worker: Some(worker),
        }
    }

    /// Runs a change on the session and restarts the save timer.
    pub fn edit<R>(&self, change: impl FnOnce(&mut Session) -> R) -> R {
        let result = change(&mut self.shared.session.lock().unwrap());
        self.shared.signal.lock().unwrap().last_change = Some(Instant::now());
        self.shared.wake.notify_all();
        result
    }

    pub fn status(&self) -> SaveStatus {
        self.status.lock().unwrap().clone()
    }
}

impl Drop for Autosave {
    fn drop(&mut self) {
        self.shared.signal.lock().unwrap().stop = true;
        self.shared.wake.notify_all();
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

// The worker waits for a change, waits for the quiet time, then saves.
// The locks are never held together: the signal is released before the
// session is locked.
fn run(shared: &Shared) {
    let mut signal = shared.signal.lock().unwrap();
    while !signal.stop {
        match signal.last_change {
            None => signal = shared.wake.wait(signal).unwrap(),
            Some(last) => {
                let due = last + shared.debounce;
                let now = Instant::now();
                if now < due {
                    signal = shared.wake.wait_timeout(signal, due - now).unwrap().0;
                } else {
                    signal.last_change = None;
                    drop(signal);
                    shared.session.lock().unwrap().flush();
                    signal = shared.signal.lock().unwrap();
                }
            }
        }
    }
    drop(signal);
    shared.session.lock().unwrap().flush();
}
