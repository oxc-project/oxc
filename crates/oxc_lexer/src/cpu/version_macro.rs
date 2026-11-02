/// Macro to mark a function as the version of some code for one CPU feature set.
///
/// [`cpu_select!`] picks between the versions.
///
/// Applied to a function with one of:
/// - `#[apply(cpu_version(Avx2))]`
/// - `#[apply(cpu_version(Neon))]`
/// - `#[apply(cpu_version(Generic))]`
///
/// ```ignore
/// use crate::cpu::{Avx2, apply, cpu_version};
///
/// #[apply(cpu_version(Avx2))]
/// #[inline]
/// unsafe fn bm_any_avx2(bm: *const u64, nw: usize, _cpu: Avx2) -> bool {
///     // ...
/// }
/// ```
///
/// * `Avx2`: The last parameter must be `cpu: Avx2` (any name instead of `cpu` is fine).
///   The function gets `#[cfg(target_arch = "x86_64")]`,
///   and `#[target_feature(enable = "avx2,bmi2,popcnt")]` unless it's `#[inline(always)]`,
///   which can't be combined with `#[target_feature]`.
///   An `#[inline(always)]` function is compiled as part of its callers, with their features.
/// * `Neon`: The same, with `cpu: Neon`, `#[cfg(target_arch = "aarch64")]` and
///   `#[target_feature(enable = "neon")]`.
/// * `Generic`: No parameter can have a CPU type
///   (`Avx2`, `Neon`, `Generic`, `DynCpu` or `impl Cpu`).
///   A generic version only runs where the CPU feature set is [`BASELINE`]'s,
///   so it uses [`BASELINE`] where it needs a CPU type. The function gets no attributes.
///
/// Otherwise the function is output unchanged, from its original tokens.
/// So it compiles, and is linted, exactly as written.
///
/// ## Implementation
///
/// The macro works in stages, like [`cpu_versions!`]. The stages move the function's tokens
/// into `item` one at a time:
///
/// * `attrs!`: Attributes. Records whether one is `#[inline(always)]`.
/// * `head!`: Visibility, `const` and `unsafe`, then `fn` and the function's name.
/// * `generics!`: Generic parameters, then the parameters' `(...)`.
///   `angles` holds a `<` for each `<` not yet closed, so a `(...)` in the generics
///   (e.g. `F: Fn(u8)`) isn't taken as the parameters.
/// * `tail!`: Return type and body, up to the `(Avx2)` which `#[apply]` adds after the function.
/// * `check!`: Checks the parameters, then outputs the function with its new attributes.
///
/// Literal tokens (e.g. `fn`) are matched in `rest_to_match`, and captured from `rest_to_capture`,
/// in the same way as `cpu_versions!`'s `attrs!`. So `item` holds the function's original tokens.
///
/// [`cpu_select!`]: super::cpu_select!
/// [`cpu_versions!`]: super::cpu_versions!
/// [`BASELINE`]: super::BASELINE
macro_rules! cpu_version {
    ($($input:tt)*) => {
        crate::cpu::__cpu_version::attrs!(
            rest_to_match = [ $($input)* ]
            rest_to_capture = [ $($input)* ]
            inline_always = false
            item = []
        );
    };
}

pub(crate) use cpu_version;

/// Moves the attributes into `item`, and records whether one is `#[inline(always)]`.
#[doc(hidden)]
macro_rules! attrs {
    // `#[inline(always)]`
    (
        rest_to_match = [ #[inline(always)] $($_rest:tt)* ]
        rest_to_capture = [ $hash:tt $attr:tt $($rest:tt)* ]
        inline_always = $_inline_always:tt
        item = [ $($item:tt)* ]
    ) => {
        crate::cpu::__cpu_version::attrs!(
            rest_to_match = [ $($rest)* ]
            rest_to_capture = [ $($rest)* ]
            inline_always = true
            item = [ $($item)* $hash $attr ]
        );
    };

    // Any other attribute
    (
        rest_to_match = [ # $_attr:tt $($_rest:tt)* ]
        rest_to_capture = [ $hash:tt $attr:tt $($rest:tt)* ]
        inline_always = $inline_always:tt
        item = [ $($item:tt)* ]
    ) => {
        crate::cpu::__cpu_version::attrs!(
            rest_to_match = [ $($rest)* ]
            rest_to_capture = [ $($rest)* ]
            inline_always = $inline_always
            item = [ $($item)* $hash $attr ]
        );
    };

    // No attributes left
    (
        rest_to_match = $_rest_to_match:tt
        rest_to_capture = $rest:tt
        inline_always = $inline_always:tt
        item = $item:tt
    ) => {
        crate::cpu::__cpu_version::head!(
            rest_to_match = $rest
            rest_to_capture = $rest
            inline_always = $inline_always
            item = $item
        );
    };
}

pub(crate) use attrs;

/// Moves the tokens before `fn` into `item`, then `fn` and the function's name.
#[doc(hidden)]
macro_rules! head {
    // `fn`, then the function's name
    (
        rest_to_match = [ fn $($_rest:tt)* ]
        rest_to_capture = [ $fn_kw:tt $fn_name:tt $($rest:tt)* ]
        inline_always = $inline_always:tt
        item = [ $($item:tt)* ]
    ) => {
        crate::cpu::__cpu_version::generics!(
            rest_to_match = [ $($rest)* ]
            rest_to_capture = [ $($rest)* ]
            angles = []
            inline_always = $inline_always
            item = [ $($item)* $fn_kw $fn_name ]
        );
    };

    // Visibility, `const` or `unsafe`
    (
        rest_to_match = $_rest_to_match:tt
        rest_to_capture = [ $token:tt $($rest:tt)* ]
        inline_always = $inline_always:tt
        item = [ $($item:tt)* ]
    ) => {
        crate::cpu::__cpu_version::head!(
            rest_to_match = [ $($rest)* ]
            rest_to_capture = [ $($rest)* ]
            inline_always = $inline_always
            item = [ $($item)* $token ]
        );
    };

    // No `fn`
    ($($_any:tt)*) => {
        compile_error!("`cpu_version!` must be applied to a `fn`");
    };
}

pub(crate) use head;

/// Moves the generic parameters into `item`, then the parameters' `(...)`.
#[doc(hidden)]
macro_rules! generics {
    // The parameters' `(...)`, outside the generics
    (
        rest_to_match = [ ( $($params:tt)* ) $($_rest:tt)* ]
        rest_to_capture = [ $params_group:tt $($rest:tt)* ]
        angles = []
        inline_always = $inline_always:tt
        item = [ $($item:tt)* ]
    ) => {
        crate::cpu::__cpu_version::tail!(
            rest = [ $($rest)* ]
            params = [ $($params)* ]
            inline_always = $inline_always
            item = [ $($item)* $params_group ]
        );
    };

    // `<`
    (
        rest_to_match = [ < $($_rest:tt)* ]
        rest_to_capture = [ $token:tt $($rest:tt)* ]
        angles = [ $($angles:tt)* ]
        inline_always = $inline_always:tt
        item = [ $($item:tt)* ]
    ) => {
        crate::cpu::__cpu_version::generics!(
            rest_to_match = [ $($rest)* ]
            rest_to_capture = [ $($rest)* ]
            angles = [ < $($angles)* ]
            inline_always = $inline_always
            item = [ $($item)* $token ]
        );
    };

    // `>`
    (
        rest_to_match = [ > $($_rest:tt)* ]
        rest_to_capture = [ $token:tt $($rest:tt)* ]
        angles = [ < $($angles:tt)* ]
        inline_always = $inline_always:tt
        item = [ $($item:tt)* ]
    ) => {
        crate::cpu::__cpu_version::generics!(
            rest_to_match = [ $($rest)* ]
            rest_to_capture = [ $($rest)* ]
            angles = [ $($angles)* ]
            inline_always = $inline_always
            item = [ $($item)* $token ]
        );
    };

    // `>>` is a single token, which closes 2 `<`s
    (
        rest_to_match = [ >> $($_rest:tt)* ]
        rest_to_capture = [ $token:tt $($rest:tt)* ]
        angles = [ < < $($angles:tt)* ]
        inline_always = $inline_always:tt
        item = [ $($item:tt)* ]
    ) => {
        crate::cpu::__cpu_version::generics!(
            rest_to_match = [ $($rest)* ]
            rest_to_capture = [ $($rest)* ]
            angles = [ $($angles)* ]
            inline_always = $inline_always
            item = [ $($item)* $token ]
        );
    };

    // Any other token
    (
        rest_to_match = $_rest_to_match:tt
        rest_to_capture = [ $token:tt $($rest:tt)* ]
        angles = $angles:tt
        inline_always = $inline_always:tt
        item = [ $($item:tt)* ]
    ) => {
        crate::cpu::__cpu_version::generics!(
            rest_to_match = [ $($rest)* ]
            rest_to_capture = [ $($rest)* ]
            angles = $angles
            inline_always = $inline_always
            item = [ $($item)* $token ]
        );
    };
}

pub(crate) use generics;

/// Moves the rest of the function into `item`, until only the `(Avx2)` after it is left.
#[doc(hidden)]
macro_rules! tail {
    // Only the `(Avx2)`, `(Neon)` or `(Generic)` is left
    (
        rest = [ ( $kind:ident ) ]
        params = $params:tt
        inline_always = $inline_always:tt
        item = $item:tt
    ) => {
        crate::cpu::__cpu_version::check!(
            kind = $kind
            params = $params
            inline_always = $inline_always
            item = $item
        );
    };

    // Return type, `where` clause or body
    (
        rest = [ $token:tt $($rest:tt)+ ]
        params = $params:tt
        inline_always = $inline_always:tt
        item = [ $($item:tt)* ]
    ) => {
        crate::cpu::__cpu_version::tail!(
            rest = [ $($rest)+ ]
            params = $params
            inline_always = $inline_always
            item = [ $($item)* $token ]
        );
    };

    // No `(Avx2)`, `(Neon)` or `(Generic)`
    ($($_any:tt)*) => {
        compile_error!(
            "`cpu_version!` must be applied as `#[apply(cpu_version(Avx2))]`, `Neon` or `Generic`"
        );
    };
}

pub(crate) use tail;

/// Checks the parameters, then outputs the function.
///
/// The parameters' tokens are dropped from `params` one at a time.
/// * `Avx2` and `Neon`: Until only the last parameter is left.
///   It must be `cpu: Avx2` or `cpu: Neon`.
/// * `Generic`: Until none are left. No parameter can be `cpu: Avx2` etc along the way.
#[doc(hidden)]
macro_rules! check {
    // `Avx2` version, with last parameter `cpu: Avx2`
    (
        kind = Avx2
        params = [ $_cpu_name:ident: Avx2 $(,)? ]
        inline_always = $inline_always:tt
        item = [ $($item:tt)* ]
    ) => {
        crate::cpu::__cpu_version::check!(
            output
            cfg = [ target_arch = "x86_64" ]
            features = "avx2,bmi2,popcnt"
            inline_always = $inline_always
            item = [ $($item)* ]
        );
    };

    // `Neon` version, with last parameter `cpu: Neon`
    (
        kind = Neon
        params = [ $_cpu_name:ident: Neon $(,)? ]
        inline_always = $inline_always:tt
        item = [ $($item:tt)* ]
    ) => {
        crate::cpu::__cpu_version::check!(
            output
            cfg = [ target_arch = "aarch64" ]
            features = "neon"
            inline_always = $inline_always
            item = [ $($item)* ]
        );
    };

    // Output an `#[inline(always)]` `Avx2` or `Neon` version
    (
        output
        cfg = [ $($cfg:tt)* ]
        features = $_features:tt
        inline_always = true
        item = [ $($item:tt)* ]
    ) => {
        #[cfg($($cfg)*)]
        $($item)*
    };

    // Output any other `Avx2` or `Neon` version
    (
        output
        cfg = [ $($cfg:tt)* ]
        features = $features:tt
        inline_always = false
        item = [ $($item:tt)* ]
    ) => {
        #[cfg($($cfg)*)]
        #[target_feature(enable = $features)]
        $($item)*
    };

    // `Generic` version, with a parameter which has a CPU type
    (kind = Generic params = [ $_name:ident: Avx2 $(, $($_rest:tt)*)? ] $($_slots:tt)*) => {
        crate::cpu::__cpu_version::check!(generic_with_cpu);
    };
    (kind = Generic params = [ $_name:ident: Neon $(, $($_rest:tt)*)? ] $($_slots:tt)*) => {
        crate::cpu::__cpu_version::check!(generic_with_cpu);
    };
    (kind = Generic params = [ $_name:ident: Generic $(, $($_rest:tt)*)? ] $($_slots:tt)*) => {
        crate::cpu::__cpu_version::check!(generic_with_cpu);
    };
    (kind = Generic params = [ $_name:ident: DynCpu $(, $($_rest:tt)*)? ] $($_slots:tt)*) => {
        crate::cpu::__cpu_version::check!(generic_with_cpu);
    };
    (kind = Generic params = [ $_name:ident: impl Cpu $(, $($_rest:tt)*)? ] $($_slots:tt)*) => {
        crate::cpu::__cpu_version::check!(generic_with_cpu);
    };
    (generic_with_cpu) => {
        compile_error!("`cpu_version(Generic)` functions can't have a parameter with a CPU type");
    };

    // `Generic` version, all parameters checked
    (
        kind = Generic
        params = []
        inline_always = $_inline_always:tt
        item = [ $($item:tt)* ]
    ) => {
        $($item)*
    };

    // Drop a token from `params`
    (
        kind = $kind:ident
        params = [ $_token:tt $($params:tt)* ]
        inline_always = $inline_always:tt
        item = $item:tt
    ) => {
        crate::cpu::__cpu_version::check!(
            kind = $kind
            params = [ $($params)* ]
            inline_always = $inline_always
            item = $item
        );
    };

    // `Avx2` or `Neon` version, with last parameter not `cpu: Avx2` / `cpu: Neon`
    (kind = Avx2 params = [] $($_slots:tt)*) => {
        compile_error!("`cpu_version(Avx2)` functions must have `cpu: Avx2` as last parameter");
    };
    (kind = Neon params = [] $($_slots:tt)*) => {
        compile_error!("`cpu_version(Neon)` functions must have `cpu: Neon` as last parameter");
    };

    // Not `Avx2`, `Neon` or `Generic`
    ($($_any:tt)*) => {
        compile_error!("`cpu_version!` takes `Avx2`, `Neon` or `Generic`");
    };
}

pub(crate) use check;

#[doc(hidden)]
pub mod __cpu_version {
    pub(crate) use super::{attrs, check, generics, head, tail};
}
