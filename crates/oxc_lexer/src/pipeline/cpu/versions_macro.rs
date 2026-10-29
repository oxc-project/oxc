/// Macro to turn a function into one with a separate entry point for each CPU feature set.
///
/// Used for pipeline stages, and for functions they call which aren't always inlined.
/// Each entry point is compiled with its feature set's target features enabled,
/// so SIMD functions which the function calls can be inlined into it.
///
/// Applied to a function with `#[apply(cpu_versions)]`:
///
/// ```ignore
/// use crate::pipeline::cpu::{Cpu, apply, cpu_versions};
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
/// `#[inline(always)]` isn't allowed, because it can't be combined with `#[target_feature]`.
///
/// The last parameter must be `cpu: impl Cpu` (any name instead of `cpu` is fine).
/// The other parameters must be `name: Type` or `mut name: Type` (no other patterns),
/// and can't be called `__impl`, `__avx2`, etc.
/// `mut` only goes on `__impl`'s parameters. The other functions just pass the values on.
///
/// The function can be a `const fn`, `unsafe fn` or `const unsafe fn`.
///
/// The function can have const generic parameters (e.g. `<const IS_TS: bool>`), but no other generics.
/// A nested function can't use its parent's generic parameters, so each nested function declares them again,
/// and calls pass them on (`__impl::<IS_TS>(..)`).
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
///                 DynCpu::Generic(cpu) => __generic(src, n, cpu),
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
///                 DynCpu::Generic(cpu) => __generic(src, n, cpu),
///             }
///         }
///         _ => {
///             match to_dyn(cpu) {
///                 DynCpu::Generic(cpu) => __generic(src, n, cpu),
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
/// * `attrs!`: Sorts the attributes, then parses the rest of the function.
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
/// `fn_body` is matched as `tt` even where it's used. `attrs!` has already checked that it's a block,
/// and matching it as `block` again would parse the whole body again.
///
/// A stage which needs more slots while it works adds them in its first rule.
/// Its other rules call the stage again until it's done, and then call the next stage.
///
/// [`cpu_select!`]: super::cpu_select!
macro_rules! cpu_versions {
    ($($item:tt)*) => {
        crate::pipeline::cpu::__cpu_versions::attrs!(item = [ $($item)* ]);
    };
}

pub(crate) use cpu_versions;

/// Moves the attributes into 2 lists one at a time, then parses the rest of the function.
/// `#[inline]` attributes go in `inline_attrs`, which go on the entry points.
/// All other attributes go in `outer_attrs`, which go on the outer function.
///
/// The function's tokens are kept twice, as `rest_to_match` and `rest_to_capture`.
/// `rest_to_match` is matched against patterns (e.g. `#[inline ...]`), and `rest_to_capture`
/// is captured as `tt`s.
///
/// Patterns can't capture the tokens they match literally, so this is how `#` and `[...]` are captured.
/// Captured `tt`s keep their original spans, so a compiler error about an attribute points at
/// the attribute where the macro is used, not at this macro.
/// e.g. `#[inline(always)]`, which can't be combined with `#[target_feature]`.
///
/// When no attributes are left, a rule for each of `fn`, `unsafe fn`, `const fn` and `const unsafe fn`
/// records the keywords, and passes the rest of the function on in `signature`.
/// The rule which matches `signature` puts the function into `func`,
/// which bundles everything about the function except its parameters:
///
/// * `fn_qualifiers` is `[]`, `[ unsafe ]`, `[ const ]` or `[ const unsafe ]`.
///   It's copied onto every function.
/// * Const generic parameters are recorded twice:
///   * `generic_params` is the declaration `<const N: Type, ..>`, which goes on every function.
///   * `turbofish` is the arguments `::<N, ..>` for calls.
/// * `generic_params` is `[()]` and `turbofish` is `[]` if there are none.
/// * `generic_params`' contents are wrapped in `(...)`, because `pastey` would treat `[<...>]` as a paste.
///
/// The parameters are passed on unparsed, in `params`.
#[doc(hidden)]
macro_rules! attrs {
    // Entry point
    (item = [ $($item:tt)* ]) => {
        crate::pipeline::cpu::__cpu_versions::attrs!(
            rest_to_match = [ $($item)* ]
            rest_to_capture = [ $($item)* ]
            outer_attrs = []
            inline_attrs = []
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
    ) => {
        crate::pipeline::cpu::__cpu_versions::attrs!(
            rest_to_match = [ $($rest)* ]
            rest_to_capture = [ $($rest)* ]
            outer_attrs = $outer_attrs
            inline_attrs = [
                $($inline_attrs)*
                $hash $attr
            ]
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
    ) => {
        crate::pipeline::cpu::__cpu_versions::attrs!(
            rest_to_match = [ $($rest)* ]
            rest_to_capture = [ $($rest)* ]
            outer_attrs = [
                $($outer_attrs)*
                $hash $attr
            ]
            inline_attrs = $inline_attrs
        );
    };

    // No attributes left, `fn`
    (
        rest_to_match = $_rest_to_match:tt
        rest_to_capture = [ $vis:vis fn $($rest:tt)* ]
        outer_attrs = $outer_attrs:tt
        inline_attrs = $inline_attrs:tt
    ) => {
        crate::pipeline::cpu::__cpu_versions::attrs!(
            signature = [ $($rest)* ]
            vis = $vis
            fn_qualifiers = []
            outer_attrs = $outer_attrs
            inline_attrs = $inline_attrs
        );
    };

    // No attributes left, `unsafe fn`
    (
        rest_to_match = $_rest_to_match:tt
        rest_to_capture = [ $vis:vis unsafe fn $($rest:tt)* ]
        outer_attrs = $outer_attrs:tt
        inline_attrs = $inline_attrs:tt
    ) => {
        crate::pipeline::cpu::__cpu_versions::attrs!(
            signature = [ $($rest)* ]
            vis = $vis
            fn_qualifiers = [ unsafe ]
            outer_attrs = $outer_attrs
            inline_attrs = $inline_attrs
        );
    };

    // No attributes left, `const fn`
    (
        rest_to_match = $_rest_to_match:tt
        rest_to_capture = [ $vis:vis const fn $($rest:tt)* ]
        outer_attrs = $outer_attrs:tt
        inline_attrs = $inline_attrs:tt
    ) => {
        crate::pipeline::cpu::__cpu_versions::attrs!(
            signature = [ $($rest)* ]
            vis = $vis
            fn_qualifiers = [ const ]
            outer_attrs = $outer_attrs
            inline_attrs = $inline_attrs
        );
    };

    // No attributes left, `const unsafe fn`
    (
        rest_to_match = $_rest_to_match:tt
        rest_to_capture = [ $vis:vis const unsafe fn $($rest:tt)* ]
        outer_attrs = $outer_attrs:tt
        inline_attrs = $inline_attrs:tt
    ) => {
        crate::pipeline::cpu::__cpu_versions::attrs!(
            signature = [ $($rest)* ]
            vis = $vis
            fn_qualifiers = [ const unsafe ]
            outer_attrs = $outer_attrs
            inline_attrs = $inline_attrs
        );
    };

    // The rest of the function, after `fn`
    (
        signature = [
            $fn_name:ident
            $(<$(const $const_name:ident: $const_ty:ty),* $(,)?>)?
            ($($params:tt)*) $(-> $ret_ty:ty)? $fn_body:block
        ]
        vis = $vis:vis
        fn_qualifiers = $fn_qualifiers:tt
        outer_attrs = $outer_attrs:tt
        inline_attrs = $inline_attrs:tt
    ) => {
        crate::pipeline::cpu::__cpu_versions::receiver!(
            params = [ $($params)* ]
            func = {
                fn_name = $fn_name
                outer_attrs = $outer_attrs
                inline_attrs = $inline_attrs
                vis = $vis
                fn_qualifiers = $fn_qualifiers
                // Extra `(...)` parens to prevent `pastey` matching `[<...>]`
                generic_params = [( $(<$(const $const_name: $const_ty),*>)? )]
                turbofish = [ $(::<$($const_name),*>)? ]
                ret_ty = [ $($ret_ty)? ]
                fn_body = $fn_body
            }
        );
    };

    // Not a function, or has generic parameters which aren't `const`
    ($($_any:tt)*) => {
        compile_error!("`cpu_versions!` must be applied to a `fn` with no generics except `const` ones");
    };
}

pub(crate) use attrs;

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
        crate::pipeline::cpu::__cpu_versions::receiver!(
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
        crate::pipeline::cpu::__cpu_versions::params!(
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
        crate::pipeline::cpu::__cpu_versions::params!(
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
        crate::pipeline::cpu::__cpu_versions::params!(
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
        crate::pipeline::cpu::__cpu_versions::params!(
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
        crate::pipeline::cpu::__cpu_versions::params!(
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
        crate::pipeline::cpu::__cpu_versions::params!(
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
        crate::pipeline::cpu::__cpu_versions::expand!(
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
        crate::pipeline::cpu::__cpu_versions::params!(
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
        crate::pipeline::cpu::__cpu_versions::params!(
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
#[doc(hidden)]
macro_rules! expand {
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
        crate::pipeline::cpu::__cpu_versions::expand_method!(
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
        crate::pipeline::cpu::__cpu_versions::expand_func!(
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
            $($fn_qualifiers)* fn __generic $($generic_params)* (
                $($arg_name: $arg_ty,)* $cpu_name: crate::pipeline::cpu::Generic
            ) $(-> $ret_ty)? {
                __impl $($turbofish)* ($($arg_name,)* $cpu_name)
            }

            cfg_select! {
                target_arch = "x86_64" => {
                    $($inline_attrs)*
                    #[target_feature(enable = "avx2,bmi2,popcnt")]
                    $($fn_qualifiers)* fn __avx2 $($generic_params)* (
                        $($arg_name: $arg_ty,)* $cpu_name: crate::pipeline::cpu::Avx2
                    ) $(-> $ret_ty)? {
                        __impl $($turbofish)* ($($arg_name,)* $cpu_name)
                    }

                    match crate::pipeline::cpu::to_dyn($cpu_name) {
                        // SAFETY: `Avx2` type guarantees the CPU supports the features `__avx2` uses
                        crate::pipeline::cpu::DynCpu::Avx2($cpu_name) => unsafe {
                            __avx2 $($turbofish)* ($($arg_name,)* $cpu_name)
                        },
                        crate::pipeline::cpu::DynCpu::Generic($cpu_name) => {
                            __generic $($turbofish)* ($($arg_name,)* $cpu_name)
                        }
                    }
                }
                target_arch = "aarch64" => {
                    $($inline_attrs)*
                    #[target_feature(enable = "neon")]
                    $($fn_qualifiers)* fn __neon $($generic_params)* (
                        $($arg_name: $arg_ty,)* $cpu_name: crate::pipeline::cpu::Neon
                    ) $(-> $ret_ty)? {
                        __impl $($turbofish)* ($($arg_name,)* $cpu_name)
                    }

                    match crate::pipeline::cpu::to_dyn($cpu_name) {
                        // SAFETY: `Neon` type guarantees the CPU supports the features `__neon` uses
                        crate::pipeline::cpu::DynCpu::Neon($cpu_name) => unsafe {
                            __neon $($turbofish)* ($($arg_name,)* $cpu_name)
                        },
                        crate::pipeline::cpu::DynCpu::Generic($cpu_name) => {
                            __generic $($turbofish)* ($($arg_name,)* $cpu_name)
                        }
                    }
                }
                _ => {
                    match crate::pipeline::cpu::to_dyn($cpu_name) {
                        crate::pipeline::cpu::DynCpu::Generic($cpu_name) => {
                            __generic $($turbofish)* ($($arg_name,)* $cpu_name)
                        }
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
            crate::pipeline::cpu::__cpu_versions::expand_method!(
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
        $vis $($fn_qualifiers)* fn $fn_name $($generic_params)* (
            $($self_param)*, $($arg_name: $arg_ty,)* $cpu_name: impl $cpu_trait
        ) $(-> $ret_ty)? {
            cfg_select! {
                target_arch = "x86_64" => {
                    match crate::pipeline::cpu::to_dyn($cpu_name) {
                        // SAFETY: `Avx2` type guarantees the CPU supports the features `__*_avx2` uses
                        crate::pipeline::cpu::DynCpu::Avx2($cpu_name) => unsafe {
                            Self::$avx2_fn $($turbofish)* ($self_kw, $($arg_name,)* $cpu_name)
                        },
                        crate::pipeline::cpu::DynCpu::Generic($cpu_name) => {
                            Self::$generic_fn $($turbofish)* ($self_kw, $($arg_name,)* $cpu_name)
                        }
                    }
                }
                target_arch = "aarch64" => {
                    match crate::pipeline::cpu::to_dyn($cpu_name) {
                        // SAFETY: `Neon` type guarantees the CPU supports the features `__*_neon` uses
                        crate::pipeline::cpu::DynCpu::Neon($cpu_name) => unsafe {
                            Self::$neon_fn $($turbofish)* ($self_kw, $($arg_name,)* $cpu_name)
                        },
                        crate::pipeline::cpu::DynCpu::Generic($cpu_name) => {
                            Self::$generic_fn $($turbofish)* ($self_kw, $($arg_name,)* $cpu_name)
                        }
                    }
                }
                _ => {
                    match crate::pipeline::cpu::to_dyn($cpu_name) {
                        crate::pipeline::cpu::DynCpu::Generic($cpu_name) => {
                            Self::$generic_fn $($turbofish)* ($self_kw, $($arg_name,)* $cpu_name)
                        }
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
        $($fn_qualifiers)* fn $generic_fn $($generic_params)* (
            $($self_param)*, $($arg_name: $arg_ty,)* $cpu_name: crate::pipeline::cpu::Generic
        ) $(-> $ret_ty)? {
            Self::$impl_fn $($turbofish)* ($self_kw, $($arg_name,)* $cpu_name)
        }

        cfg_select! {
            target_arch = "x86_64" => {
                $($inline_attrs)*
                #[target_feature(enable = "avx2,bmi2,popcnt")]
                $($fn_qualifiers)* fn $avx2_fn $($generic_params)* (
                    $($self_param)*, $($arg_name: $arg_ty,)* $cpu_name: crate::pipeline::cpu::Avx2
                ) $(-> $ret_ty)? {
                    Self::$impl_fn $($turbofish)* ($self_kw, $($arg_name,)* $cpu_name)
                }
            }
            target_arch = "aarch64" => {
                $($inline_attrs)*
                #[target_feature(enable = "neon")]
                $($fn_qualifiers)* fn $neon_fn $($generic_params)* (
                    $($self_param)*, $($arg_name: $arg_ty,)* $cpu_name: crate::pipeline::cpu::Neon
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
    pub(crate) use super::{attrs, expand, expand_func, expand_method, params, receiver};
}

/// Re-exported so call sites can import it from here, along with [`cpu_versions!`].
pub use macro_rules_attr::apply;
