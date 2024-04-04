pub struct Es2022Options {
    /// https://babeljs.io/docs/babel-plugin-transform-class-properties
    pub class_properties: bool,
    /// https://babeljs.io/docs/babel-plugin-transform-class-static-block
    pub class_static_block: bool,
    /// https://babeljs.io/docs/babel-plugin-transform-private-methods
    pub class_private_methods: bool,
    /// https://babeljs.io/docs/babel-plugin-transform-private-property-in-object
    pub class_private_properties: bool,
}

impl Default for Es2022Options {
    fn default() -> Self {
        Self {
            class_properties: true,
            class_static_block: true,
            class_private_methods: true,
            class_private_properties: true,
        }
    }
}
