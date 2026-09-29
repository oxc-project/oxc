use std::cell::RefCell;

use rustc_hash::FxHashMap;

#[derive(Clone, Copy)]
pub(crate) struct Resolved {
    closer: Option<u32>,
    pub(crate) args: Option<bool>,
}

impl Resolved {
    pub(crate) fn closer(self) -> Option<usize> {
        self.closer.map(|c| c as usize)
    }
}

pub(crate) struct Open {
    pub(crate) pos: u32,
    pub(crate) parens: i32,
    pub(crate) brackets: i32,
    pub(crate) braces: i32,
    /// A run of ruled-out <s, kept for the depth of the lists around them.
    pub(crate) dead: u32,
}

#[derive(Default)]
pub(crate) struct Closers {
    memo: RefCell<FxHashMap<u32, Resolved>>,
    /// Scratch stacks of the forward passes, kept across queries.
    pub(crate) lists: RefCell<Vec<Open>>,
    pub(crate) groups: RefCell<Vec<u32>>,
}

impl Closers {
    pub(crate) fn clear(&mut self) {
        self.memo.get_mut().clear();
    }

    #[inline]
    pub(crate) fn get(&self, pos: usize) -> Option<Resolved> {
        let memo = self.memo.borrow();
        if memo.is_empty() {
            return None;
        }
        memo.get(&(pos as u32)).copied()
    }

    pub(crate) fn set(&self, pos: usize, closer: Option<usize>) {
        let closer = closer.map(|c| c as u32);
        self.memo.borrow_mut().insert(pos as u32, Resolved { closer, args: None });
    }

    pub(crate) fn set_args(&self, lt: usize, yes: bool) {
        let mut memo = self.memo.borrow_mut();
        if memo.is_empty() {
            return;
        }
        if let Some(r) = memo.get_mut(&(lt as u32)) {
            r.args = Some(yes);
        }
    }
}
