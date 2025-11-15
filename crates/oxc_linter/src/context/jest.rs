use std::{cell::OnceCell, rc::Rc};

use crate::utils::PossibleJestNode;

pub struct Jest<'a> {
    possible_jest_nodes: OnceCell<Rc<[PossibleJestNode<'a, 'a>]>>,
}

impl Default for Jest<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> Jest<'a> {
    pub fn new() -> Self {
        Self { possible_jest_nodes: OnceCell::new() }
    }

    pub fn set_possible_jest_nodes(
        &self,
        nodes: Vec<PossibleJestNode<'a, 'a>>,
    ) -> Rc<[PossibleJestNode<'a, 'a>]> {
        let nodes: Rc<[PossibleJestNode<'a, 'a>]> = Rc::from(nodes.into_boxed_slice());
        let _ = self.possible_jest_nodes.set(Rc::clone(&nodes));
        nodes
    }

    pub fn possible_jest_nodes(&self) -> Option<Rc<[PossibleJestNode<'a, 'a>]>> {
        self.possible_jest_nodes.get().map(Rc::clone)
    }
}
