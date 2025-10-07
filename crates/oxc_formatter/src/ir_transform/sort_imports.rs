use crate::{formatter::format_element::document::Document, options::SortImports};

pub struct SortImportsTransform {
    options: SortImports,
}

impl SortImportsTransform {
    pub fn new(options: SortImports) -> Self {
        Self { options }
    }

    pub fn transform<'a>(&self, document: &Document<'a>) -> Document<'a> {
        let mut new_elements = Vec::with_capacity(document.len());

        // TODO: THESE ARE DUMMY IMPLEMENTATIONS!
        let mut temp = None;
        for (idx, element) in document.iter().enumerate() {
            if idx == 0 {
                temp = Some(element);
                continue;
            }

            new_elements.push(element.clone());
        }

        if let Some(temp) = temp {
            new_elements.insert(new_elements.len() - 1, temp.clone());
        }

        Document::from(new_elements)
    }
}
