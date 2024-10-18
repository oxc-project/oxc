use std::{borrow::Cow, num::NonZeroU32, ops::Deref};

use oxc_span::{Atom, GetSpan, Span};

use super::{Entropy, SecretsEnum};

/// A credential discovered in source code.
///
/// Could be an API key, an auth token, or any other sensitive information.
#[allow(clippy::struct_field_names)]
#[derive(Debug, Clone)]
pub struct Secret<'a> {
    secret: &'a str,
    /// Secret span
    span: Span,
    /// TODO: find and pass identifiers once we have rules that need it
    #[allow(dead_code)]
    identifier: Option<Atom<'a>>,
    entropy: f32,
}

/// A secret that was positively identified by a secret rule.
///
/// This gets used to construct the final diagnostic message.
#[derive(Debug, Clone)]
pub struct SecretViolation<'a> {
    // NOTE: Rules get a &mut reference to a SecretViolation to verify the
    // violation. It is important that the underlying secret is not modified.
    secret: Secret<'a>,
    rule_name: Cow<'a, str>, // really should be &'static
    message: Cow<'a, str>,   // really should be &'static
}

// SAFETY: 8 is a valid value for NonZeroU32
pub(super) const DEFAULT_MIN_LEN: NonZeroU32 = unsafe { NonZeroU32::new_unchecked(8) };
pub(super) const DEFAULT_MIN_ENTROPY: f32 = 0.5;

/// Metadata trait separated out of [`SecretScanner`]. The easiest way to implement this is with
/// the [`oxc_macros::declare_oxc_secret!`] macro.
pub trait SecretScannerMeta {
    /// Human-readable unique identifier describing what service this rule finds api keys for.
    /// Must be kebab-case.
    fn rule_name(&self) -> Cow<'static, str>;

    fn message(&self) -> Cow<'static, str>;

    /// Min str length a key candidate must have to be considered a violation. Must be >= 1.
    #[inline]
    fn min_len(&self) -> NonZeroU32 {
        DEFAULT_MIN_LEN
    }

    /// Secret candidates above this length will not be considered.
    ///
    /// By default, no maximum length is enforced.
    #[inline]
    fn max_len(&self) -> Option<NonZeroU32> {
        None
    }

    /// Min entropy a key must have to be considered a violation. Must be >= 0.
    ///
    /// Defaults to 0.5
    #[inline]
    fn min_entropy(&self) -> f32 {
        DEFAULT_MIN_ENTROPY
    }
}

/// Detects hard-coded API keys and other credentials of a single kind or for a single SaaS
/// service.
pub trait SecretScanner: SecretScannerMeta {
    /// Returns `true` if `candidate` is a leaked credential.
    fn detect(&self, candidate: &Secret<'_>) -> bool;

    /// `verify` lets secret rules modify diagnostic messages and/or perform additional
    /// verification checks on secrets before they are reported. You may mutate state such as the
    /// diagnostic message, but you _must not_ modify the secret itself as it is shared between
    /// all rules.
    ///
    /// Returns `true` to report the violation, or `false` to ignore it.
    #[inline]
    fn verify(&self, violation: &mut SecretViolation<'_>) -> bool {
        true
    }
}

impl<'a> Secret<'a> {
    pub fn new(secret: &'a str, span: Span, identifier: Option<Atom<'a>>) -> Self {
        let entropy = secret.entropy();
        Self { secret, span, identifier, entropy }
    }
}
impl Deref for Secret<'_> {
    type Target = str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.secret
    }
}

impl Entropy for Secret<'_> {
    #[inline]
    fn entropy(&self) -> f32 {
        self.entropy
    }
}

impl GetSpan for Secret<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> SecretViolation<'a> {
    pub fn new(secret: Secret<'a>, rule: &SecretsEnum) -> Self {
        Self { secret, rule_name: rule.rule_name(), message: rule.message() }
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn set_message<S: Into<Cow<'static, str>>>(&mut self, message: S) {
        self.message = message.into();
    }

    pub fn rule_name(&self) -> &str {
        &self.rule_name
    }
}

impl GetSpan for SecretViolation<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.secret.span()
    }
}
impl Deref for SecretViolation<'_> {
    type Target = str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.secret
    }
}
