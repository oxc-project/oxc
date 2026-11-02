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
/// fn bm_any_avx2(bm: &[u64], _cpu: Avx2) -> bool {
///     // ...
/// }
/// ```
///
/// * `Avx2`: A parameter must prove that the CPU supports AVX2.
///   Any parameter can do that, if its type is `Avx2`, `Vec128<Avx2>` or `Vec256<Avx2>`.
///   A vector can only exist if the CPU supports its feature set.
///   The type must be written with its `<Avx2>`, because the macro only sees tokens.
///
///   The function gets `#[cfg(target_arch = "x86_64")]`. What else it gets depends on its kind:
///   * `#[inline(always)]`: Nothing, because it can't be combined with `#[target_feature]`.
///     It's compiled as part of its callers, with their features.
///   * `unsafe fn`: It gets `#[target_feature(enable = "avx2,bmi2,popcnt")]`.
///   * Any other function: It's wrapped in a safe function. See below.
/// * `Neon`: The same, with `Neon`, `Vec128<Neon>` or `Vec256<Neon>`,
///   `#[cfg(target_arch = "aarch64")]` and `#[target_feature(enable = "neon")]`.
/// * `Generic`: No parameter can have a CPU type
///   (`Avx2`, `Neon`, `Generic`, `DynCpu` or `impl Cpu`), or be a `Vec128` or `Vec256`.
///   A generic version only runs where the CPU feature set is [`BASELINE`]'s,
///   so it uses [`BASELINE`] where it needs a CPU type. The function gets no attributes.
///
/// A function which isn't wrapped is otherwise output unchanged, from its original tokens.
/// So it compiles, and is linted, exactly as written.
///
/// ## Safe functions
///
/// Calling a function with `#[target_feature]` needs an `unsafe` block, unless the caller
/// has the same features. There's no reason for that when the function has to receive proof that
/// the CPU supports those features. So the function is wrapped in one which is safe to call.
///
/// ```ignore
/// #[apply(cpu_version(Avx2))]
/// #[inline(never)]
/// fn foo<const N: usize>(mut n: u64, cpu: Avx2) -> bool {
///     // ...
/// }
/// ```
///
/// expands to:
///
/// ```ignore
/// #[cfg(target_arch = "x86_64")]
/// #[inline(always)]
/// fn foo<const N: usize>(n: u64, cpu: Avx2) -> bool {
///     #[inline(never)]
///     #[target_feature(enable = "avx2,bmi2,popcnt")]
///     fn __impl<const N: usize>(mut n: u64, cpu: Avx2) -> bool {
///         // ...
///     }
///
///     let _: crate::cpu::Avx2 = cpu;
///
///     // SAFETY: The check above shows this parameter is a type which guarantees
///     // that the CPU supports the target features which `__impl` has
///     unsafe { __impl::<N>(n, cpu) }
/// }
/// ```
///
/// The `let _` is required for soundness. The macro only sees tokens, so it can't tell that
/// a parameter's type `Avx2` is really `crate::cpu::Avx2`. It might be a type alias or a generic
/// parameter with that name. The `let _` fails to compile if so, and generates no code.
///
/// Like [`cpu_versions!`], `#[inline]` and `#[inline(never)]` attributes go on `__impl`,
/// all other attributes go on `foo`, and `mut` only goes on `__impl`'s parameters.
///
/// A nested function can't use `self`, so for a method with a `self` parameter, `__impl` is
/// a private method next to it, called `__foo_impl`. So it can't be in a trait implementation.
///
/// A nested function can't use `Self` or the generic parameters of an `impl` either.
/// So a wrapped associated function with no `self` parameter can't use them.
///
/// The parameters must be `name: Type` or `mut name: Type` (no other patterns),
/// and there can't be a `where` clause.
///
/// ## Implementation
///
/// The macro works in stages, like [`cpu_versions!`]. The stages move the function's tokens
/// into `item` one at a time:
///
/// * `attrs!`: Attributes. Records whether one is `#[inline(always)]`.
/// * `head!`: Visibility, `const` and `unsafe`, then `fn` and the function's name.
///   Records whether the function is `unsafe`.
/// * `generics!`: Generic parameters, then the parameters' `(...)`.
///   `angles` holds a `<` for each `<` not yet closed, so a `(...)` in the generics
///   (e.g. `F: Fn(u8)`) isn't taken as the parameters.
/// * `tail!`: Return type and body, up to the `(Avx2)` which `#[apply]` adds after the function.
/// * `check!`: Checks the parameters, then outputs the function with its new attributes,
///   or passes it on to be wrapped.
///
/// `style` is which of these the function gets:
///
/// * `inline_always`: Output unchanged, apart from `#[cfg]`.
/// * `target_feature`: Output unchanged, apart from `#[cfg]` and `#[target_feature]`.
/// * `wrapped`: Wrapped in a safe function, which `wrapped_fn!` or `wrapped_method!` generates.
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
            style = wrapped
            item = []
        );
    };
}

pub(crate) use cpu_version;

/// Moves the attributes into `item`.
/// If one is `#[inline(always)]`, sets `style` to `inline_always`.
#[doc(hidden)]
macro_rules! attrs {
    // `#[inline(always)]`
    (
        rest_to_match = [ #[inline(always)] $($_rest:tt)* ]
        rest_to_capture = [ $hash:tt $attr:tt $($rest:tt)* ]
        style = $_style:tt
        item = [ $($item:tt)* ]
    ) => {
        crate::cpu::__cpu_version::attrs!(
            rest_to_match = [ $($rest)* ]
            rest_to_capture = [ $($rest)* ]
            style = inline_always
            item = [ $($item)* $hash $attr ]
        );
    };

    // Any other attribute
    (
        rest_to_match = [ # $_attr:tt $($_rest:tt)* ]
        rest_to_capture = [ $hash:tt $attr:tt $($rest:tt)* ]
        style = $style:tt
        item = [ $($item:tt)* ]
    ) => {
        crate::cpu::__cpu_version::attrs!(
            rest_to_match = [ $($rest)* ]
            rest_to_capture = [ $($rest)* ]
            style = $style
            item = [ $($item)* $hash $attr ]
        );
    };

    // No attributes left
    (
        rest_to_match = $_rest_to_match:tt
        rest_to_capture = $rest:tt
        style = $style:tt
        item = $item:tt
    ) => {
        crate::cpu::__cpu_version::head!(
            rest_to_match = $rest
            rest_to_capture = $rest
            style = $style
            item = $item
        );
    };
}

pub(crate) use attrs;

/// Moves the tokens before `fn` into `item`, then `fn` and the function's name.
/// If the function is `unsafe`, sets `style` to `target_feature`.
#[doc(hidden)]
macro_rules! head {
    // `fn`, then the function's name
    (
        rest_to_match = [ fn $($_rest:tt)* ]
        rest_to_capture = [ $fn_kw:tt $fn_name:tt $($rest:tt)* ]
        style = $style:tt
        item = [ $($item:tt)* ]
    ) => {
        crate::cpu::__cpu_version::generics!(
            rest_to_match = [ $($rest)* ]
            rest_to_capture = [ $($rest)* ]
            angles = []
            style = $style
            item = [ $($item)* $fn_kw $fn_name ]
        );
    };

    // `unsafe`. Its callers need an `unsafe` block anyway, so it isn't wrapped.
    // An `#[inline(always)]` function keeps that style.
    (
        rest_to_match = [ unsafe $($_rest:tt)* ]
        rest_to_capture = [ $token:tt $($rest:tt)* ]
        style = wrapped
        item = [ $($item:tt)* ]
    ) => {
        crate::cpu::__cpu_version::head!(
            rest_to_match = [ $($rest)* ]
            rest_to_capture = [ $($rest)* ]
            style = target_feature
            item = [ $($item)* $token ]
        );
    };

    // Visibility, `const`, or `unsafe` on an `#[inline(always)]` function
    (
        rest_to_match = $_rest_to_match:tt
        rest_to_capture = [ $token:tt $($rest:tt)* ]
        style = $style:tt
        item = [ $($item:tt)* ]
    ) => {
        crate::cpu::__cpu_version::head!(
            rest_to_match = [ $($rest)* ]
            rest_to_capture = [ $($rest)* ]
            style = $style
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
        style = $style:tt
        item = [ $($item:tt)* ]
    ) => {
        crate::cpu::__cpu_version::tail!(
            rest = [ $($rest)* ]
            params = [ $($params)* ]
            style = $style
            item = [ $($item)* $params_group ]
        );
    };

    // `<`
    (
        rest_to_match = [ < $($_rest:tt)* ]
        rest_to_capture = [ $token:tt $($rest:tt)* ]
        angles = [ $($angles:tt)* ]
        style = $style:tt
        item = [ $($item:tt)* ]
    ) => {
        crate::cpu::__cpu_version::generics!(
            rest_to_match = [ $($rest)* ]
            rest_to_capture = [ $($rest)* ]
            angles = [ < $($angles)* ]
            style = $style
            item = [ $($item)* $token ]
        );
    };

    // `>`
    (
        rest_to_match = [ > $($_rest:tt)* ]
        rest_to_capture = [ $token:tt $($rest:tt)* ]
        angles = [ < $($angles:tt)* ]
        style = $style:tt
        item = [ $($item:tt)* ]
    ) => {
        crate::cpu::__cpu_version::generics!(
            rest_to_match = [ $($rest)* ]
            rest_to_capture = [ $($rest)* ]
            angles = [ $($angles)* ]
            style = $style
            item = [ $($item)* $token ]
        );
    };

    // `>>` is a single token, which closes 2 `<`s
    (
        rest_to_match = [ >> $($_rest:tt)* ]
        rest_to_capture = [ $token:tt $($rest:tt)* ]
        angles = [ < < $($angles:tt)* ]
        style = $style:tt
        item = [ $($item:tt)* ]
    ) => {
        crate::cpu::__cpu_version::generics!(
            rest_to_match = [ $($rest)* ]
            rest_to_capture = [ $($rest)* ]
            angles = [ $($angles)* ]
            style = $style
            item = [ $($item)* $token ]
        );
    };

    // Any other token
    (
        rest_to_match = $_rest_to_match:tt
        rest_to_capture = [ $token:tt $($rest:tt)* ]
        angles = $angles:tt
        style = $style:tt
        item = [ $($item:tt)* ]
    ) => {
        crate::cpu::__cpu_version::generics!(
            rest_to_match = [ $($rest)* ]
            rest_to_capture = [ $($rest)* ]
            angles = $angles
            style = $style
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
        style = $style:tt
        item = $item:tt
    ) => {
        crate::cpu::__cpu_version::check!(
            kind = $kind
            params = $params
            style = $style
            item = $item
        );
    };

    // Return type, `where` clause or body
    (
        rest = [ $token:tt $($rest:tt)+ ]
        params = $params:tt
        style = $style:tt
        item = [ $($item:tt)* ]
    ) => {
        crate::cpu::__cpu_version::tail!(
            rest = [ $($rest)+ ]
            params = $params
            style = $style
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
/// * `Avx2` and `Neon`: Until a parameter proves that the CPU supports the feature set.
///   That's a parameter with that CPU type, or a `Vec128` or `Vec256` of that feature set.
///   `proof` records its name, and the full path of the type it must be.
/// * `Generic`: Until none are left. No parameter can be `cpu: Avx2` etc along the way.
///
/// A `wrapped` function goes through the same stages as a function using [`cpu_versions!`],
/// which parse it, and then call `wrapped_fn!` or `wrapped_method!` instead of generating
/// the usual output. `version` holds the slots they need which those stages don't.
///
/// [`cpu_versions!`]: super::cpu_versions!
#[doc(hidden)]
macro_rules! check {
    // `Avx2` version, with a parameter which proves the CPU supports AVX2
    (kind = Avx2 params = [ $name:ident: Avx2 $(, $($_rest:tt)*)? ] $($slots:tt)*) => {
        crate::cpu::__cpu_version::check!(
            avx2
            proof = {
                name = $name
                ty = [ crate::cpu::Avx2 ]
            }
            $($slots)*
        );
    };
    (kind = Avx2 params = [ $name:ident: Vec128<Avx2> $(, $($_rest:tt)*)? ] $($slots:tt)*) => {
        crate::cpu::__cpu_version::check!(
            avx2
            proof = {
                name = $name
                ty = [ crate::simd::Vec128<crate::cpu::Avx2> ]
            }
            $($slots)*
        );
    };
    (kind = Avx2 params = [ $name:ident: Vec256<Avx2> $(, $($_rest:tt)*)? ] $($slots:tt)*) => {
        crate::cpu::__cpu_version::check!(
            avx2
            proof = {
                name = $name
                ty = [ crate::simd::Vec256<crate::cpu::Avx2> ]
            }
            $($slots)*
        );
    };
    (
        avx2
        proof = $proof:tt
        style = $style:tt
        item = $item:tt
    ) => {
        crate::cpu::__cpu_version::check!(
            output
            style = $style
            cfg = [ target_arch = "x86_64" ]
            features = "avx2,bmi2,popcnt"
            proof = $proof
            item = $item
        );
    };

    // `Neon` version, with a parameter which proves the CPU supports Neon
    (kind = Neon params = [ $name:ident: Neon $(, $($_rest:tt)*)? ] $($slots:tt)*) => {
        crate::cpu::__cpu_version::check!(
            neon
            proof = {
                name = $name
                ty = [ crate::cpu::Neon ]
            }
            $($slots)*
        );
    };
    (kind = Neon params = [ $name:ident: Vec128<Neon> $(, $($_rest:tt)*)? ] $($slots:tt)*) => {
        crate::cpu::__cpu_version::check!(
            neon
            proof = {
                name = $name
                ty = [ crate::simd::Vec128<crate::cpu::Neon> ]
            }
            $($slots)*
        );
    };
    (kind = Neon params = [ $name:ident: Vec256<Neon> $(, $($_rest:tt)*)? ] $($slots:tt)*) => {
        crate::cpu::__cpu_version::check!(
            neon
            proof = {
                name = $name
                ty = [ crate::simd::Vec256<crate::cpu::Neon> ]
            }
            $($slots)*
        );
    };
    (
        neon
        proof = $proof:tt
        style = $style:tt
        item = $item:tt
    ) => {
        crate::cpu::__cpu_version::check!(
            output
            style = $style
            cfg = [ target_arch = "aarch64" ]
            features = "neon"
            proof = $proof
            item = $item
        );
    };

    // Output an `#[inline(always)]` `Avx2` or `Neon` version
    (
        output
        style = inline_always
        cfg = [ $($cfg:tt)* ]
        features = $_features:tt
        proof = $_proof:tt
        item = [ $($item:tt)* ]
    ) => {
        #[cfg($($cfg)*)]
        $($item)*
    };

    // Output an `unsafe` `Avx2` or `Neon` version
    (
        output
        style = target_feature
        cfg = [ $($cfg:tt)* ]
        features = $features:tt
        proof = $_proof:tt
        item = [ $($item:tt)* ]
    ) => {
        #[cfg($($cfg)*)]
        #[target_feature(enable = $features)]
        $($item)*
    };

    // Output any other `Avx2` or `Neon` version, wrapped in a safe function
    (
        output
        style = wrapped
        cfg = $cfg:tt
        features = $features:tt
        proof = $proof:tt
        item = $item:tt
    ) => {
        crate::cpu::__cpu_versions::attrs!(
            item = $item
            version = { cfg = $cfg features = $features proof = $proof }
        );
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
    (
        kind = Generic
        params = [ $_name:ident: Vec128<$_cpu:ident> $(, $($_rest:tt)*)? ]
        $($_slots:tt)*
    ) => {
        crate::cpu::__cpu_version::check!(generic_with_cpu);
    };
    (
        kind = Generic
        params = [ $_name:ident: Vec256<$_cpu:ident> $(, $($_rest:tt)*)? ]
        $($_slots:tt)*
    ) => {
        crate::cpu::__cpu_version::check!(generic_with_cpu);
    };
    (generic_with_cpu) => {
        compile_error!("`cpu_version(Generic)` functions can't have a parameter with a CPU or SIMD Vec type");
    };

    // `Generic` version, all parameters checked
    (
        kind = Generic
        params = []
        style = $_style:tt
        item = [ $($item:tt)* ]
    ) => {
        $($item)*
    };

    // Drop a token from `params`
    (
        kind = $kind:ident
        params = [ $_token:tt $($params:tt)* ]
        style = $style:tt
        item = $item:tt
    ) => {
        crate::cpu::__cpu_version::check!(
            kind = $kind
            params = [ $($params)* ]
            style = $style
            item = $item
        );
    };

    // `Avx2` or `Neon` version, with no parameter which proves the CPU supports the feature set
    (kind = Avx2 params = [] $($_slots:tt)*) => {
        compile_error!(
            "`cpu_version(Avx2)` functions must have an `Avx2`, `Vec128<Avx2>` or `Vec256<Avx2>` parameter"
        );
    };
    (kind = Neon params = [] $($_slots:tt)*) => {
        compile_error!(
            "`cpu_version(Neon)` functions must have a `Neon`, `Vec128<Neon>` or `Vec256<Neon>` parameter"
        );
    };

    // Not `Avx2`, `Neon` or `Generic`
    ($($_any:tt)*) => {
        compile_error!("`cpu_version!` takes `Avx2`, `Neon` or `Generic`");
    };
}

pub(crate) use check;

/// Generates the output for a wrapped function. `__impl` is a function inside it.
#[doc(hidden)]
macro_rules! wrapped_fn {
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
        version = {
            cfg = [ $($cfg:tt)* ]
            features = $features:tt
            proof = {
                name = $proof_name:ident
                ty = [ $($proof_ty:tt)* ]
            }
        }
        params = [$(
            {
                mut_kw = [ $($mut_kw:tt)? ]
                arg_name = $arg_name:ident
                arg_ty = $arg_ty:ty
            }
        )*]
    ) => {
        #[cfg($($cfg)*)]
        $($outer_attrs)*
        #[inline(always)]
        $vis $($fn_qualifiers)* fn $fn_name $($generic_params)* (
            $($arg_name: $arg_ty,)*
        ) $(-> $ret_ty)? {
            $($inline_attrs)*
            #[target_feature(enable = $features)]
            $($fn_qualifiers)* fn __impl $($generic_params)* (
                $($($mut_kw)? $arg_name: $arg_ty,)*
            ) $(-> $ret_ty)?
            $fn_body

            // `check!` only matched the type by name. See the docs on `cpu_version!`.
            let _: $($proof_ty)* = $proof_name;

            // SAFETY: The check above shows this parameter is a type which guarantees
            // that the CPU supports the target features which `__impl` has
            unsafe { __impl $($turbofish)* ($($arg_name,)*) }
        }
    };
}

pub(crate) use wrapped_fn;

/// Generates the output for a wrapped method. `__*_impl` is a private method next to it.
///
/// The first rule makes the name of `__*_impl`,
/// in the same way as `cpu_versions!`'s `expand_method!`.
#[doc(hidden)]
#[cfg_attr(not(test), expect(unused_macros, reason = "only tests have wrapped methods so far"))]
macro_rules! wrapped_method {
    // Entry point - make the name of `__*_impl`
    (
        fn_name = $fn_name:ident
        $($rest:tt)*
    ) => {
        pastey::paste! {
            crate::cpu::__cpu_version::wrapped_method!(
                fn_names = {
                    fn_name = $fn_name
                    impl_fn = [<__ $fn_name _impl>]
                }
                $($rest)*
            );
        }
    };

    // Name made - generate the output
    (
        fn_names = {
            fn_name = $fn_name:ident
            impl_fn = $impl_fn:ident
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
        version = {
            cfg = [ $($cfg:tt)* ]
            features = $features:tt
            proof = {
                name = $proof_name:ident
                ty = [ $($proof_ty:tt)* ]
            }
        }
        params = [$(
            {
                mut_kw = [ $($mut_kw:tt)? ]
                arg_name = $arg_name:ident
                arg_ty = $arg_ty:ty
            }
        )*]
    ) => {
        #[cfg($($cfg)*)]
        $($outer_attrs)*
        #[inline(always)]
        $vis $($fn_qualifiers)* fn $fn_name $($generic_params)* (
            $($self_param)*, $($arg_name: $arg_ty,)*
        ) $(-> $ret_ty)? {
            // `check!` only matched the type by name. See the docs on `cpu_version!`.
            let _: $($proof_ty)* = $proof_name;

            // SAFETY: The check above shows this parameter is a type which guarantees
            // that the CPU supports the target features which `__*_impl` has
            unsafe { Self::$impl_fn $($turbofish)* ($self_kw, $($arg_name,)*) }
        }

        #[cfg($($cfg)*)]
        $($inline_attrs)*
        #[target_feature(enable = $features)]
        $($fn_qualifiers)* fn $impl_fn $($generic_params)* (
            $($self_mut_kw)? $($self_param)*,
            $($($mut_kw)? $arg_name: $arg_ty,)*
        ) $(-> $ret_ty)?
        $fn_body
    };
}

pub(crate) use wrapped_method;

#[doc(hidden)]
pub mod __cpu_version {
    pub(crate) use super::{attrs, check, generics, head, tail, wrapped_fn};

    #[cfg_attr(
        not(test),
        expect(unused_imports, reason = "only tests have wrapped methods so far")
    )]
    pub(crate) use super::wrapped_method;
}
