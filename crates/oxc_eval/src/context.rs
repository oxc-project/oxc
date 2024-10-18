use std::marker::PhantomData;

#[derive(Debug, Default)]
pub struct EvalContext<'a> {
    strict: bool,
    // TODO
    marker: PhantomData<&'a ()>,
}
impl<'a> EvalContext<'a> {
    #[inline]
    fn is_strict(&self) -> bool {
        self.strict
    }

    #[inline]
    pub fn enter_strict(&mut self) {
        self.strict = true;
    }

    #[inline]
    pub fn leave_strict(&mut self, strict: bool) {
        self.strict = strict;
    }
}
