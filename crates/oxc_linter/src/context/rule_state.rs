use std::{
    any::Any,
    cell::{RefCell, RefMut},
};

use nohash_hasher::IntMap;

use crate::{rules::RuleEnum, RuleMeta};

#[must_use]
pub struct RuleState {
    inner: IntMap</* RuleId */ usize, Box<RefCell<dyn Any>>>,
}

impl RuleState {
    pub fn new<I, R>(rules: I) -> Self
    where
        R: AsRef<RuleEnum>,
        I: IntoIterator<Item = R>,
    {
        let inner = rules
            .into_iter()
            .map(|rule| {
                let rule = rule.as_ref();
                (rule.id(), rule.new_state())
            })
            .collect();
        Self { inner }
    }

    pub fn get_raw_mut(&self, rule_id: usize) -> RefMut<'_, dyn Any> {
        self.inner[&rule_id].borrow_mut()
    }

    pub fn get_mut<R: RuleMeta + 'static>(&self, rule_id: usize) -> RefMut<'_, R::State> {
        RefMut::map(self.get_raw_mut(rule_id), |state| {
            state.downcast_mut().expect("downcast failed")
        })
    }
}
