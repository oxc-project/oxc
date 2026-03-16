//! Options for serializing / updating tokens.

// `#[inline(always)]` used on all methods as they're tiny, and to ensure compiler removes dead code
// resulting from static values
#![expect(clippy::inline_always)]

use crate::jsx_state::{JSXState, JSXStateJS, JSXStateTS};

/// Trait for options for serializing tokens.
///
/// # Token styles
///
/// Espree (JS) and TS-ESLint (TS) differ in several ways in the tokens they produce.
///
/// 1. `yield`, `let`, `static` used as identifiers (`obj = { yield: 1, let: 2, static: 3 };`)
///   * Espree emits these as `Keyword` tokens.
///   * TS-ESLint as `Identifier` tokens.
///
/// 2. Escaped identifiers (e.g. `\u0061`)
///   * Espree decodes escapes in the token `value`.
///   * TS-ESLint preserves the raw source text.
///
/// 3. JSX namespaced names (`<ns:tag>`)
///   * Espree emits `JSXIdentifier` tokens for both parts,
///   * TS-ESLint leaves them as their default token type (`Identifier`).
///
/// 4. Member expressions in JSX expressions (`<C x={a.b}>`)
///   * Espree emits them as `Identifier` tokens.
///   * TS-ESLint emits `JSXIdentifier` tokens for non-computed member expression identifiers
///     inside JSX expression containers.
///
/// # Options
///
/// This trait is implemented by 3 structs which can be passed as options to [`to_estree_tokens_json`] and
/// [`to_estree_tokens_pretty_json`]:
///
/// * [`ESTreeTokenOptions`]
/// * [`ESTreeTokenOptionsJS`]
/// * [`ESTreeTokenOptionsTS`]
///
/// The difference between them is:
/// * [`ESTreeTokenOptions`] supports both JS and TS styles, and branches at runtime to handle the differences.
/// * [`ESTreeTokenOptionsJS`] and [`ESTreeTokenOptionsTS`] only support a single style, and branches for the other
///   style are removed as dead code at compile time.
///
/// If your application only uses one style of tokens, use [`ESTreeTokenOptionsJS`] or [`ESTreeTokenOptionsTS`].
/// That will produce the smallest binary size and fastest runtime performance.
///
/// If your application uses either JS and TS style tokens depending on a condition only known at runtime,
/// [`ESTreeTokenOptions`] is likely preferable. It is marginally slower, but avoids compiling the code for serializing
/// tokens twice in the binary.
///
/// [`to_estree_tokens_json`]: crate::to_estree_tokens_json
/// [`to_estree_tokens_pretty_json`]: crate::to_estree_tokens_pretty_json
pub trait ESTreeTokenConfig {
    /// JSX state type.
    type JSXState: JSXState;

    /// Returns `true` if serializing in TS style.
    fn is_ts(&self) -> bool;

    /// Returns `true` if serializing in JS style.
    #[inline(always)]
    fn is_js(&self) -> bool {
        !self.is_ts()
    }
}

/// Options for serializing tokens in JS style.
///
/// Prefer this over [`ESTreeTokenOptions`] when your application only uses JS style tokens,
/// as the code paths for TS style can be removed.
///
/// If your application uses both JS and TS style tokens, [`ESTreeTokenOptions`] is probably preferable
/// as it can be used to serialize both styles without binary including all this code twice.
///
/// See [`ESTreeTokenConfig`] for more details.
#[derive(Clone, Copy, Default, Debug)]
pub struct ESTreeTokenOptionsJS;

impl ESTreeTokenConfig for ESTreeTokenOptionsJS {
    /// No-op JSX state, because no state is required for JS-style tokens.
    type JSXState = JSXStateJS;

    #[inline(always)]
    fn is_ts(&self) -> bool {
        false
    }
}

/// Options for serializing tokens in TS style.
///
/// Prefer this over [`ESTreeTokenOptions`] when your application only uses TS style tokens,
/// as the code paths for JS style can be removed.
///
/// If your application uses both JS and TS style tokens, [`ESTreeTokenOptions`] is probably preferable
/// as it can be used to serialize both styles without binary including all this code twice.
///
/// See [`ESTreeTokenConfig`] for more details.
#[derive(Clone, Copy, Default, Debug)]
pub struct ESTreeTokenOptionsTS;

impl ESTreeTokenConfig for ESTreeTokenOptionsTS {
    /// Working JSX state, because state is required for TS-style tokens.
    type JSXState = JSXStateTS;

    #[inline(always)]
    fn is_ts(&self) -> bool {
        true
    }
}

/// Options for serializing tokens in either JS or TS style.
///
/// If your application uses both JS and TS style tokens, [`ESTreeTokenOptions`] is probably preferable
/// over [`ESTreeTokenOptionsJS`] and [`ESTreeTokenOptionsTS`], as it can be used to serialize both styles
/// without binary including all this code twice.
///
/// See [`ESTreeTokenConfig`] for more details.
#[derive(Clone, Copy, Debug)]
pub struct ESTreeTokenOptions {
    is_ts: bool,
}

impl ESTreeTokenOptions {
    #[inline(always)]
    pub fn new(is_ts: bool) -> Self {
        Self { is_ts }
    }
}

impl ESTreeTokenConfig for ESTreeTokenOptions {
    /// Working JSX state, because state is required for TS-style tokens.
    type JSXState = JSXStateTS;

    #[inline(always)]
    fn is_ts(&self) -> bool {
        self.is_ts
    }
}
