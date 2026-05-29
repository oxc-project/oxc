#[derive(Debug, Clone, Copy, Default)]
pub struct MangleOptionsKeepNames {
    /// Preserve `name` property for functions.
    ///
    /// Default `false`
    pub function: bool,

    /// Preserve `name` property for classes.
    ///
    /// Default `false`
    pub class: bool,
}

impl MangleOptionsKeepNames {
    pub fn all_false() -> Self {
        Self { function: false, class: false }
    }

    pub fn all_true() -> Self {
        Self { function: true, class: true }
    }
}

impl From<bool> for MangleOptionsKeepNames {
    fn from(keep_names: bool) -> Self {
        if keep_names { Self::all_true() } else { Self::all_false() }
    }
}
