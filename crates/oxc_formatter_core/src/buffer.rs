use crate::format_element::FormatElement;

pub trait Buffer {
    fn write_element(&mut self, element: FormatElement);

    fn elements(&self) -> &[FormatElement];
}

#[derive(Debug, Default)]
pub struct VecBuffer {
    elements: Vec<FormatElement>,
}

impl VecBuffer {
    pub fn into_vec(self) -> Vec<FormatElement> {
        self.elements
    }
}

impl Buffer for VecBuffer {
    fn write_element(&mut self, element: FormatElement) {
        self.elements.push(element);
    }

    fn elements(&self) -> &[FormatElement] {
        &self.elements
    }
}
