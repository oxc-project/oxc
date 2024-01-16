use rustc_hash::FxHashMap;

/// The `settings` field from ESLint config
#[derive(Debug, Clone)]
pub struct LintSettings {
    pub jsx_a11y: JsxA11y,
}

impl Default for LintSettings {
    fn default() -> Self {
        Self { jsx_a11y: JsxA11y { polymorphic_prop_name: None, components: FxHashMap::default() } }
    }
}

impl LintSettings {
    pub fn new(jsx_a11y: JsxA11y) -> Self {
        Self { jsx_a11y }
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
