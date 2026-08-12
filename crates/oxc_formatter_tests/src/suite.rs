//! Prettier test-suite provisioning.
//!
//! The conformance tests compare output against the Prettier repository's own test suite (`tests/format`).
//! The pin is the `prettier` version in `apps/oxfmt/package.json`,
//! the same Prettier that oxfmt bundles as the oracle.
//! So the suite and the oracle cannot drift apart.
//! [`ensure_prettier_suite`] downloads the release tarball on demand (degit-style, no git objects, `tests/format` only),
//! so neither CI nor local runs need a separate clone step;
//! a warm checkout is verified offline.
//!
//! Bumping Prettier =
//! bumping `apps/oxfmt/package.json` + regenerating the conformance snapshots against it
//! (they must change together; the suite re-provisions itself).

use std::{
    fs::{self, File},
    path::Path,
    process::Command,
    sync::OnceLock,
};

/// The version pin. Also serves as the cross-process provisioning lock.
const PACKAGE_JSON: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../apps/oxfmt/package.json");

/// Root of the extracted Prettier suite (gitignored). Contains `tests/format`
/// plus a `.version` stamp written after a successful extraction.
#[must_use]
pub fn prettier_suite_root() -> &'static Path {
    Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/prettier"))
}

/// Ensures the suite at [`prettier_suite_root`] matches the pinned Prettier version and returns the root.
/// Convergent:
///
/// - `.version` stamp matches the pin: return immediately — no network, no subprocess
/// - missing / stale (version bumped): wipe, download the release tarball from codeload, extract `tests/format`, stamp
///
/// Cross-process exclusion (parallel test binaries, e.g. under nextest) uses an advisory lock on the `package.json` handle;
/// within a process the result is memoized.
///
/// # Errors
/// Any download/extraction failure, as a display string.
/// Conformance callers fail loudly on `Err`;
/// environments that cannot provision opt out in `ci.yml` via `-- --skip prettier_conformance`.
pub fn ensure_prettier_suite() -> Result<&'static Path, String> {
    static RESULT: OnceLock<Result<(), String>> = OnceLock::new();
    RESULT.get_or_init(provision).clone()?;
    Ok(prettier_suite_root())
}

fn provision() -> Result<(), String> {
    let root = prettier_suite_root();
    let stamp = root.join(".version");

    // Advisory lock; released when the handle drops. Locking the pin file itself
    // spares a separate (gitignore-managed) lockfile, and the handle doubles as the
    // single source of the pinned version.
    let pkg_json = File::open(PACKAGE_JSON).map_err(|e| format!("open {PACKAGE_JSON}: {e}"))?;
    pkg_json.lock().map_err(|e| format!("lock {PACKAGE_JSON}: {e}"))?;
    let version = prettier_version(&pkg_json)?;

    if fs::read_to_string(&stamp).is_ok_and(|s| s.trim() == version) {
        return Ok(());
    }

    // Wipe-and-extract keeps this convergent;
    // the stamp is written last, so a half-provisioned tree is always re-done.
    // Only the CONTENTS are wiped: in CI the root is a cache-volume mount point, and removing it fails with EBUSY.
    fs::create_dir_all(root).map_err(|e| format!("create {}: {e}", root.display()))?;
    let read_err = |e| format!("read {}: {e}", root.display());
    for entry in fs::read_dir(root).map_err(read_err)? {
        let path = entry.map_err(read_err)?.path();
        if path.is_dir() { fs::remove_dir_all(&path) } else { fs::remove_file(&path) }
            .map_err(|e| format!("remove {}: {e}", path.display()))?;
    }

    let tarball = std::env::temp_dir().join(format!("oxc-prettier-{version}.tar.gz"));
    let url = format!("https://codeload.github.com/prettier/prettier/tar.gz/refs/tags/{version}");
    run("curl", &["-fsSL", "-o", &tarball.to_string_lossy(), &url], root)?;
    // Extract only what the conformance tests read; drop the `prettier-<version>/` prefix.
    run(
        "tar",
        &[
            "-xzf",
            &tarball.to_string_lossy(),
            "--strip-components=1",
            &format!("prettier-{version}/tests/format"),
        ],
        root,
    )?;
    let _ = fs::remove_file(&tarball);

    fs::write(&stamp, &version).map_err(|e| format!("write {}: {e}", stamp.display()))?;
    Ok(())
}

/// Reads `dependencies.prettier` from the (already-opened) oxfmt package.json.
fn prettier_version(pkg_json: &File) -> Result<String, String> {
    let json: serde_json::Value =
        serde_json::from_reader(pkg_json).map_err(|e| format!("parse {PACKAGE_JSON}: {e}"))?;
    let version = json["dependencies"]["prettier"]
        .as_str()
        .ok_or_else(|| format!("no dependencies.prettier in {PACKAGE_JSON}"))?;
    // The tarball URL needs an exact tag; a semver range would mean the pin is gone.
    if !version.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        return Err(format!("dependencies.prettier must be an exact version, got {version}"));
    }
    Ok(version.to_string())
}

fn run(program: &str, args: &[&str], cwd: &Path) -> Result<(), String> {
    let output = Command::new(program)
        .args(args)
        .current_dir(cwd)
        .output()
        .map_err(|e| format!("spawn {program}: {e}"))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "{program} {}: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        ))
    }
}
