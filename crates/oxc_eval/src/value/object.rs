use std::rc::Rc;

/// ## References
/// - [EMCA-262 4.3.1 Objects](https://262.ecma-international.org/15.0/index.html#sec-objects)
#[derive(Debug, Clone, Hash)]
pub struct Object {
    prototype: Option<Rc<Object>>,
    // properties: Vec<Property>,
    // name: String,
}
impl Object {
    pub fn name(&self) -> &str {
        "Object"
    }
}
