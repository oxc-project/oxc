use std::collections::VecDeque;

use itertools::Itertools;

use crate::{schema::RType, CodegenCtx, Result};

impl CodegenCtx {
    pub fn pass<P>(self, mut pass: P) -> Result<Self>
    where
        P: FnMut(&mut RType, &Self) -> Result<bool>,
    {
        // we sort by `TypeId` so we always have the same ordering as how it is written in the rust.
        let mut unresolved = self
            .ident_table
            .iter()
            .sorted_by_key(|it| it.1)
            .map(|it| it.0)
            .collect::<VecDeque<_>>();

        while let Some(next) = unresolved.pop_back() {
            let next_id = *self.type_id(next).unwrap();

            let val = &mut self.ty_table[next_id].borrow_mut();

            if !pass(val, &self)? {
                unresolved.push_front(next);
            }
        }
        Ok(self)
    }
}
