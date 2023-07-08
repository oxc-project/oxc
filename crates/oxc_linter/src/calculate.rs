/// FROM https://github.com/jiachao2130/path-calculate/blob/master/Cargo.toml
/// TODO: upstream? or something else? not sure...
use std::{
    env,
    io::{self, ErrorKind},
    path::{Component, Path, PathBuf},
};

//extern crate path_absolutize;
use path_absolutize::Absolutize;

/// Let `Path` and `PathBuf` have some path calculate methods.
pub trait Calculate {
    /// Get current env's home_dir if it exist.
    fn home_dir(&self) -> io::Result<PathBuf>;

    /// Get the absolute path, even if the path is not exist.
    fn as_absolute_path(&self) -> io::Result<PathBuf>;

    /// Get a relative root path betwwen two pathes.
    fn relative_root_with(&self, path_b: &Path) -> io::Result<PathBuf>;

    /// Get dst_path's relative path from the src_path.
    fn related_to(&self, src_path: &Path) -> io::Result<PathBuf>;
}

impl Calculate for Path {
    fn home_dir(&self) -> io::Result<PathBuf> {
        #[allow(deprecated)]
        let home_dir = env::home_dir().unwrap();
        if home_dir.to_str().unwrap() == "" {
            // do not set or support env $HOME/~
            return Err(io::Error::from(ErrorKind::InvalidInput));
        }

        Ok(home_dir)
    }

    fn as_absolute_path(&self) -> io::Result<PathBuf> {
        let mut iters = self.components();

        let first_component = iters.next();

        // if not start with `~`, return self.absolutize() directly.
        match first_component {
            Some(Component::Normal(dir)) => {
                if dir.to_str().unwrap() == "~" {
                } else {
                    return Ok(self.absolutize()?.to_path_buf());
                }
            }
            None | Some(Component::RootDir) => return Ok(self.absolutize()?.to_path_buf()),
            _ => {}
        }

        // here get replace HOME by abs_path
        let mut path_buf = PathBuf::new();

        let home_dir = self.home_dir()?;
        let home_iters = home_dir.components();
        for iter in home_iters {
            path_buf.push(iter);
        }

        for iter in iters {
            path_buf.push(iter);
        }

        Ok(path_buf)
    }

    fn relative_root_with(&self, path_b: &Path) -> io::Result<PathBuf> {
        // Absolutize
        let pa = self.as_absolute_path()?;
        let pb = path_b.as_absolute_path()?;

        // new pathbuf
        let mut path_buf = PathBuf::new();

        let mut itera = pa.components();
        let mut iterb = pb.components();

        let first_componenta = itera.next().unwrap();
        let first_componentb = iterb.next().unwrap();

        // On Windows, do not support diff Prefix Pathes calculate.
        if first_componenta == first_componentb {
            path_buf.push(first_componenta);
        } else {
            return Err(io::Error::from(ErrorKind::InvalidInput));
        }

        for componenta in itera {
            if let Some(componentb) = iterb.next() {
                if componenta == componentb {
                    path_buf.push(componenta);
                } else {
                    break;
                }
            }
        }

        Ok(path_buf)
    }

    fn related_to(&self, src_path: &Path) -> io::Result<PathBuf> {
        // /home/cc/work/a related_to /home/cc => work/a
        // /home/cc/work/a related_to /home/cc/App/demo => ../../work/a
        // return a absolutily path
        let pa = self.as_absolute_path().unwrap();
        let pb = src_path.as_absolute_path().unwrap();
        let relative_root = self.relative_root_with(src_path).unwrap();

        let mut path_buf = PathBuf::new();

        // pop relative_root
        let mut itera = pa.components();
        let mut iterb = pb.components();

        let iterr = relative_root.components();

        // drop same root
        for _item in iterr {
            itera.next();
            iterb.next();
        }

        // from src to relative_root
        for _item in iterb {
            path_buf.push(Component::ParentDir);
        }

        // relative_root to self
        for item in itera {
            path_buf.push(item);
        }

        Ok(path_buf)
    }
}

impl Calculate for PathBuf {
    fn home_dir(&self) -> io::Result<PathBuf> {
        self.as_path().home_dir()
    }

    fn as_absolute_path(&self) -> io::Result<PathBuf> {
        self.as_path().as_absolute_path()
    }

    fn relative_root_with(&self, path_b: &Path) -> io::Result<PathBuf> {
        self.as_path().relative_root_with(path_b)
    }

    fn related_to(&self, src_path: &Path) -> io::Result<PathBuf> {
        self.as_path().related_to(src_path)
    }
}
