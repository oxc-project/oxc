use oxc_allocator::{Allocator, FromIn};
use oxc_ast::{VisitMut, ast::*};
use rustc_hash::FxHashMap;

use crate::{InlineString, base54};

pub struct PrivateClassNameMangler<'a> {
    allocator: &'a Allocator,
    rename_map: FxHashMap<Atom<'a>, Atom<'a>>,
    count: usize,
}

impl<'a> PrivateClassNameMangler<'a> {
    pub fn new(allocator: &'a Allocator) -> Self {
        Self { allocator, rename_map: FxHashMap::default(), count: 0 }
    }

    fn new_name(&mut self) -> InlineString<12> {
        let new_name = base54(self.count);
        self.count += 1;
        new_name
    }

    pub fn build(&mut self, program: &mut Program<'a>) {
        self.visit_program(program);
    }
}

impl<'a> VisitMut<'a> for PrivateClassNameMangler<'a> {
    fn visit_private_identifier(&mut self, node: &mut PrivateIdentifier<'a>) {
        if let Some(new_name) = self.rename_map.get(&node.name) {
            node.name = Atom::from(new_name.as_str());
        } else {
            let new_name = self.new_name();
            let new_name = Atom::from_in(new_name.as_str(), self.allocator);
            self.rename_map.insert(node.name, new_name);
            node.name = new_name;
        }
    }
}
