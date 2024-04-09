mod display_name;
mod jsx;
mod jsx_self;
mod jsx_source;
mod options;

pub use self::{
    display_name::{ReactDisplayName, ReactDisplayNameOptions},
    jsx::ReactJsx,
    jsx_self::{ReactJsxSelf, ReactJsxSelfOptions},
    jsx_source::{ReactJsxSource, ReactJsxSourceOptions},
    options::ReactOptions,
};

/// [Preset React](https://babel.dev/docs/babel-preset-react)
///
/// This preset includes the following plugins:
///
/// * [plugin-transform-react-jsx](https://babeljs.io/docs/babel-plugin-transform-react-jsx)
/// * [plugin-transform-react-jsx-self](https://babeljs.io/docs/babel-plugin-transform-react-jsx-self)
/// * [plugin-transform-react-jsx](https://babeljs.io/docs/babel-plugin-transform-react-jsx)
/// * [plugin-transform-react-display-name](https://babeljs.io/docs/babel-plugin-transform-react-display-name)
#[derive(Default)]
pub struct React {
    jsx: ReactJsx,
    jsx_self: ReactJsxSelf,
    jsx_source: ReactJsxSource,
    display_name: ReactDisplayName,
}

impl React {
    pub fn new(&mut self, options: ReactOptions) -> &mut Self {
        self.jsx = ReactJsx::new(options);
        self
    }

    pub fn with_jsx_self(&mut self, options: ReactJsxSelfOptions) -> &mut Self {
        self.jsx_self = ReactJsxSelf::new(options);
        self
    }

    pub fn with_jsx_source(&mut self, options: ReactJsxSourceOptions) -> &mut Self {
        self.jsx_source = ReactJsxSource::new(options);
        self
    }

    pub fn with_display_name(&mut self, options: ReactDisplayNameOptions) -> &mut Self {
        self.display_name = ReactDisplayName::new(options);
        self
    }
}
