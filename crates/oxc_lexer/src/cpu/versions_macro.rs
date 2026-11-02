/// Macro to turn a function into one with a separate entry point for each CPU feature set.
///
/// Used for pipeline stages, and for functions they call which aren't always inlined.
/// Each entry point is compiled with its feature set's target features enabled,
/// so SIMD functions which the function calls can be inlined into it.
///
/// Applied to a function with `#[apply(cpu_versions)]`:
///
/// ```ignore
/// use crate::cpu::{Cpu, apply, cpu_versions};
///
/// /// Compress tokens.
/// #[apply(cpu_versions)]
/// #[inline(never)]
/// pub unsafe fn compress(src: &[u8], n: usize, cpu: impl Cpu) -> usize {
///     // ...
/// }
/// ```
///
/// `compress` becomes a function which calls one of 3 entry points, depending on the type of `cpu`.
/// It's `#[inline(always)]`, so when called with a specific type (e.g. `Avx2`),
/// it compiles to a direct call to that entry point.
///
/// The entry points are functions defined inside `compress`:
///
/// - `__avx2`: Takes `cpu: Avx2`. Compiled with AVX2 enabled. Only exists on x86_64.
/// - `__neon`: Takes `cpu: Neon`. Compiled with Neon enabled. Only exists on aarch64.
/// - `__generic`: Takes `cpu: Generic`.
///
/// Each entry point calls `__impl`, an `#[inline(always)]` function containing the original function body.
/// So the body is compiled as part of each entry point, and the AVX2 functions it calls can be inlined
/// into the AVX2 entry point. They can't be inlined into a function which doesn't have AVX2 enabled.
///
/// A function defined inside `compress` shadows a parameter with the same name in its body.
/// The names start with `__`, so they don't clash with any real parameter names.
///
/// Methods are supported too, with a `self`, `&self`, `&mut self`, `mut self` or `self: Type` parameter.
/// Only methods can take `self`, so a method's entry points are methods next to it in the same `impl`,
/// rather than functions inside it. Their names are made with `pastey`, e.g. for a method `pipeline`:
/// `__pipeline_impl`, `__pipeline_generic`, `__pipeline_avx2` and `__pipeline_neon`.
/// They're private, so they're only visible in the module which defines the method.
///
/// They aren't methods of a trait defined inside the method, because rustc gives trait methods external
/// linkage. That stops LLVM from inlining helpers into them, and from removing the generic entry point
/// on aarch64 with Neon, where it can never be called.
///
/// Code for other platforms is removed with `cfg_select!`, not `#[cfg]` attributes.
/// That code contains copies of the function's parameters.
/// Rust Analyzer greys out the original code of anything in a node disabled by `#[cfg]`,
/// so `#[cfg]` (or [`cpu_select!`], which uses it) would grey out the function's signature.
/// Branches of `cfg_select!` are just dropped, so nothing is greyed out.
///
/// `#[inline]` and `#[inline(never)]` attributes go on the entry points.
/// All other attributes (e.g. doc comments) go on `compress`.
///
/// `#[inline(never)]` keeps the entry points as separate functions.
/// Without it, they can be inlined into their callers in builds which enable AVX2 globally.
///
/// An `#[inline(always)]` function is output unchanged, after the same checks as any other.
/// It's compiled for each CPU feature set as part of its callers' entry points, so it needs
/// none of its own. The macro only marks it as code which is compiled for each feature set.
/// It couldn't have entry points anyway, because `#[inline(always)]` can't be combined with
/// `#[target_feature]`.
///
/// When SSE2 is enabled for the whole crate, `__generic` also gets `#[target_feature(enable = "sse2")]`.
/// That's the case on nearly all x86_64 and x86 targets. It doesn't change the code `__generic` compiles to,
/// because SSE2 is enabled anyway. It stops rustc's MIR inliner from inlining `__generic` into `compress`.
///
/// Without it, an `#[inline]` function would act as if it was `#[inline(always)]` in its generic version.
/// `__generic` only contains a call to `__impl`, so it's small enough for the MIR inliner to inline it.
/// `compress` and `__impl` are both `#[inline(always)]`, so the whole body would then be inlined
/// into every caller, which bloats the generic version of the callers.
///
/// The MIR inliner doesn't inline a function with target features which its caller doesn't have,
/// and it ignores features which are enabled for the whole crate. `__avx2` and `__neon` are protected
/// by their own target features in the same way.
///
/// Only `#[inline]` functions need this. The MIR inliner never inlines `#[inline(never)]` functions,
/// and only inlines functions without an inline attribute if they don't call any other function.
///
/// The only x86_64 targets which don't enable SSE2 are these 2, and Oxc can't or won't run on either:
///
/// - `x86_64-unknown-none`: Bare metal, e.g. operating system kernels. It has no `std`, which Oxc requires.
/// - `x86_64-unknown-uefi`: UEFI firmware programs, which run before an operating system starts.
///   `std` only partly supports it.
///
/// SSE2 can also be disabled with `-C target-feature=-sse2`, but rustc warns that the x86_64 ABI needs it,
/// and that this will become a hard error.
///
/// Most 32-bit x86 targets enable SSE2 too. The exceptions are the `i586` targets, for CPUs from the 1990s,
/// and `i686-unknown-uefi`. 32-bit x86 has no AVX2 version, so there's only one version of the code there.
///
/// Where SSE2 isn't enabled, `__generic` doesn't get the attribute. The code is still correct, only bigger.
///
/// The last parameter must be `cpu: impl Cpu` (any name instead of `cpu` is fine).
/// The other parameters must be `name: Type` or `mut name: Type` (no other patterns),
/// and can't be called `__impl`, `__avx2`, etc.
/// `mut` only goes on `__impl`'s parameters. The other functions just pass the values on.
///
/// The function can be a `const fn`, `unsafe fn` or `const unsafe fn`.
///
/// The function can have generic parameters of any kind, with bounds
/// (e.g. `<'a, 'b: 'a, T: Copy + 'a, const IS_TS: bool>`), but not a `where` clause.
///
/// ## Expansion
///
/// The example above expands to:
///
/// ```ignore
/// /// Compress tokens.
/// #[inline(always)]
/// pub unsafe fn compress(src: &[u8], n: usize, cpu: impl Cpu) -> usize {
///     #[inline(always)]
///     unsafe fn __impl(src: &[u8], n: usize, cpu: impl Cpu) -> usize {
///         // ...
///     }
///
///     #[inline(never)]
///     // Stops the MIR inliner inlining `__generic` into this function. See the docs above.
///     #[cfg_attr(target_feature = "sse2", target_feature(enable = "sse2"))]
///     unsafe fn __generic(src: &[u8], n: usize, cpu: Generic) -> usize {
///         __impl(src, n, cpu)
///     }
///
///     cfg_select! {
///         target_arch = "x86_64" => {
///             #[inline(never)]
///             #[target_feature(enable = "avx2,bmi2,popcnt")]
///             unsafe fn __avx2(src: &[u8], n: usize, cpu: Avx2) -> usize {
///                 __impl(src, n, cpu)
///             }
///
///             match to_dyn(cpu) {
///                 // SAFETY: `Avx2` type guarantees the CPU supports the features `__avx2` uses
///                 DynCpu::Avx2(cpu) => unsafe { __avx2(src, n, cpu) },
///                 // SAFETY: `__generic` only has `#[target_feature(enable = "sse2")]`
///                 // when SSE2 is enabled for the whole crate, so the CPU supports it
///                 DynCpu::Generic(cpu) => unsafe { __generic(src, n, cpu) },
///             }
///         }
///         target_arch = "aarch64" => {
///             #[inline(never)]
///             #[target_feature(enable = "neon")]
///             unsafe fn __neon(src: &[u8], n: usize, cpu: Neon) -> usize {
///                 __impl(src, n, cpu)
///             }
///
///             match to_dyn(cpu) {
///                 // SAFETY: `Neon` type guarantees the CPU supports the features `__neon` uses
///                 DynCpu::Neon(cpu) => unsafe { __neon(src, n, cpu) },
///                 DynCpu::Generic(cpu) => unsafe { __generic(src, n, cpu) },
///             }
///         }
///         _ => {
///             match to_dyn(cpu) {
///                 // SAFETY: `__generic` only has `#[target_feature(enable = "sse2")]`
///                 // when SSE2 is enabled for the whole crate, so the CPU supports it
///                 DynCpu::Generic(cpu) => unsafe { __generic(src, n, cpu) },
///             }
///         }
///     }
/// }
/// ```
///
/// ## Implementation
///
/// The macro works in stages. Each stage is a separate macro, which calls the next one:
///
/// * `attrs!`: Sorts the attributes, and records the function's qualifiers.
/// * `generics!`: Parses the generic parameters, with `generic_param_end!`.
/// * `signature!`: Parses the rest of the function.
/// * `receiver!`: Checks for a `self` parameter.
/// * `params!`: Turns the parameters into records.
/// * `expand!`: Passes everything on to the macro which generates the output:
///   `expand_func!` for a function, or `expand_method!` for a method.
///
/// Each stage's input is a set of named slots. Each slot's value is a single token tree: either a single
/// token or fragment (e.g. `fn_name = foo`), or a list in `[ ... ]` (e.g. `ret_ty = [ usize ]`).
/// Slots which a stage passes on without looking at them are matched as `$name:tt`,
/// or all together as `$($rest:tt)*` if they come last.
///
/// Slots which tell a stage's rules apart come first, and slots which it only passes on come last.
/// So when a rule doesn't match, the compiler finds out early, without matching all the other slots first.
///
/// `fn_body` is matched as `tt` even where it's used. `signature!` has already checked that it's
/// a block, and matching it as `block` again would parse the whole body again.
///
/// A stage which needs more slots while it works adds them in its first rule.
/// Its other rules call the stage again until it's done, and then call the next stage.
///
/// [`cpu_select!`]: super::cpu_select!
macro_rules! cpu_versions {
    ($($item:tt)*) => {
        crate::cpu::__cpu_versions::attrs!(item = [ $($item)* ]);
    };
}

pub(crate) use cpu_versions;

/// Moves the attributes into 2 lists one at a time, then records the function's qualifiers.
/// `#[inline]` attributes go in `inline_attrs`, which go on the entry points.
/// All other attributes go in `outer_attrs`, which go on the outer function.
///
/// The function's tokens are kept twice, as `rest_to_match` and `rest_to_capture`.
/// `rest_to_match` is matched against patterns (e.g. `#[inline ...]`), and `rest_to_capture`
/// is captured as `tt`s.
///
/// Patterns can't capture the tokens they match literally, so this is how `#` and `[...]` are captured.
/// Captured `tt`s keep their original spans, so a compiler error about an attribute points at
/// the attribute where the macro is used, not at this macro. e.g. `#[inline(x)]`, which isn't valid.
///
/// The whole function is also kept unchanged in `item`, which `expand!` outputs for
/// an `#[inline(always)]` function.
///
/// When no attributes are left, a rule for each of `fn`, `unsafe fn`, `const fn` and `const unsafe fn`
/// records the keywords, and passes the rest of the function on to `generics!`, in `signature`.
#[doc(hidden)]
macro_rules! attrs {
    // Entry point
    (item = [ $($item:tt)* ]) => {
        crate::cpu::__cpu_versions::attrs!(
            rest_to_match = [ $($item)* ]
            rest_to_capture = [ $($item)* ]
            outer_attrs = []
            inline_attrs = []
            item = [ $($item)* ]
        );
    };

    // `#[inline]` or `#[inline(...)]` attribute - goes on the entry points
    (
        rest_to_match = [
            #[inline $($_attr:tt)?]
            $($_rest:tt)*
        ]
        rest_to_capture = [
            $hash:tt $attr:tt
            $($rest:tt)*
        ]
        outer_attrs = $outer_attrs:tt
        inline_attrs = [ $($inline_attrs:tt)* ]
        item = $item:tt
    ) => {
        crate::cpu::__cpu_versions::attrs!(
            rest_to_match = [ $($rest)* ]
            rest_to_capture = [ $($rest)* ]
            outer_attrs = $outer_attrs
            inline_attrs = [
                $($inline_attrs)*
                $hash $attr
            ]
            item = $item
        );
    };

    // Any other attribute - goes on the outer function
    (
        rest_to_match = [
            # $_attr:tt
            $($_rest:tt)*
        ]
        rest_to_capture = [
            $hash:tt $attr:tt
            $($rest:tt)*
        ]
        outer_attrs = [ $($outer_attrs:tt)* ]
        inline_attrs = $inline_attrs:tt
        item = $item:tt
    ) => {
        crate::cpu::__cpu_versions::attrs!(
            rest_to_match = [ $($rest)* ]
            rest_to_capture = [ $($rest)* ]
            outer_attrs = [
                $($outer_attrs)*
                $hash $attr
            ]
            inline_attrs = $inline_attrs
            item = $item
        );
    };

    // No attributes left, `fn`
    (
        rest_to_match = $_rest_to_match:tt
        rest_to_capture = [ $vis:vis fn $($rest:tt)* ]
        outer_attrs = $outer_attrs:tt
        inline_attrs = $inline_attrs:tt
        item = $item:tt
    ) => {
        crate::cpu::__cpu_versions::generics!(
            signature = [ $($rest)* ]
            vis = $vis
            fn_qualifiers = []
            outer_attrs = $outer_attrs
            inline_attrs = $inline_attrs
            item = $item
        );
    };

    // No attributes left, `unsafe fn`
    (
        rest_to_match = $_rest_to_match:tt
        rest_to_capture = [ $vis:vis unsafe fn $($rest:tt)* ]
        outer_attrs = $outer_attrs:tt
        inline_attrs = $inline_attrs:tt
        item = $item:tt
    ) => {
        crate::cpu::__cpu_versions::generics!(
            signature = [ $($rest)* ]
            vis = $vis
            fn_qualifiers = [ unsafe ]
            outer_attrs = $outer_attrs
            inline_attrs = $inline_attrs
            item = $item
        );
    };

    // No attributes left, `const fn`
    (
        rest_to_match = $_rest_to_match:tt
        rest_to_capture = [ $vis:vis const fn $($rest:tt)* ]
        outer_attrs = $outer_attrs:tt
        inline_attrs = $inline_attrs:tt
        item = $item:tt
    ) => {
        crate::cpu::__cpu_versions::generics!(
            signature = [ $($rest)* ]
            vis = $vis
            fn_qualifiers = [ const ]
            outer_attrs = $outer_attrs
            inline_attrs = $inline_attrs
            item = $item
        );
    };

    // No attributes left, `const unsafe fn`
    (
        rest_to_match = $_rest_to_match:tt
        rest_to_capture = [ $vis:vis const unsafe fn $($rest:tt)* ]
        outer_attrs = $outer_attrs:tt
        inline_attrs = $inline_attrs:tt
        item = $item:tt
    ) => {
        crate::cpu::__cpu_versions::generics!(
            signature = [ $($rest)* ]
            vis = $vis
            fn_qualifiers = [ const unsafe ]
            outer_attrs = $outer_attrs
            inline_attrs = $inline_attrs
            item = $item
        );
    };

    // Not a function
    ($($_any:tt)*) => {
        compile_error!("`cpu_versions!` must be applied to a `fn`");
    };
}

pub(crate) use attrs;

/// Parses the generic parameters, then passes them on to `signature!`.
///
/// A parameter's bounds can't be matched with a fragment. A `ty` can't be output as a bound,
/// and a `path` can't be followed by `+`. So generic parameters are parsed one token at a time.
///
/// This macro handles the start of each parameter, and `generic_param_end!` handles the rest of it.
/// Between them they move every token into `generic_params`, including the outer `<` and `>`,
/// so it ends up as an exact copy of the function's generic parameters.
///
/// The tokens are kept twice, as `rest_to_match` and `rest_to_capture`, using the same trick as
/// `attrs!`. So a compiler error about the generic parameters points at them, not at this macro.
///
/// The name of each type and const parameter also goes in `generic_args`.
/// Lifetime parameters don't, because calls don't need to pass them.
#[doc(hidden)]
macro_rules! generics {
    // Entry point
    (
        signature = [ $fn_name:ident $($rest:tt)* ]
        $($func:tt)*
    ) => {
        crate::cpu::__cpu_versions::generics!(
            rest_to_match = [ $($rest)* ]
            rest_to_capture = [ $($rest)* ]
            generic_params = []
            generic_args = []
            fn_name = $fn_name
            $($func)*
        );
    };

    // Opening `<`
    (
        rest_to_match = [ < $($_rest:tt)* ]
        rest_to_capture = [ $token:tt $($rest:tt)* ]
        generic_params = []
        $($slots:tt)*
    ) => {
        crate::cpu::__cpu_versions::generics!(
            rest_to_match = [ $($rest)* ]
            rest_to_capture = [ $($rest)* ]
            generic_params = [ $token ]
            $($slots)*
        );
    };

    // No generic parameters
    (
        rest_to_match = $_rest_to_match:tt
        rest_to_capture = $signature:tt
        generic_params = []
        $($slots:tt)*
    ) => {
        crate::cpu::__cpu_versions::signature!(
            signature = $signature
            generic_params = []
            $($slots)*
        );
    };

    // Closing `>`, after a trailing comma
    (
        rest_to_match = [ > $($_rest:tt)* ]
        rest_to_capture = [ $token:tt $($rest:tt)* ]
        generic_params = [ $($generic_params:tt)* ]
        $($slots:tt)*
    ) => {
        crate::cpu::__cpu_versions::signature!(
            signature = [ $($rest)* ]
            generic_params = [ $($generic_params)* $token ]
            $($slots)*
        );
    };

    // Lifetime parameter
    (
        rest_to_match = $_rest_to_match:tt
        rest_to_capture = [ $lifetime:lifetime $($rest:tt)* ]
        generic_params = [ $($generic_params:tt)* ]
        $($slots:tt)*
    ) => {
        crate::cpu::__cpu_versions::generic_param_end!(
            depth = []
            rest_to_match = [ $($rest)* ]
            rest_to_capture = [ $($rest)* ]
            generic_params = [ $($generic_params)* $lifetime ]
            $($slots)*
        );
    };

    // Const parameter. Must come before the type parameter rule, because `ident` matches `const`.
    (
        rest_to_match = [ const $($_rest:tt)* ]
        rest_to_capture = [ $const_kw:tt $name:ident $($rest:tt)* ]
        generic_params = [ $($generic_params:tt)* ]
        generic_args = [ $($generic_args:tt)* ]
        $($slots:tt)*
    ) => {
        crate::cpu::__cpu_versions::generic_param_end!(
            depth = []
            rest_to_match = [ $($rest)* ]
            rest_to_capture = [ $($rest)* ]
            generic_params = [ $($generic_params)* $const_kw $name ]
            generic_args = [ $($generic_args)* $name ]
            $($slots)*
        );
    };

    // Type parameter
    (
        rest_to_match = $_rest_to_match:tt
        rest_to_capture = [ $name:ident $($rest:tt)* ]
        generic_params = [ $($generic_params:tt)* ]
        generic_args = [ $($generic_args:tt)* ]
        $($slots:tt)*
    ) => {
        crate::cpu::__cpu_versions::generic_param_end!(
            depth = []
            rest_to_match = [ $($rest)* ]
            rest_to_capture = [ $($rest)* ]
            generic_params = [ $($generic_params)* $name ]
            generic_args = [ $($generic_args)* $name ]
            $($slots)*
        );
    };
}

pub(crate) use generics;

/// Moves the rest of a generic parameter into `generic_params`, one token at a time.
/// `generics!` has already handled the start of the parameter.
///
/// What's left is the parameter's bounds, or a const parameter's type.
/// It ends at the first `,` or `>` which isn't inside `<...>`.
/// `depth` has a `*` for each `<` which is still open.
///
/// `<<` and `>>` are single tokens, so they each count as 2 `<`s or `>`s.
/// When only 1 `<` is open, `>>` closes it and also ends the generic parameters (`<T: Into<u8>>`).
///
/// `(...)` is a single token tree, so a `,` in `Fn(u8, u8)` isn't seen.
/// `->` is a single token, so its `>` isn't seen either.
#[doc(hidden)]
macro_rules! generic_param_end {
    // `,` - start of the next parameter
    (
        depth = []
        rest_to_match = [ , $($_rest:tt)* ]
        rest_to_capture = [ $token:tt $($rest:tt)* ]
        generic_params = [ $($generic_params:tt)* ]
        $($slots:tt)*
    ) => {
        crate::cpu::__cpu_versions::generics!(
            rest_to_match = [ $($rest)* ]
            rest_to_capture = [ $($rest)* ]
            generic_params = [ $($generic_params)* $token ]
            $($slots)*
        );
    };

    // `>` - end of the generic parameters
    (
        depth = []
        rest_to_match = [ > $($_rest:tt)* ]
        rest_to_capture = [ $token:tt $($rest:tt)* ]
        generic_params = [ $($generic_params:tt)* ]
        $($slots:tt)*
    ) => {
        crate::cpu::__cpu_versions::signature!(
            signature = [ $($rest)* ]
            generic_params = [ $($generic_params)* $token ]
            $($slots)*
        );
    };

    // `>>` - closes the only open `<`, and ends the generic parameters
    (
        depth = [ * ]
        rest_to_match = [ >> $($_rest:tt)* ]
        rest_to_capture = [ $token:tt $($rest:tt)* ]
        generic_params = [ $($generic_params:tt)* ]
        $($slots:tt)*
    ) => {
        crate::cpu::__cpu_versions::signature!(
            signature = [ $($rest)* ]
            generic_params = [ $($generic_params)* $token ]
            $($slots)*
        );
    };

    // `>>` - closes 2 `<`s
    (
        depth = [ * * $($depth:tt)* ]
        rest_to_match = [ >> $($_rest:tt)* ]
        rest_to_capture = [ $token:tt $($rest:tt)* ]
        generic_params = [ $($generic_params:tt)* ]
        $($slots:tt)*
    ) => {
        crate::cpu::__cpu_versions::generic_param_end!(
            depth = [ $($depth)* ]
            rest_to_match = [ $($rest)* ]
            rest_to_capture = [ $($rest)* ]
            generic_params = [ $($generic_params)* $token ]
            $($slots)*
        );
    };

    // `>` - closes a `<`
    (
        depth = [ * $($depth:tt)* ]
        rest_to_match = [ > $($_rest:tt)* ]
        rest_to_capture = [ $token:tt $($rest:tt)* ]
        generic_params = [ $($generic_params:tt)* ]
        $($slots:tt)*
    ) => {
        crate::cpu::__cpu_versions::generic_param_end!(
            depth = [ $($depth)* ]
            rest_to_match = [ $($rest)* ]
            rest_to_capture = [ $($rest)* ]
            generic_params = [ $($generic_params)* $token ]
            $($slots)*
        );
    };

    // `<`
    (
        depth = [ $($depth:tt)* ]
        rest_to_match = [ < $($_rest:tt)* ]
        rest_to_capture = [ $token:tt $($rest:tt)* ]
        generic_params = [ $($generic_params:tt)* ]
        $($slots:tt)*
    ) => {
        crate::cpu::__cpu_versions::generic_param_end!(
            depth = [ * $($depth)* ]
            rest_to_match = [ $($rest)* ]
            rest_to_capture = [ $($rest)* ]
            generic_params = [ $($generic_params)* $token ]
            $($slots)*
        );
    };

    // `<<`
    (
        depth = [ $($depth:tt)* ]
        rest_to_match = [ << $($_rest:tt)* ]
        rest_to_capture = [ $token:tt $($rest:tt)* ]
        generic_params = [ $($generic_params:tt)* ]
        $($slots:tt)*
    ) => {
        crate::cpu::__cpu_versions::generic_param_end!(
            depth = [ * * $($depth)* ]
            rest_to_match = [ $($rest)* ]
            rest_to_capture = [ $($rest)* ]
            generic_params = [ $($generic_params)* $token ]
            $($slots)*
        );
    };

    // Any other token
    (
        depth = $depth:tt
        rest_to_match = $_rest_to_match:tt
        rest_to_capture = [ $token:tt $($rest:tt)* ]
        generic_params = [ $($generic_params:tt)* ]
        $($slots:tt)*
    ) => {
        crate::cpu::__cpu_versions::generic_param_end!(
            depth = $depth
            rest_to_match = [ $($rest)* ]
            rest_to_capture = [ $($rest)* ]
            generic_params = [ $($generic_params)* $token ]
            $($slots)*
        );
    };
}

pub(crate) use generic_param_end;

/// Parses the rest of the function, after its generic parameters.
/// Then puts the function into `func`, which bundles everything about it except its parameters:
///
/// * `fn_qualifiers` is `[]`, `[ unsafe ]`, `[ const ]` or `[ const unsafe ]`.
///   It's copied onto every function.
/// * Generic parameters are recorded twice:
///   * `generic_params` is the declaration `<'a, T: Copy, const N: Type, ..>`,
///     used on every function.
///   * `turbofish` is the type and const arguments `::<T, N, ..>` for calls.
///     Lifetimes aren't passed.
/// * `generic_params` is `[()]` if there are none.
///   `turbofish` is `[]` if there are no type or const ones.
/// * `generic_params`' contents are wrapped in `(...)`,
///   because `pastey` would treat `[<...>]` as a paste.
///
/// The parameters are passed on unparsed, in `params`.
#[doc(hidden)]
macro_rules! signature {
    (
        signature = [ ($($params:tt)*) $(-> $ret_ty:ty)? $fn_body:block ]
        generic_params = [ $($generic_params:tt)* ]
        generic_args = [ $($($generic_args:tt)+)? ]
        fn_name = $fn_name:ident
        vis = $vis:vis
        fn_qualifiers = $fn_qualifiers:tt
        outer_attrs = $outer_attrs:tt
        inline_attrs = $inline_attrs:tt
        item = $item:tt
    ) => {
        crate::cpu::__cpu_versions::receiver!(
            params = [ $($params)* ]
            func = {
                fn_name = $fn_name
                outer_attrs = $outer_attrs
                inline_attrs = $inline_attrs
                item = $item
                vis = $vis
                fn_qualifiers = $fn_qualifiers
                // Extra `(...)` parens to prevent `pastey` matching `[<...>]`
                generic_params = [( $($generic_params)* )]
                turbofish = [ $(::<$($generic_args),+>)? ]
                ret_ty = [ $($ret_ty)? ]
                fn_body = $fn_body
            }
        );
    };

    // Has a `where` clause, or has no body
    ($($_any:tt)*) => {
        compile_error!(
            "`cpu_versions!` must be applied to a `fn` with a body and no `where` clause"
        );
    };
}

pub(crate) use signature;

/// Checks for a `self` parameter. If there is one, adds `method` to the top of `func`.
/// It's at the top so `expand!` can check for it without matching the rest of `func`.
///
/// A function has no `method`.
/// For a method it's `{ self_mut_kw = [ ... ] self_param = [ ... ] self_kw = self }`:
///
/// * `self_param` is the `self` parameter without a leading `mut` (e.g. `&mut self` or `self: Box<Self>`).
/// * `self_mut_kw` is `[ mut ]` for `mut self` or `mut self: Type`, otherwise `[]`.
///   Like parameters' `mut_kw`, it only goes on `__*_impl`.
///
/// A method's entry points must use the method's own `self` token. A `self` written by this macro
/// wouldn't refer to the same `self` as the body does. So the parameters are kept twice,
/// as `params_to_match` and `params_to_capture`, using the same trick as `attrs!`.
#[doc(hidden)]
macro_rules! receiver {
    // Entry point
    (
        params = [ $($params:tt)* ]
        func = $func:tt
    ) => {
        crate::cpu::__cpu_versions::receiver!(
            params_to_match = [ $($params)* ]
            params_to_capture = [ $($params)* ]
            func = $func
        );
    };

    // `self` or `self: Type` parameter
    (
        params_to_match = [ self $($_rest:tt)* ]
        params_to_capture = [
            $self_kw:ident $(: $self_ty:ty)?,
            $($rest:tt)*
        ]
        func = { $($func:tt)* }
    ) => {
        crate::cpu::__cpu_versions::params!(
            params = [ $($rest)* ]
            func = {
                method = {
                    self_mut_kw = []
                    self_param = [ $self_kw $(: $self_ty)? ]
                    self_kw = $self_kw
                }
                $($func)*
            }
        );
    };

    // `mut self` or `mut self: Type` parameter
    (
        params_to_match = [ mut self $($_rest:tt)* ]
        params_to_capture = [
            mut $self_kw:ident $(: $self_ty:ty)?,
            $($rest:tt)*
        ]
        func = { $($func:tt)* }
    ) => {
        crate::cpu::__cpu_versions::params!(
            params = [ $($rest)* ]
            func = {
                method = {
                    self_mut_kw = [ mut ]
                    self_param = [ $self_kw $(: $self_ty)? ]
                    self_kw = $self_kw
                }
                $($func)*
            }
        );
    };

    // `&self` parameter
    (
        params_to_match = [ &self $($_rest:tt)* ]
        params_to_capture = [
            &$self_kw:ident,
            $($rest:tt)*
        ]
        func = { $($func:tt)* }
    ) => {
        crate::cpu::__cpu_versions::params!(
            params = [ $($rest)* ]
            func = {
                method = {
                    self_mut_kw = []
                    self_param = [ &$self_kw ]
                    self_kw = $self_kw
                }
                $($func)*
            }
        );
    };

    // `&mut self` parameter
    (
        params_to_match = [ &mut self $($_rest:tt)* ]
        params_to_capture = [
            &mut $self_kw:ident,
            $($rest:tt)*
        ]
        func = { $($func:tt)* }
    ) => {
        crate::cpu::__cpu_versions::params!(
            params = [ $($rest)* ]
            func = {
                method = {
                    self_mut_kw = []
                    self_param = [ &mut $self_kw ]
                    self_kw = $self_kw
                }
                $($func)*
            }
        );
    };

    // No `self` parameter
    (
        params_to_match = $_params_to_match:tt
        params_to_capture = [ $($params:tt)* ]
        func = $func:tt
    ) => {
        crate::cpu::__cpu_versions::params!(
            params = [ $($params)* ]
            func = $func
        );
    };
}

pub(crate) use receiver;

/// Turns the parameters into records, and checks that the last one is `cpu: impl Cpu`.
///
/// The last parameter gets a different type in each function, so it has to be matched on its own.
/// `macro_rules!` can't match "all parameters except the last one" in one go.
/// So parameters are moved from `rest_params` to `params` one at a time, until only the last one is left.
/// Each becomes a record `{ mut_kw = [ ... ] arg_name = ... arg_ty = ... }`.
/// `mut_kw` is `[ mut ]` or `[]`, so `mut` can be put on `__impl`'s parameters only.
///
/// `func` is passed on unchanged.
#[doc(hidden)]
macro_rules! params {
    // Entry point
    (
        params = [ $($params:tt)* ]
        func = $func:tt
    ) => {
        crate::cpu::__cpu_versions::params!(
            rest_params = [ $($params)* ]
            params = []
            func = $func
        );
    };

    // Last parameter `cpu: impl Cpu`
    (
        rest_params = [ $cpu_name:ident: impl $cpu_trait:ident $(,)? ]
        params = $params:tt
        func = $func:tt
    ) => {
        crate::cpu::__cpu_versions::expand!(
            func = $func
            params = $params
            cpu_name = $cpu_name
            cpu_trait = $cpu_trait
        );
    };

    // `mut` parameter
    (
        rest_params = [
            mut $arg_name:ident: $arg_ty:ty,
            $($rest_params:tt)*
        ]
        params = [ $($params:tt)* ]
        func = $func:tt
    ) => {
        crate::cpu::__cpu_versions::params!(
            rest_params = [ $($rest_params)* ]
            params = [
                $($params)*
                {
                    mut_kw = [ mut ]
                    arg_name = $arg_name
                    arg_ty = $arg_ty
                }
            ]
            func = $func
        );
    };

    // Any other parameter
    (
        rest_params = [
            $arg_name:ident: $arg_ty:ty,
            $($rest_params:tt)*
        ]
        params = [ $($params:tt)* ]
        func = $func:tt
    ) => {
        crate::cpu::__cpu_versions::params!(
            rest_params = [ $($rest_params)* ]
            params = [
                $($params)*
                {
                    mut_kw = []
                    arg_name = $arg_name
                    arg_ty = $arg_ty
                }
            ]
            func = $func
        );
    };

    // Last parameter isn't `cpu: impl Cpu`, or a parameter isn't `name: Type` or `mut name: Type`
    ($($_any:tt)*) => {
        compile_error!(
            "`cpu_versions!` parameters must be `[mut] name: Type`, and the last must be `cpu: impl Cpu`"
        );
    };
}

pub(crate) use params;

/// Unwraps `func`, and passes its slots and all the other slots on to:
/// * `expand_func!` for a function.
/// * `expand_method!` for a method.
///
/// `func` only bundles the function's slots to carry them through the stages before this one.
///
/// An `#[inline(always)]` function or method is output unchanged, from `item`.
#[doc(hidden)]
macro_rules! expand {
    // `#[inline(always)]` function or method - output unchanged
    (
        func = {
            $(method = $_method:tt)?
            fn_name = $_fn_name:ident
            outer_attrs = $_outer_attrs:tt
            inline_attrs = [ #[inline(always)] ]
            item = [ $($item:tt)* ]
            $($_func:tt)*
        }
        $($_rest:tt)*
    ) => {
        $($item)*
    };

    // Method. Must come before the function rule, which matches any `func`.
    //
    // `method` moves to after the function's slots. So `expand_method!` gets the function's slots
    // in the same order as `expand_func!`, starting with `fn_name`.
    (
        func = {
            method = $method:tt
            $($func:tt)*
        }
        $($rest:tt)*
    ) => {
        crate::cpu::__cpu_versions::expand_method!(
            $($func)*
            method = $method
            $($rest)*
        );
    };

    // Function
    (
        func = { $($func:tt)* }
        $($rest:tt)*
    ) => {
        crate::cpu::__cpu_versions::expand_func!(
            $($func)*
            $($rest)*
        );
    };
}

pub(crate) use expand;

/// Generates the output for a function. Its entry points are functions inside it.
#[doc(hidden)]
macro_rules! expand_func {
    (
        fn_name = $fn_name:ident
        outer_attrs = [ $($outer_attrs:tt)* ]
        inline_attrs = [ $($inline_attrs:tt)* ]
        item = $_item:tt
        vis = $vis:vis
        fn_qualifiers = [ $($fn_qualifiers:tt)* ]
        generic_params = [( $($generic_params:tt)* )]
        turbofish = [ $($turbofish:tt)* ]
        ret_ty = [ $($ret_ty:ty)? ]
        fn_body = $fn_body:tt
        params = [$(
            {
                mut_kw = [ $($mut_kw:tt)? ]
                arg_name = $arg_name:ident
                arg_ty = $arg_ty:ty
            }
        )*]
        cpu_name = $cpu_name:ident
        cpu_trait = $cpu_trait:ident
    ) => {
        $($outer_attrs)*
        #[inline(always)]
        $vis $($fn_qualifiers)* fn $fn_name $($generic_params)* (
            $($arg_name: $arg_ty,)* $cpu_name: impl $cpu_trait
        ) $(-> $ret_ty)? {
            #[inline(always)]
            $($fn_qualifiers)* fn __impl $($generic_params)* (
                $($($mut_kw)? $arg_name: $arg_ty,)* $cpu_name: impl $cpu_trait
            ) $(-> $ret_ty)?
            $fn_body

            $($inline_attrs)*
            // Stops the MIR inliner inlining `__generic` into this function. See the docs above.
            #[cfg_attr(target_feature = "sse2", target_feature(enable = "sse2"))]
            #[allow(unsafe_op_in_unsafe_fn, clippy::allow_attributes)]
            $($fn_qualifiers)* fn __generic $($generic_params)* (
                $($arg_name: $arg_ty,)* $cpu_name: crate::cpu::Generic
            ) $(-> $ret_ty)? {
                __impl $($turbofish)* ($($arg_name,)* $cpu_name)
            }

            cfg_select! {
                target_arch = "x86_64" => {
                    $($inline_attrs)*
                    #[target_feature(enable = "avx2,bmi2,popcnt")]
                    #[allow(unsafe_op_in_unsafe_fn, clippy::allow_attributes)]
                    $($fn_qualifiers)* fn __avx2 $($generic_params)* (
                        $($arg_name: $arg_ty,)* $cpu_name: crate::cpu::Avx2
                    ) $(-> $ret_ty)? {
                        __impl $($turbofish)* ($($arg_name,)* $cpu_name)
                    }

                    match crate::cpu::to_dyn($cpu_name) {
                        // SAFETY: `Avx2` type guarantees the CPU supports the features `__avx2` uses
                        crate::cpu::DynCpu::Avx2($cpu_name) => unsafe {
                            __avx2 $($turbofish)* ($($arg_name,)* $cpu_name)
                        },
                        // SAFETY: `__generic` only has `#[target_feature(enable = "sse2")]`
                        // when SSE2 is enabled for the whole crate, so the CPU supports it
                        #[allow(unused_unsafe, clippy::allow_attributes)]
                        crate::cpu::DynCpu::Generic($cpu_name) => unsafe {
                            __generic $($turbofish)* ($($arg_name,)* $cpu_name)
                        },
                    }
                }
                target_arch = "aarch64" => {
                    $($inline_attrs)*
                    #[target_feature(enable = "neon")]
                    #[allow(unsafe_op_in_unsafe_fn, clippy::allow_attributes)]
                    $($fn_qualifiers)* fn __neon $($generic_params)* (
                        $($arg_name: $arg_ty,)* $cpu_name: crate::cpu::Neon
                    ) $(-> $ret_ty)? {
                        __impl $($turbofish)* ($($arg_name,)* $cpu_name)
                    }

                    match crate::cpu::to_dyn($cpu_name) {
                        // SAFETY: `Neon` type guarantees the CPU supports the features `__neon` uses
                        crate::cpu::DynCpu::Neon($cpu_name) => unsafe {
                            __neon $($turbofish)* ($($arg_name,)* $cpu_name)
                        },
                        #[allow(unused_unsafe, clippy::undocumented_unsafe_blocks, clippy::allow_attributes)]
                        crate::cpu::DynCpu::Generic($cpu_name) => unsafe {
                            __generic $($turbofish)* ($($arg_name,)* $cpu_name)
                        },
                    }
                }
                _ => {
                    match crate::cpu::to_dyn($cpu_name) {
                        // SAFETY: `__generic` only has `#[target_feature(enable = "sse2")]`
                        // when SSE2 is enabled for the whole crate, so the CPU supports it
                        #[allow(unused_unsafe, clippy::allow_attributes)]
                        crate::cpu::DynCpu::Generic($cpu_name) => unsafe {
                            __generic $($turbofish)* ($($arg_name,)* $cpu_name)
                        },
                    }
                }
            }
        }
    };
}

pub(crate) use expand_func;

/// Generates the output for a method. Its entry points are private methods next to it.
///
/// The first rule makes the names of the entry points (`impl_fn = __pipeline_impl` etc),
/// and puts them in `fn_names`, along with `fn_name`.
///
/// The names are made with `pastey`, which has to wrap the macro call which receives them.
/// Doing it in its own step keeps `pastey::paste!` out of the main output.
#[doc(hidden)]
macro_rules! expand_method {
    // Entry point - make the names of the entry points
    (
        fn_name = $fn_name:ident
        $($rest:tt)*
    ) => {
        pastey::paste! {
            crate::cpu::__cpu_versions::expand_method!(
                fn_names = {
                    fn_name = $fn_name
                    impl_fn = [<__ $fn_name _impl>]
                    generic_fn = [<__ $fn_name _generic>]
                    avx2_fn = [<__ $fn_name _avx2>]
                    neon_fn = [<__ $fn_name _neon>]
                }
                $($rest)*
            );
        }
    };

    // Names of the entry points made - generate the output
    (
        fn_names = {
            fn_name = $fn_name:ident
            impl_fn = $impl_fn:ident
            generic_fn = $generic_fn:ident
            avx2_fn = $avx2_fn:ident
            neon_fn = $neon_fn:ident
        }
        outer_attrs = [ $($outer_attrs:tt)* ]
        inline_attrs = [ $($inline_attrs:tt)* ]
        item = $_item:tt
        vis = $vis:vis
        fn_qualifiers = [ $($fn_qualifiers:tt)* ]
        generic_params = [( $($generic_params:tt)* )]
        turbofish = [ $($turbofish:tt)* ]
        ret_ty = [ $($ret_ty:ty)? ]
        fn_body = $fn_body:tt
        method = {
            self_mut_kw = [ $($self_mut_kw:tt)? ]
            self_param = [ $($self_param:tt)* ]
            self_kw = $self_kw:ident
        }
        params = [$(
            {
                mut_kw = [ $($mut_kw:tt)? ]
                arg_name = $arg_name:ident
                arg_ty = $arg_ty:ty
            }
        )*]
        cpu_name = $cpu_name:ident
        cpu_trait = $cpu_trait:ident
    ) => {
        $($outer_attrs)*
        #[inline(always)]
        #[allow(unused_unsafe, clippy::undocumented_unsafe_blocks, clippy::allow_attributes)]
        $vis $($fn_qualifiers)* fn $fn_name $($generic_params)* (
            $($self_param)*, $($arg_name: $arg_ty,)* $cpu_name: impl $cpu_trait
        ) $(-> $ret_ty)? {
            cfg_select! {
                target_arch = "x86_64" => {
                    match crate::cpu::to_dyn($cpu_name) {
                        // SAFETY: `Avx2` type guarantees the CPU supports the features `__*_avx2` uses
                        crate::cpu::DynCpu::Avx2($cpu_name) => unsafe {
                            Self::$avx2_fn $($turbofish)* ($self_kw, $($arg_name,)* $cpu_name)
                        },
                        // SAFETY: `__generic` only has `#[target_feature(enable = "sse2")]`
                        // when SSE2 is enabled for the whole crate, so the CPU supports it
                        crate::cpu::DynCpu::Generic($cpu_name) => unsafe {
                            Self::$generic_fn $($turbofish)* ($self_kw, $($arg_name,)* $cpu_name)
                        },
                    }
                }
                target_arch = "aarch64" => {
                    match crate::cpu::to_dyn($cpu_name) {
                        // SAFETY: `Neon` type guarantees the CPU supports the features `__*_neon` uses
                        crate::cpu::DynCpu::Neon($cpu_name) => unsafe {
                            Self::$neon_fn $($turbofish)* ($self_kw, $($arg_name,)* $cpu_name)
                        },
                        crate::cpu::DynCpu::Generic($cpu_name) => unsafe {
                            Self::$generic_fn $($turbofish)* ($self_kw, $($arg_name,)* $cpu_name)
                        },
                    }
                }
                _ => {
                    match crate::cpu::to_dyn($cpu_name) {
                        // SAFETY: `__generic` only has `#[target_feature(enable = "sse2")]`
                        // when SSE2 is enabled for the whole crate, so the CPU supports it
                        crate::cpu::DynCpu::Generic($cpu_name) => unsafe {
                            Self::$generic_fn $($turbofish)* ($self_kw, $($arg_name,)* $cpu_name)
                        },
                    }
                }
            }
        }

        #[inline(always)]
        $($fn_qualifiers)* fn $impl_fn $($generic_params)* (
            $($self_mut_kw)? $($self_param)*,
            $($($mut_kw)? $arg_name: $arg_ty,)*
            $cpu_name: impl $cpu_trait
        ) $(-> $ret_ty)?
        $fn_body

        $($inline_attrs)*
        // Stops the MIR inliner inlining this method into the outer method. See the docs above.
        #[cfg_attr(target_feature = "sse2", target_feature(enable = "sse2"))]
        #[allow(unsafe_op_in_unsafe_fn, clippy::allow_attributes)]
        $($fn_qualifiers)* fn $generic_fn $($generic_params)* (
            $($self_param)*, $($arg_name: $arg_ty,)* $cpu_name: crate::cpu::Generic
        ) $(-> $ret_ty)? {
            Self::$impl_fn $($turbofish)* ($self_kw, $($arg_name,)* $cpu_name)
        }

        cfg_select! {
            target_arch = "x86_64" => {
                $($inline_attrs)*
                #[target_feature(enable = "avx2,bmi2,popcnt")]
                #[allow(unsafe_op_in_unsafe_fn, clippy::allow_attributes)]
                $($fn_qualifiers)* fn $avx2_fn $($generic_params)* (
                    $($self_param)*, $($arg_name: $arg_ty,)* $cpu_name: crate::cpu::Avx2
                ) $(-> $ret_ty)? {
                    Self::$impl_fn $($turbofish)* ($self_kw, $($arg_name,)* $cpu_name)
                }
            }
            target_arch = "aarch64" => {
                $($inline_attrs)*
                #[target_feature(enable = "neon")]
                #[allow(unsafe_op_in_unsafe_fn, clippy::allow_attributes)]
                $($fn_qualifiers)* fn $neon_fn $($generic_params)* (
                    $($self_param)*, $($arg_name: $arg_ty,)* $cpu_name: crate::cpu::Neon
                ) $(-> $ret_ty)? {
                    Self::$impl_fn $($turbofish)* ($self_kw, $($arg_name,)* $cpu_name)
                }
            }
            _ => {}
        }
    };
}

pub(crate) use expand_method;

#[doc(hidden)]
pub mod __cpu_versions {
    pub(crate) use super::{
        attrs, expand, expand_func, expand_method, generic_param_end, generics, params, receiver,
        signature,
    };
}

/// Re-exported so call sites can import it from here, along with [`cpu_versions!`].
pub use macro_rules_attr::apply;
