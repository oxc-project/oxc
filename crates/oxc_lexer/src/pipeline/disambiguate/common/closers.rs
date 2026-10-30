use std::cell::RefCell;

use rustc_hash::FxHashMap;

use crate::pipeline::cpu::{Cpu, apply, cpu_versions};

#[derive(Clone, Copy)]
pub struct Resolved {
    closer: Option<u32>,
    pub args: Option<bool>,
}

impl Resolved {
    #[apply(cpu_versions)]
    pub fn closer(self, _cpu: impl Cpu) -> Option<usize> {
        self.closer.map(|c| c as usize)
    }
}

pub struct Open {
    pub pos: u32,
    pub parens: i32,
    pub brackets: i32,
    pub braces: i32,
    /// A run of ruled-out <s, kept for the depth of the lists around them.
    pub dead: u32,
}

#[derive(Default)]
pub struct Closers {
    memo: RefCell<FxHashMap<u32, Resolved>>,
    /// Scratch stacks of the forward passes, kept across queries.
    pub lists: RefCell<Vec<Open>>,
    pub groups: RefCell<Vec<u32>>,
}

impl Closers {
    #[apply(cpu_versions)]
    pub fn clear(&mut self, _cpu: impl Cpu) {
        self.memo.get_mut().clear();
    }

    #[apply(cpu_versions)]
    #[inline]
    pub fn get(&self, pos: usize, _cpu: impl Cpu) -> Option<Resolved> {
        let memo = self.memo.borrow();
        if memo.is_empty() {
            return None;
        }
        memo.get(&(pos as u32)).copied()
    }

    #[apply(cpu_versions)]
    pub fn set(&self, pos: usize, closer: Option<usize>, _cpu: impl Cpu) {
        let closer = closer.map(|c| c as u32);
        self.memo.borrow_mut().insert(pos as u32, Resolved { closer, args: None });
    }

    #[apply(cpu_versions)]
    pub fn set_args(&self, lt: usize, yes: bool, _cpu: impl Cpu) {
        let mut memo = self.memo.borrow_mut();
        if memo.is_empty() {
            return;
        }
        if let Some(r) = memo.get_mut(&(lt as u32)) {
            r.args = Some(yes);
        }
    }
}
