use core::ptr;

use crate::error::Diagnostic;

#[repr(C)]
#[derive(Clone, Copy, Default, Debug)]
pub struct LineEntry {
    pub off: u32,
    pub line: u32,
}

#[derive(Debug)]
pub struct LexResult {
    pub diagnostics: *mut Diagnostic,
    pub diagnostic_count: u32,
    pub lines: *mut LineEntry,
    pub line_count: u32,
    pub hit_resource_limit: bool,

    pub token_count: u32,
    pub numbers_count: u32,
    pub atoms_count: u32,
    pub strings_count: u32,
    pub templates_count: u32,
    pub regex_flags_count: u32,
    pub comment_meta_count: u32,
    pub comments_count: u32,
    pub cooked_bytes_count: u32,
}

macro_rules! lane_accessor {
    ($(#[$doc:meta])* $name:ident, $field:ident, $count:ident, $ty:ty) => {
        $(#[$doc])*
        #[must_use]
        pub fn $name<'a>(&'a self, arena: &'a Arena) -> &'a [$ty] {
            if arena.$field.is_null() || self.$count == 0 {
                return &[];
            }
            // SAFETY: the lex that produced `self` initialized exactly `$count` elements (count <= capacity asserted at the copy site).
            unsafe { core::slice::from_raw_parts(arena.$field, self.$count as usize) }
        }
    };
}

impl LexResult {
    #[must_use]
    pub fn diagnostics(&self) -> &[Diagnostic] {
        if self.diagnostics.is_null() || self.diagnostic_count == 0 {
            return &[];
        }
        // SAFETY: the lex that produced `self` copied `diagnostic_count` diagnostics into this buffer.
        unsafe { core::slice::from_raw_parts(self.diagnostics, self.diagnostic_count as usize) }
    }

    lane_accessor!(tok_kinds, tok_kinds, token_count, u8);

    #[must_use]
    pub fn tok_starts<'a>(&'a self, arena: &'a Arena) -> &'a [u32] {
        if arena.tok_starts.is_null() {
            return &[];
        }
        let n = self.token_count as usize;
        let len_with_sentinel = if n == 0 { 0 } else { n + 1 };
        // SAFETY: the lexer wrote `token_count` starts plus the sentinel.
        unsafe { core::slice::from_raw_parts(arena.tok_starts, len_with_sentinel) }
    }

    lane_accessor!(numbers, numbers, numbers_count, f64);
    lane_accessor!(strings, strings, strings_count, crate::token::StringSpan);
    lane_accessor!(cooked_bytes, cooked_bytes, cooked_bytes_count, u8);
    lane_accessor!(templates, templates, templates_count, crate::token::StringSpan);
    lane_accessor!(atoms, atoms, atoms_count, crate::token::StringSpan);
    lane_accessor!(regex_flags, regex_flags, regex_flags_count, oxc_ast::ast::RegExpFlags);
    lane_accessor!(comment_meta, comment_meta, comment_meta_count, u8);
    lane_accessor!(comments, comments, comments_count, oxc_ast::ast::Comment);
}

#[derive(Debug)]
pub struct Arena {
    pub tokens_capacity: u32,
    pub diags: *mut Diagnostic,
    pub diags_capacity: u32,
    pub lines: *mut LineEntry,
    pub lines_capacity: u32,

    pub tok_kinds: *mut u8,
    pub tok_kinds_capacity: u32,
    pub tok_starts: *mut u32,
    pub tok_starts_capacity: u32,
    pub numbers: *mut f64,
    pub numbers_capacity: u32,
    pub atoms: *mut crate::token::StringSpan,
    pub atoms_capacity: u32,
    pub strings: *mut crate::token::StringSpan,
    pub strings_capacity: u32,
    pub cooked_bytes: *mut u8,
    pub cooked_bytes_capacity: u32,
    pub templates: *mut crate::token::StringSpan,
    pub templates_capacity: u32,
    pub regex_flags: *mut oxc_ast::ast::RegExpFlags,
    pub regex_flags_capacity: u32,
    pub comment_meta: *mut u8,
    pub comment_meta_capacity: u32,
    pub comments: *mut oxc_ast::ast::Comment,
    pub comments_capacity: u32,
}

impl Arena {
    #[must_use]
    pub fn new(tokens_capacity: u32, diags_capacity: u32, lines_capacity: u32) -> Self {
        Self {
            tokens_capacity,
            diags: alloc_uninit::<Diagnostic>(diags_capacity),
            diags_capacity,
            lines: alloc_uninit::<LineEntry>(lines_capacity),
            lines_capacity,
            tok_kinds: ptr::null_mut(),
            tok_kinds_capacity: 0,
            tok_starts: ptr::null_mut(),
            tok_starts_capacity: 0,
            numbers: ptr::null_mut(),
            numbers_capacity: 0,
            atoms: ptr::null_mut(),
            atoms_capacity: 0,
            strings: ptr::null_mut(),
            strings_capacity: 0,
            cooked_bytes: ptr::null_mut(),
            cooked_bytes_capacity: 0,
            templates: ptr::null_mut(),
            templates_capacity: 0,
            regex_flags: ptr::null_mut(),
            regex_flags_capacity: 0,
            comment_meta: ptr::null_mut(),
            comment_meta_capacity: 0,
            comments: ptr::null_mut(),
            comments_capacity: 0,
        }
    }

    pub fn ensure_token_capacity(&mut self) {
        if !self.tok_kinds.is_null() {
            return;
        }
        let cap = self.tokens_capacity;
        if cap == 0 {
            return;
        }
        self.tok_kinds = alloc_uninit::<u8>(cap);
        self.tok_kinds_capacity = cap;
        self.tok_starts = alloc_uninit::<u32>(cap + 1);
        self.tok_starts_capacity = cap + 1;

        self.numbers = alloc_uninit::<f64>((cap / 2) + 64);
        self.numbers_capacity = (cap / 2) + 64;
        self.atoms = alloc_uninit::<crate::token::StringSpan>((cap / 4) + 64);
        self.atoms_capacity = (cap / 4) + 64;
        self.strings = alloc_uninit::<crate::token::StringSpan>((cap / 2) + 64);
        self.strings_capacity = (cap / 2) + 64;
        self.cooked_bytes = alloc_uninit::<u8>(cap);
        self.cooked_bytes_capacity = cap;
        self.templates = alloc_uninit::<crate::token::StringSpan>((cap / 2) + 64);
        self.templates_capacity = (cap / 2) + 64;
        self.regex_flags = alloc_uninit::<oxc_ast::ast::RegExpFlags>((cap / 2) + 64);
        self.regex_flags_capacity = (cap / 2) + 64;
        self.comment_meta = alloc_uninit::<u8>((cap / 2) + 64);
        self.comment_meta_capacity = (cap / 2) + 64;
        self.comments = alloc_uninit::<oxc_ast::ast::Comment>((cap / 3) + 64);
        self.comments_capacity = (cap / 3) + 64;
    }
}

#[inline]
fn alloc_uninit<T>(cap: u32) -> *mut T {
    if cap == 0 {
        return ptr::null_mut();
    }
    let mut v = Vec::<T>::with_capacity(cap as usize);
    let p = v.as_mut_ptr();
    core::mem::forget(v);
    p
}

#[inline]
unsafe fn free_uninit<T>(ptr: *mut T, cap: u32) {
    if ptr.is_null() {
        return;
    }
    // SAFETY: `ptr`/`cap` came from `alloc_uninit`'s leaked Vec; len 0 means only the allocation is released.
    unsafe {
        drop(Vec::from_raw_parts(ptr, 0, cap as usize));
    }
}

impl Drop for Arena {
    fn drop(&mut self) {
        // SAFETY: every pointer is null or a live `alloc_uninit` allocation with the recorded capacity, freed once.
        unsafe {
            free_uninit::<Diagnostic>(self.diags, self.diags_capacity);
            free_uninit::<LineEntry>(self.lines, self.lines_capacity);
            free_uninit::<u8>(self.tok_kinds, self.tok_kinds_capacity);
            free_uninit::<u32>(self.tok_starts, self.tok_starts_capacity);
            free_uninit::<f64>(self.numbers, self.numbers_capacity);
            free_uninit::<crate::token::StringSpan>(self.atoms, self.atoms_capacity);
            free_uninit::<crate::token::StringSpan>(self.strings, self.strings_capacity);
            free_uninit::<u8>(self.cooked_bytes, self.cooked_bytes_capacity);
            free_uninit::<crate::token::StringSpan>(self.templates, self.templates_capacity);
            free_uninit::<oxc_ast::ast::RegExpFlags>(self.regex_flags, self.regex_flags_capacity);
            free_uninit::<u8>(self.comment_meta, self.comment_meta_capacity);
            free_uninit::<oxc_ast::ast::Comment>(self.comments, self.comments_capacity);
        }
        self.diags = ptr::null_mut();
        self.lines = ptr::null_mut();
        self.tok_kinds = ptr::null_mut();
        self.tok_starts = ptr::null_mut();
        self.numbers = ptr::null_mut();
        self.atoms = ptr::null_mut();
        self.strings = ptr::null_mut();
        self.cooked_bytes = ptr::null_mut();
        self.templates = ptr::null_mut();
        self.regex_flags = ptr::null_mut();
        self.comment_meta = ptr::null_mut();
        self.comments = ptr::null_mut();
    }
}
