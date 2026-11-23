//! A cache directory: its files are checksummed `postcard`, written atomically.

use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    time::SystemTime,
};

use serde::{Serialize, de::DeserializeOwned};

use crate::digest::Digest;

/// A cache's directory.
#[derive(Debug)]
pub struct CacheDir {
    /// Where it is: every name read or written is relative to it.
    path: PathBuf,
}

impl CacheDir {
    /// Opens the directory at `path`, creating it if needed. Returns `None` if it can't be created.
    pub fn open(path: &Path) -> Option<Self> {
        fs::create_dir_all(path).ok()?;
        Some(Self { path: path.to_path_buf() })
    }

    /// Reads the value in `name`, a path relative to the directory.
    /// Returns `None` if it's missing or corrupt: a miss, never a wrong value.
    pub fn read<T: DeserializeOwned>(&self, name: &str) -> Option<T> {
        let bytes = fs::read(self.path.join(name)).ok()?;
        let (checksum, payload) = bytes.split_at_checked(32)?;
        if Digest::of_bytes(payload).as_ref() != checksum {
            return None;
        }
        postcard::from_bytes(payload).ok()
    }

    /// Writes `value` to `name` through a temporary file and a rename, so readers see either the
    /// old or the new file. Failures are ignored: a cache only saves work.
    pub fn write(&self, name: &str, value: &impl Serialize) {
        let Ok(payload) = postcard::to_stdvec(value) else { return };
        let path = self.path.join(name);
        let Some(parent) = path.parent() else { return };
        let _ = fs::create_dir_all(parent);
        let _ = tempfile::NamedTempFile::new_in(parent).and_then(|mut file| {
            file.write_all(Digest::of_bytes(&payload).as_ref())?;
            file.write_all(&payload)?;
            file.persist(&path)?;
            Ok(())
        });
    }

    /// Removes all but the `keep` most recently written files in `subdir`.
    pub fn prune(&self, subdir: &str, keep: usize) {
        let Ok(entries) = fs::read_dir(self.path.join(subdir)) else { return };
        let mut files: Vec<(SystemTime, PathBuf)> = entries
            .filter_map(|entry| {
                let entry = entry.ok()?;
                Some((entry.metadata().ok()?.modified().ok()?, entry.path()))
            })
            .collect();
        if files.len() > keep {
            files.sort_unstable_by_key(|(modified, _)| std::cmp::Reverse(*modified));
            for (_, path) in files.drain(keep..) {
                let _ = fs::remove_file(path);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_what_it_wrote_and_nothing_corrupt() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = CacheDir::open(tmp.path()).unwrap();
        dir.write("value", &(1u32, "a".to_string()));
        assert_eq!(dir.read::<(u32, String)>("value"), Some((1, "a".to_string())));

        let path = dir.path.join("value");
        let mut bytes = fs::read(&path).unwrap();
        *bytes.last_mut().unwrap() ^= 1;
        fs::write(&path, &bytes).unwrap();
        assert_eq!(dir.read::<(u32, String)>("value"), None);
        fs::write(&path, &bytes[..10]).unwrap();
        assert_eq!(dir.read::<(u32, String)>("value"), None);
    }

    /// Whatever bytes of a file change, or however it's cut short, it reads as nothing:
    /// a corrupt cache is a miss, never a wrong value.
    #[test]
    fn any_corruption_reads_as_nothing() {
        #[expect(clippy::needless_pass_by_value, reason = "quickcheck passes values")]
        fn property(value: (Vec<String>, u64, bool), flip: (usize, u8), cut: usize) -> bool {
            let tmp = tempfile::tempdir().unwrap();
            let dir = CacheDir::open(tmp.path()).unwrap();
            dir.write("value", &value);
            if dir.read::<(Vec<String>, u64, bool)>("value").as_ref() != Some(&value) {
                return false;
            }
            let path = dir.path.join("value");
            let bytes = fs::read(&path).unwrap();
            let mut flipped = bytes.clone();
            let at = flip.0 % flipped.len();
            flipped[at] ^= flip.1.max(1);
            fs::write(&path, &flipped).unwrap();
            let corrupt = dir.read::<(Vec<String>, u64, bool)>("value").is_none();
            fs::write(&path, &bytes[..cut % bytes.len()]).unwrap();
            corrupt && dir.read::<(Vec<String>, u64, bool)>("value").is_none()
        }
        quickcheck::QuickCheck::new()
            .tests(300)
            .quickcheck(property as fn((Vec<String>, u64, bool), (usize, u8), usize) -> bool);
    }

    #[test]
    fn prune_keeps_the_most_recently_written() {
        const KEEP: usize = 8;
        let tmp = tempfile::tempdir().unwrap();
        let dir = CacheDir::open(tmp.path()).unwrap();
        for i in 0..KEEP + 5 {
            dir.write(&format!("runs/{i}"), &i);
            let time = SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(i as u64 + 1);
            fs::File::options()
                .write(true)
                .open(dir.path.join(format!("runs/{i}")))
                .unwrap()
                .set_modified(time)
                .unwrap();
        }
        dir.prune("runs", KEEP);
        assert_eq!(fs::read_dir(dir.path.join("runs")).unwrap().count(), KEEP);
        assert!(dir.read::<usize>("runs/4").is_none(), "the oldest go");
        assert_eq!(dir.read::<usize>(&format!("runs/{}", KEEP + 4)), Some(KEEP + 4));
    }
}
