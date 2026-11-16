//! A file-backed value that is reloaded when the file it was read from changes.
//!
//! The platform serves stores that offline tooling rewrites underneath it: the corpus index, the
//! packed `.turbovec` store, the `.tercode` store. A `OnceLock` made each of those loads permanent
//! for the life of the process, so a rebuilt store was shadowed by the first read -- and a route
//! could report the file as fresh while scoring the copy it had loaded minutes earlier. That is the
//! claim drawn from one source and the data from another. A cache keyed on the file's own
//! (mtime, length) costs one `stat` per call and always describes what it is serving.

use std::path::Path;
use std::sync::{Arc, Mutex};

/// The identity of a file as it was last read: nanoseconds since the epoch, and its length.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Sig {
    pub mtime_ns: u128,
    pub len: u64,
}

impl Sig {
    pub fn of(p: &Path) -> Option<Sig> {
        let md = std::fs::metadata(p).ok()?;
        let mtime_ns = md
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        Some(Sig { mtime_ns, len: md.len() })
    }

    /// The signature as a reader sees it: whole seconds, so a report can show it beside the mtimes
    /// the staleness checks already print.
    pub fn mtime_s(&self) -> u64 {
        (self.mtime_ns / 1_000_000_000) as u64
    }
}

/// What the last read of a file produced. A rejected load is cached too: a corpus that cannot be
/// parsed must not be re-parsed on every request, but it must be re-tried once the file changes.
pub enum Outcome<T> {
    Read(Arc<T>),
    Failed(String),
}

/// A cache slot for one file-backed value. A `Mutex` because a load can replace it mid-flight.
pub type Slot<T> = Mutex<Option<(Sig, Outcome<T>)>>;

/// The value for `path`, reading it only when the file differs from what was last read.
///
/// `Err` when the file cannot be stat'ed or the loader rejected it: an unreadable file is not an
/// empty one, and a caller that cannot see the file must not be handed a stale copy as if current.
pub fn get<T>(
    slot: &Slot<T>,
    path: &Path,
    load: impl FnOnce(&Path) -> Result<T, String>,
) -> Result<Arc<T>, String> {
    let sig = Sig::of(path).ok_or_else(|| format!("cannot stat {}", path.display()))?;
    if let Ok(g) = slot.lock() {
        if let Some((cached, out)) = g.as_ref() {
            if *cached == sig {
                return match out {
                    Outcome::Read(v) => Ok(Arc::clone(v)),
                    Outcome::Failed(e) => Err(e.clone()),
                };
            }
        }
    }
    let out = match load(path) {
        Ok(v) => Outcome::Read(Arc::new(v)),
        Err(e) => Outcome::Failed(e),
    };
    let answer = match &out {
        Outcome::Read(v) => Ok(Arc::clone(v)),
        Outcome::Failed(e) => Err(e.clone()),
    };
    if let Ok(mut g) = slot.lock() {
        *g = Some((sig, out));
    }
    answer
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write(p: &Path, s: &str) {
        let mut f = std::fs::File::create(p).unwrap();
        f.write_all(s.as_bytes()).unwrap();
    }

    fn len_of(p: &Path) -> Result<usize, String> {
        std::fs::read(p).map(|b| b.len()).map_err(|e| e.to_string())
    }

    #[test]
    fn reloads_when_the_file_changes_and_not_otherwise() {
        let p = std::env::temp_dir().join(format!("file_cache_reload_{}.bin", std::process::id()));
        write(&p, "first");
        let slot: Slot<usize> = Mutex::new(None);
        let a = get(&slot, &p, len_of).unwrap();
        assert_eq!(*a, 5);
        let b = get(&slot, &p, len_of).unwrap();
        assert!(Arc::ptr_eq(&a, &b), "an unchanged file must not be re-read");
        std::thread::sleep(std::time::Duration::from_millis(20));
        write(&p, "second, longer");
        let c = get(&slot, &p, len_of).unwrap();
        assert_eq!(*c, 14, "a rewritten file must be re-read");
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn a_deleted_file_is_refused_not_served_from_the_cache() {
        let p = std::env::temp_dir().join(format!("file_cache_absent_{}.bin", std::process::id()));
        let slot: Slot<usize> = Mutex::new(None);
        write(&p, "here");
        assert!(get(&slot, &p, len_of).is_ok());
        let _ = std::fs::remove_file(&p);
        let err = get(&slot, &p, len_of).unwrap_err();
        assert!(err.starts_with("cannot stat"), "got: {err}");
    }

    #[test]
    fn a_rejected_load_is_cached_until_the_file_changes() {
        let p = std::env::temp_dir().join(format!("file_cache_err_{}.bin", std::process::id()));
        write(&p, "v1");
        let slot: Slot<usize> = Mutex::new(None);
        let calls = std::sync::atomic::AtomicUsize::new(0);
        let fail = |_: &Path| {
            calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Err("loader said no".to_string())
        };
        assert!(get(&slot, &p, fail).is_err());
        assert!(get(&slot, &p, fail).is_err());
        assert_eq!(
            calls.load(std::sync::atomic::Ordering::SeqCst),
            1,
            "a rejected load must not repeat on an unchanged file"
        );
        std::thread::sleep(std::time::Duration::from_millis(20));
        write(&p, "v2 longer");
        assert!(get(&slot, &p, fail).is_err());
        assert_eq!(
            calls.load(std::sync::atomic::Ordering::SeqCst),
            2,
            "a changed file must be re-tried"
        );
        let _ = std::fs::remove_file(&p);
    }
}
