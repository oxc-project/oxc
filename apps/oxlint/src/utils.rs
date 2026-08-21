use std::path::{Component, Path, PathBuf};

use oxc_allocator::AllocatorPool;
use oxc_linter::AllocatorPools;

/// Create the allocator pools for a lint run.
///
/// JS plugins need fixed-size arenas, because those are the only arenas that can be shared with JS
/// via raw transfer. Which pool is fixed-size depends on whether multi-file analysis is enabled:
///
/// * JS plugins, no `import` plugin: parse straight into fixed-size arenas. At most `thread_count`
///   ASTs are live at once, so `thread_count` arenas suffice.
/// * JS plugins and `import` plugin: parse into standard arenas (many ASTs stay live at once), and
///   copy into a fixed-size arena only when handing a file to JS.
/// * No JS plugins: standard arenas throughout.
///
/// Each fixed-size arena carries a buffer id that `ExternalLinter` uses to route a file
/// to the isolate that owns that arena.
pub fn create_allocator_pools(has_js_plugins: bool, cross_module: bool) -> AllocatorPools {
    let thread_count = rayon::current_num_threads();

    #[cfg(all(target_pointer_width = "64", target_endian = "little"))]
    if has_js_plugins {
        return if cross_module {
            AllocatorPools {
                parse: AllocatorPool::new(thread_count),
                js: Some(AllocatorPool::new_fixed_size(thread_count)),
            }
        } else {
            AllocatorPools { parse: AllocatorPool::new_fixed_size(thread_count), js: None }
        };
    }

    // JS plugins are unsupported on these platforms, so fixed-size arenas are never needed.
    #[cfg(not(all(target_pointer_width = "64", target_endian = "little")))]
    let _ = (has_js_plugins, cross_module);

    AllocatorPools { parse: AllocatorPool::new(thread_count), js: None }
}

/// Normalize a path by removing `.` and resolving `..` components,
/// without touching the filesystem.
pub fn normalize_path<P: AsRef<Path>>(path: P) -> PathBuf {
    let mut result = PathBuf::new();

    for component in path.as_ref().components() {
        match component {
            Component::ParentDir => {
                result.pop();
            }
            Component::CurDir => {
                // Skip current directory component
            }
            Component::Normal(c) => {
                result.push(c);
            }
            Component::RootDir | Component::Prefix(_) => {
                result.push(component.as_os_str());
            }
        }
    }

    result
}

#[cfg(test)]
mod test {
    use std::path::Path;

    use super::normalize_path;

    #[test]
    fn test_normalize_path() {
        assert_eq!(
            normalize_path(Path::new("/root/directory/./.oxlintrc.json")),
            Path::new("/root/directory/.oxlintrc.json")
        );
        assert_eq!(
            normalize_path(Path::new("/root/directory/../.oxlintrc.json")),
            Path::new("/root/.oxlintrc.json")
        );
    }
}
