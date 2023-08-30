use std::path::{Component, Path, PathBuf};

use path_calculate::Calculate;

/// Makes relative path part vec with a path relative to the current directory.
///
/// ie: if current directory = /foo/
///    and path = /foo/bar/baz.txt
///   then relative path parts = [Some("bar"), Some("baz.txt")]
///
/// # Panics
/// Panics if the path component has any non-normal component after
/// a single normal path component.
pub fn make_relative_path_parts(path: &PathBuf) -> Vec<Option<String>> {
    path.related_to(Path::new("."))
        .expect("to be able to get the relative path parts")
        .components()
        .skip_while(|x| !matches!(x, Component::Normal(_)))
        .map(|path_component| {
            let Component::Normal(component) = path_component else {
                unreachable!("there should only be normal path components here")
            };
            component.to_str().map(ToOwned::to_owned)
        })
        .collect::<Vec<_>>()
}
