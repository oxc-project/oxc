//! An FS that lets a few threads at a time read files from the one below it (macOS only).
//!
//! Opening files does not scale on macOS: building the monolith's program opens 40,000 files, which takes 0.5 s on 4
//! threads and 0.4 s on 18 (18 processes contend just like 18 threads), but the kernel spends about 2 s of system
//! time on the opens on 4 threads and 5.5 s on 18 (an 18-core Mac). Other calls (stat, directory listings) are cheap
//! enough that waiting for a turn costs more than it saves.

use std::sync::{Arc, Condvar, Mutex};
use std::time::SystemTime;

use tsrs_vfs::{Entries, FS, FileInfo};

pub struct BoundedReadsFS {
    inner: Arc<dyn FS>,
    free: Mutex<usize>,
    freed: Condvar,
}

impl BoundedReadsFS {
    pub fn new(inner: Arc<dyn FS>, readers: usize) -> BoundedReadsFS {
        assert!(readers > 0);
        BoundedReadsFS { inner, free: Mutex::new(readers), freed: Condvar::new() }
    }
}

impl FS for BoundedReadsFS {
    fn read_file(&self, path: &str) -> Option<String> {
        {
            let mut free = self.free.lock().unwrap();
            while *free == 0 {
                free = self.freed.wait(free).unwrap();
            }
            *free -= 1;
        }
        // Give the turn back even if the read panics.
        struct Release<'a>(&'a BoundedReadsFS);
        impl Drop for Release<'_> {
            fn drop(&mut self) {
                *self.0.free.lock().unwrap() += 1;
                self.0.freed.notify_one();
            }
        }
        let _release = Release(self);
        self.inner.read_file(path)
    }
    fn use_case_sensitive_file_names(&self) -> bool {
        self.inner.use_case_sensitive_file_names()
    }
    fn file_exists(&self, path: &str) -> bool {
        self.inner.file_exists(path)
    }
    fn directory_exists(&self, path: &str) -> bool {
        self.inner.directory_exists(path)
    }
    fn stat(&self, path: &str) -> Option<FileInfo> {
        self.inner.stat(path)
    }
    fn get_accessible_entries(&self, path: &str) -> Entries {
        self.inner.get_accessible_entries(path)
    }
    fn realpath(&self, path: &str) -> String {
        self.inner.realpath(path)
    }
    fn write_file(&self, path: &str, data: &str) -> Result<(), String> {
        self.inner.write_file(path, data)
    }
    fn append_file(&self, path: &str, data: &str) -> Result<(), String> {
        self.inner.append_file(path, data)
    }
    fn remove(&self, path: &str) -> Result<(), String> {
        self.inner.remove(path)
    }
    fn chtimes(&self, path: &str, a: SystemTime, m: SystemTime) -> Result<(), String> {
        self.inner.chtimes(path, a, m)
    }
}
