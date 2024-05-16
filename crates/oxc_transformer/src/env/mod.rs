mod options;

use std::rc::Rc;

pub use self::options::EnvOptions;
/// [Preset Env](https://babel.dev/docs/babel-preset-env)
///
/// This preset is a smart preset that allows you to use the latest JavaScript without needing to micromanage
/// which syntax transforms (and optionally, browser polyfills) are needed by your target environment(s).
/// This both makes your life easier and JavaScript bundles smaller!
pub struct Env {
    #[allow(unused)]
    options: Rc<EnvOptions>,
}

impl Env {
    pub fn new(options: EnvOptions) -> Self {
        let options = Rc::new(options);
        Self { options }
    }
}
