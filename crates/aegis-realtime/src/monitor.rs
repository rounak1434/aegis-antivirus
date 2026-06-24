//! Background monitor: watches folders (notify) and new processes (sysinfo),
//! debounces, and feeds events to the [`RealtimeEngine`].

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use notify::{EventKind, RecursiveMode, Watcher};

use crate::model::{FileEvent, FileEventKind, ProcessEvent};
use crate::pipeline::RealtimeEngine;

const DEBOUNCE: Duration = Duration::from_millis(500);
const PROCESS_POLL: Duration = Duration::from_millis(1000);

/// De-duplicates rapid repeat events for the same path within a time window.
#[derive(Default)]
pub struct Debouncer {
    last: Mutex<HashMap<String, Instant>>,
    window: Duration,
}

impl Debouncer {
    pub fn new(window: Duration) -> Self {
        Self { last: Mutex::new(HashMap::new()), window }
    }

    /// True if this path should be processed now (not a recent duplicate).
    pub fn should_process(&self, path: &str) -> bool {
        let now = Instant::now();
        let mut map = self.last.lock().unwrap();
        match map.get(path) {
            Some(&t) if now.duration_since(t) < self.window => false,
            _ => {
                map.insert(path.to_string(), now);
                true
            }
        }
    }
}

/// Map a notify event kind to our file-event kind (deletions are ignored).
pub fn map_event_kind(kind: &EventKind) -> Option<FileEventKind> {
    match kind {
        EventKind::Create(_) => Some(FileEventKind::Create),
        EventKind::Modify(notify::event::ModifyKind::Name(_)) => Some(FileEventKind::Rename),
        EventKind::Modify(_) => Some(FileEventKind::Modify),
        _ => None,
    }
}

/// Default watched folders: Downloads, Desktop, Documents, Temp, user profile.
pub fn default_watched_paths() -> Vec<String> {
    let mut out = Vec::new();
    if let Ok(home) = std::env::var("USERPROFILE").or_else(|_| std::env::var("HOME")) {
        for sub in ["Downloads", "Desktop", "Documents"] {
            out.push(format!("{home}\\{sub}"));
        }
        out.push(home);
    }
    if let Ok(temp) = std::env::var("TEMP").or_else(|_| std::env::var("TMP")) {
        out.push(temp);
    }
    out.into_iter().filter(|p| std::path::Path::new(p).is_dir()).collect()
}

/// Owns the background watcher + poller threads.
pub struct RealtimeMonitor {
    engine: RealtimeEngine,
    watched: Vec<String>,
    running: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl RealtimeMonitor {
    pub fn new(engine: RealtimeEngine, watched: Vec<String>) -> Self {
        Self { engine, watched, running: Arc::new(AtomicBool::new(false)), handle: None }
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    pub fn watched_paths(&self) -> &[String] {
        &self.watched
    }

    /// Spawn the watcher + process poller on a background thread.
    pub fn start(&mut self) {
        if self.is_running() {
            return;
        }
        self.running.store(true, Ordering::Relaxed);
        let engine = self.engine.clone();
        let watched = self.watched.clone();
        let running = self.running.clone();

        self.handle = Some(std::thread::spawn(move || {
            let debouncer = Debouncer::new(DEBOUNCE);
            let (tx, rx) = std::sync::mpsc::channel();
            let mut watcher = match notify::recommended_watcher(move |res| {
                let _ = tx.send(res);
            }) {
                Ok(w) => w,
                Err(_) => return,
            };
            for dir in &watched {
                let _ = watcher.watch(std::path::Path::new(dir), RecursiveMode::Recursive);
            }

            let mut sys = sysinfo::System::new();
            sys.refresh_processes(sysinfo::ProcessesToUpdate::All);
            let mut seen_pids: HashSet<u32> =
                sys.processes().keys().map(|p| p.as_u32()).collect();
            let mut last_poll = Instant::now();

            while running.load(Ordering::Relaxed) {
                if let Ok(Ok(event)) = rx.recv_timeout(Duration::from_millis(200)) {
                    if let Some(kind) = map_event_kind(&event.kind) {
                        for path in event.paths {
                            let p = path.display().to_string();
                            if debouncer.should_process(&p) {
                                engine.handle_file_event(&FileEvent { kind, path: p });
                            }
                        }
                    }
                }

                if last_poll.elapsed() >= PROCESS_POLL {
                    sys.refresh_processes(sysinfo::ProcessesToUpdate::All);
                    for (pid, proc_) in sys.processes() {
                        let id = pid.as_u32();
                        if seen_pids.insert(id) {
                            let cmd = proc_
                                .cmd()
                                .iter()
                                .map(|s| s.to_string_lossy())
                                .collect::<Vec<_>>()
                                .join(" ");
                            engine.handle_process_event(&ProcessEvent {
                                pid: id,
                                name: proc_.name().to_string_lossy().to_string(),
                                exe_path: proc_.exe().map(|e| e.display().to_string()).unwrap_or_default(),
                                command_line: cmd,
                            });
                        }
                    }
                    last_poll = Instant::now();
                }
            }
        }));
    }

    /// Stop the background threads and wait for them to finish.
    pub fn stop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for RealtimeMonitor {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debouncer_blocks_rapid_duplicates() {
        let d = Debouncer::new(Duration::from_secs(10));
        assert!(d.should_process("a.txt"));
        assert!(!d.should_process("a.txt")); // duplicate within window
        assert!(d.should_process("b.txt")); // different path
    }

    #[test]
    fn debouncer_allows_after_window() {
        let d = Debouncer::new(Duration::from_millis(1));
        assert!(d.should_process("x"));
        std::thread::sleep(Duration::from_millis(5));
        assert!(d.should_process("x"));
    }

    #[test]
    fn event_kind_mapping() {
        use notify::event::{CreateKind, ModifyKind, RemoveKind, RenameMode};
        assert_eq!(map_event_kind(&EventKind::Create(CreateKind::File)), Some(FileEventKind::Create));
        assert_eq!(
            map_event_kind(&EventKind::Modify(ModifyKind::Name(RenameMode::Both))),
            Some(FileEventKind::Rename)
        );
        assert_eq!(
            map_event_kind(&EventKind::Modify(ModifyKind::Data(notify::event::DataChange::Any))),
            Some(FileEventKind::Modify)
        );
        assert_eq!(map_event_kind(&EventKind::Remove(RemoveKind::File)), None);
    }
}
