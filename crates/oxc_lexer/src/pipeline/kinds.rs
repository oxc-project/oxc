/// One token kind byte per source byte; no method checks bounds.
#[derive(Clone, Copy)]
pub(super) struct Kinds(*mut u8);

impl Kinds {
    pub(super) fn new(ptr: *mut u8) -> Self {
        Self(ptr)
    }

    pub(super) fn as_ptr(self) -> *mut u8 {
        self.0
    }

    #[inline(always)]
    pub(super) unsafe fn get(self, i: usize) -> u8 {
        *self.0.add(i)
    }

    #[inline(always)]
    pub(super) unsafe fn set(self, i: usize, kind: u8) {
        *self.0.add(i) = kind;
    }
}
