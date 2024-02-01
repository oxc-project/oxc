use rustc_hash::FxHashMap;

/// The `settings` field from ESLint config
///
/// An object containing name-value pairs of information that should be available to all rules
#[derive(Debug, Clone)]
pub struct ESLintSettings {
    pub jsx_a11y: JsxA11y,
    pub nextjs: Nextjs,
    pub link_components: CustomComponents,
    pub form_components: CustomComponents,
}

impl Default for ESLintSettings {
    fn default() -> Self {
        Self {
            jsx_a11y: JsxA11y { polymorphic_prop_name: None, components: FxHashMap::default() },
            nextjs: Nextjs { root_dir: vec![] },
            link_components: FxHashMap::default(),
            form_components: FxHashMap::default(),
        }
    }
}

impl ESLintSettings {
    pub fn new(
        jsx_a11y: JsxA11y,
        nextjs: Nextjs,
        link_components: CustomComponents,
        form_components: CustomComponents,
    ) -> Self {
        Self { jsx_a11y, nextjs, link_components, form_components }
    }
}

#[derive(Debug, Clone)]
pub struct JsxA11y {
    pub polymorphic_prop_name: Option<String>,
    pub components: FxHashMap<String, String>,
}

impl JsxA11y {
    pub fn new(
        polymorphic_prop_name: Option<String>,
        components: FxHashMap<String, String>,
    ) -> Self {
        Self { polymorphic_prop_name, components }
    }

    pub fn set_components(&mut self, components: FxHashMap<String, String>) {
        self.components = components;
    }

    pub fn set_polymorphic_prop_name(&mut self, name: Option<String>) {
        self.polymorphic_prop_name = name;
    }
}

#[derive(Debug, Clone)]
pub struct Nextjs {
    pub root_dir: Vec<String>,
}

impl Nextjs {
    pub fn new(root_dir: Vec<String>) -> Self {
        Self { root_dir }
    }

    pub fn set_root_dir(&mut self, root_dir: Vec<String>) {
        self.root_dir = root_dir;
    }
}

pub type CustomComponents = FxHashMap<String, Vec<String>>;
