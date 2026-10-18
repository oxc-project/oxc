/// Macro for either static or dynamic feature selection.
///
/// Arms use normal `match` syntax, but there must be exactly these 3 arms, in this order:
///
/// - `Avx2(...)`: Only compiled on x86_64.
/// - `Neon(...)`: Only compiled on aarch64.
/// - `Generic(...)`: Compiled everywhere.
///   On aarch64 with Neon enabled at compile time, it's compiled and type-checked,
///   but can never run, so the compiler removes it. See [`Generic`].
///
/// Each arm binds its CPU feature set type (e.g. `Avx2`), so it can be passed on.
/// Fallback arms (e.g. `_ => ...`) and match guards are not allowed.
///
/// Requiring all 3 arms means every call site says what to do on every platform.
/// It also makes the `match` exhaustive on every platform, without a fallback arm.
///
/// # Examples
///
/// ## Static selection
///
/// ```ignore
/// use crate::pipeline::cpu::{Cpu, cpu_select};
///
/// fn classify(&mut self, cpu: impl Cpu) {
///     // Compiles down to a single arm (no runtime branch)
///     cpu_select!(cpu, {
///         Avx2(cpu) => classify_avx2(self, cpu),
///         Neon(cpu) => classify_neon(self, cpu),
///         Generic(_) => classify_generic(self),
///     });
/// }
/// ```
///
/// ## Dynamic selection
///
/// ```ignore
/// use crate::pipeline::cpu::{DynCpu, cpu_select};
///
/// fn classify(cpu: DynCpu) {
///     // Runtime match over options available on the current platform
///     cpu_select!(cpu, {
///         Avx2(cpu) => classify_avx2(self, cpu),
///         Neon(cpu) => classify_neon(self, cpu),
///         Generic(_) => classify_generic(self),
///     });
/// }
/// ```
///
/// ## Expansion
///
/// Macro expands to:
///
/// ```ignore
/// match Cpu::to_dyn(cpu) {
///     #[cfg(target_arch = "x86_64")]
///     DynCpu::Avx2(cpu) => classify_avx2(self, cpu),
///
///     #[cfg(target_arch = "aarch64")]
///     DynCpu::Neon(cpu) => classify_neon(self, cpu),
///
///     DynCpu::Generic(_) => classify_generic(self),
/// }
/// ```
///
/// ## Implementation
///
/// Tokens of the arms are consumed one at a time, and copied into `arms`.
/// `Avx2(...) =>` is rewritten to `DynCpu::Avx2(...) =>`, with a `#[cfg]` attribute for x86_64.
/// `Neon(...) =>` is rewritten the same way, with a `#[cfg]` attribute for aarch64.
/// `Generic(...) =>` is rewritten to `DynCpu::Generic(...) =>`, with no `#[cfg]` attribute.
/// All other tokens are copied unchanged, so the arm bodies are parsed by the `match` itself.
///
/// State is a set of named slots, like in `cpu_versions!`:
///
/// * `rest` is the tokens which haven't been consumed yet.
/// * `seen` lists the arms which have been seen so far (e.g. `[ avx2, neon ]`).
/// * `arms` is the tokens of the rewritten arms so far.
/// * `cpu` is the expression being matched on.
///
/// The slots which tell the rules apart (`rest`, then `seen`) come first, and `cpu`, which is only
/// passed on, comes last. So when a rule doesn't match, the compiler finds out early.
///
/// Each arm's rule only matches if the arms before it have been seen, and it hasn't.
/// So the arms must come in that order, once each. Anything else is a compile error.
///
/// [`Generic`]: super::Generic
macro_rules! cpu_select {
    // Entry point
    ($cpu:expr, { $($arms:tt)* }) => {
        crate::pipeline::cpu::cpu_select!(
            @munch
            rest = [ $($arms)* ]
            seen = []
            arms = []
            cpu = $cpu
        )
    };

    // `Avx2` arm. Must be first.
    (
        @munch
        rest = [
            Avx2($($pat:tt)*) =>
            $($rest:tt)*
        ]
        seen = []
        arms = [ $($arms:tt)* ]
        cpu = $cpu:tt
    ) => {
        crate::pipeline::cpu::cpu_select!(
            @munch
            rest = [ $($rest)* ]
            seen = [ avx2 ]
            arms = [
                $($arms)*
                #[cfg(target_arch = "x86_64")]
                crate::pipeline::cpu::DynCpu::Avx2($($pat)*) =>
            ]
            cpu = $cpu
        )
    };

    // `Neon` arm. Must follow `Avx2` arm.
    (
        @munch
        rest = [
            Neon($($pat:tt)*) =>
            $($rest:tt)*
        ]
        seen = [ avx2 ]
        arms = [ $($arms:tt)* ]
        cpu = $cpu:tt
    ) => {
        crate::pipeline::cpu::cpu_select!(
            @munch
            rest = [ $($rest)* ]
            seen = [ avx2, neon ]
            arms = [
                $($arms)*
                #[cfg(target_arch = "aarch64")]
                crate::pipeline::cpu::DynCpu::Neon($($pat)*) =>
            ]
            cpu = $cpu
        )
    };

    // `Generic` arm. Must follow `Neon` arm.
    (
        @munch
        rest = [
            Generic($($pat:tt)*) =>
            $($rest:tt)*
        ]
        seen = [ avx2, neon ]
        arms = [ $($arms:tt)* ]
        cpu = $cpu:tt
    ) => {
        crate::pipeline::cpu::cpu_select!(
            @munch
            rest = [ $($rest)* ]
            seen = [ avx2, neon, generic ]
            arms = [
                $($arms)*
                crate::pipeline::cpu::DynCpu::Generic($($pat)*) =>
            ]
            cpu = $cpu
        )
    };

    // Any other `=>` - an arm which is out of order, repeated, not one of the 3 names,
    // a fallback (e.g. `_ =>`), or has a match guard
    (
        @munch
        rest = [
            =>
            $($_rest:tt)*
        ]
        $($_any:tt)*
    ) => {
        crate::pipeline::cpu::cpu_select!(@error)
    };

    // Any other token.
    //
    // Copy 1 token, and go round again.
    //
    // The arm rules only consume `Avx2(...) =>` etc, not the arm's body.
    // So the remaining tokens can't all be copied in one go. They start with the body,
    // and a `Neon(...) =>` which still needs rewriting may come after it.
    //
    // `macro_rules!` can't match "all tokens up to the next `Neon`",
    // because `$($x:tt)*` would also match `Neon`.
    // The only way to find the next arm is to step through the tokens.
    //
    // Consuming the body in the arm rules would avoid this, but would need 2 rules for each arm
    // (block body, or expression followed by `,`), and would reject comma-free bodies like `unsafe { ... }`.
    //
    // A delimited group (e.g. a block body) is a single token, so its contents are never inspected.
    // This rule must come after the arm rules and the error rule above,
    // because `$next:tt` also matches `Avx2` and `=>`.
    (
        @munch
        rest = [
            $next:tt
            $($rest:tt)*
        ]
        seen = $seen:tt
        arms = [ $($arms:tt)* ]
        cpu = $cpu:tt
    ) => {
        crate::pipeline::cpu::cpu_select!(
            @munch
            rest = [ $($rest)* ]
            seen = $seen
            arms = [
                $($arms)*
                $next
            ]
            cpu = $cpu
        )
    };

    // All tokens consumed, and all 3 arms seen
    (
        @munch
        rest = []
        seen = [ avx2, neon, generic ]
        arms = [ $($arms:tt)* ]
        cpu = $cpu:tt
    ) => {
        match crate::pipeline::cpu::Cpu::to_dyn($cpu) {
            $($arms)*
        }
    };

    // All tokens consumed, but an arm is missing.
    // The rules above handle every other case, so this rule matches anything.
    (@munch $($_any:tt)*) => {
        crate::pipeline::cpu::cpu_select!(@error)
    };

    // Error for any arms which don't follow the rules
    (@error) => {
        compile_error!(
            "`cpu_select!` must have arms `Avx2(...)`, `Neon(...)` and `Generic(...)`, in that order, once each"
        )
    };
}

pub(crate) use cpu_select;
