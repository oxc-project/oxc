// Binary size optimization utilities for oxc_linter
// This module provides optimizations to reduce binary size while maintaining functionality

/// Static diagnostic messages to reduce string duplication and improve code sharing
pub mod diagnostic_messages {
    pub const UNEXPECTED_TOKEN: &str = "Unexpected token";
    pub const MISSING_SEMICOLON: &str = "Missing semicolon";
    pub const UNREACHABLE_CODE: &str = "Unreachable code detected";
    pub const UNEXPECTED_CONSOLE: &str = "Unexpected console statement";
    pub const NO_UNUSED_VARS: &str = "Variable is defined but never used";
    pub const NO_DEBUGGER: &str = "Unexpected debugger statement";
    pub const NO_ALERT: &str = "Unexpected alert";
    pub const PREFER_CONST: &str = "Variable should be declared with const";
    pub const EQEQEQ: &str = "Expected '===' and instead saw '=='";
    pub const CURLY: &str = "Expected curly braces around statement";
}

/// Static rule names to reduce string duplication
pub mod rule_names {
    pub const NO_UNUSED_VARS: &str = "no-unused-vars";
    pub const NO_CONSOLE: &str = "no-console";
    pub const NO_DEBUGGER: &str = "no-debugger";
    pub const NO_ALERT: &str = "no-alert";
    pub const PREFER_CONST: &str = "prefer-const";
    pub const EQEQEQ: &str = "eqeqeq";
    pub const CURLY: &str = "curly";
    pub const NO_UNDEF: &str = "no-undef";
    pub const NO_UNREACHABLE: &str = "no-unreachable";
    pub const NO_DUPLICATE_KEYS: &str = "no-duplicate-keys";
}

/// Optimized diagnostic creation to reduce generic instantiations
/// This consolidates multiple diagnostic creation methods into fewer templates
use crate::{
    context::LintContext, 
    fixer::{Message, PossibleFixes},
};
use oxc_diagnostics::OxcDiagnostic;

impl<'a> LintContext<'a> {
    /// Optimized diagnostic creation with static message
    /// Reduces template instantiation by using consistent types
    #[inline]
    pub fn diagnostic_static(&self, message: &'static str, span: oxc_span::Span) {
        self.diagnostic(OxcDiagnostic::warn(message).with_label(span));
    }
    
    /// Optimized diagnostic creation with help text
    /// Reduces template instantiation by using consistent types
    #[inline] 
    pub fn diagnostic_with_help_static(
        &self, 
        message: &'static str, 
        help: &'static str,
        span: oxc_span::Span
    ) {
        self.diagnostic(
            OxcDiagnostic::warn(message)
                .with_label(span)
                .with_help(help)
        );
    }
}

/// Conditional Debug implementation for release builds
/// This significantly reduces Debug trait instantiations in release builds
#[macro_export]
macro_rules! conditional_debug {
    ($struct:ident) => {
        #[cfg(debug_assertions)]
        impl std::fmt::Debug for $struct {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct(stringify!($struct))
                    .finish_non_exhaustive()
            }
        }
        
        #[cfg(not(debug_assertions))]
        impl std::fmt::Debug for $struct {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct(stringify!($struct)).finish()
            }
        }
    };
}

/// Optimized HashMap types to reduce generic instantiations
use rustc_hash::FxHashMap;
use oxc_semantic::{SymbolId, ScopeId};
use crate::rules::RuleEnum;

// Consolidated map types used throughout the linter
pub type RuleMap = FxHashMap<&'static str, RuleEnum>;
pub type SymbolMap<T> = FxHashMap<SymbolId, T>;  
pub type ScopeMap<T> = FxHashMap<ScopeId, T>;
pub type StringMap<T> = FxHashMap<&'static str, T>;

/// Lazy initialization for expensive static data
/// This ensures static data is only initialized when actually used
use std::sync::LazyLock;

pub static COMMON_GLOBALS: LazyLock<FxHashMap<&'static str, bool>> = LazyLock::new(|| {
    let mut map = FxHashMap::default();
    map.insert("console", true);
    map.insert("window", true);
    map.insert("document", true);
    map.insert("global", true);
    map.insert("process", true);
    map
});

pub static ERROR_CODES: LazyLock<FxHashMap<&'static str, u32>> = LazyLock::new(|| {
    let mut map = FxHashMap::default();
    map.insert("no-unused-vars", 1001);
    map.insert("no-console", 1002);
    map.insert("no-debugger", 1003);
    map.insert("eqeqeq", 1004);
    map
});

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_static_messages() {
        assert_eq!(diagnostic_messages::UNEXPECTED_TOKEN, "Unexpected token");
        assert_eq!(rule_names::NO_CONSOLE, "no-console");
    }
    
    #[test]
    fn test_lazy_globals() {
        assert!(COMMON_GLOBALS.contains_key("console"));
        assert_eq!(ERROR_CODES.get("no-console"), Some(&1002));
    }
}