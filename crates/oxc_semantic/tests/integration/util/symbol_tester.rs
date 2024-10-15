use std::rc::Rc;

use oxc_diagnostics::{Error, OxcDiagnostic};
use oxc_semantic::{Reference, ScopeFlags, ScopeId, Semantic, SymbolFlags, SymbolId};
use oxc_span::CompactStr;

use super::{Expect, SemanticTester};

/// Test a symbol in the [`Semantic`] analysis results.
///
/// To use this, chain together assertions about the symbol, such as
/// [`SymbolTester::contains_flags`], [`SymbolTester::has_number_of_reads`],
/// etc., then finish the chain off with a call to [`SymbolTester::test`].
///
/// You will never create this struct manually. Instead, use one of
/// [`SemanticTester`]'s factories, such as [`SemanticTester::has_root_symbol`].
///
/// # Example
/// ```
/// use oxc_semantic::{SymbolFlags, Semantic};
/// use super::SemanticTester;
///
/// #[test]
/// fn my_test() {
///   SemanticTester::js("let x = 0; let foo = (0, x++)")
///     .has_some_symbol("x")                  // find a symbol named "x" at any scope
///     .contains_flags(SymbolFlags::Variable) // check that it's a variable
///     .has_number_of_reads(1)                // check read references
///     .has_number_of_writes(1)               // check write references
///     .test();                               // finish the test. Will panic if any assertions failed.
/// }
///```
#[must_use]
pub struct SymbolTester<'a> {
    parent: &'a SemanticTester<'a>,
    /// Reference to semantic analysis results, from [`SemanticTester`]
    semantic: Rc<Semantic<'a>>,
    /// Name of the subject symbol
    target_symbol_name: String,
    /// Symbol data, or error if not found
    test_result: Result<SymbolId, OxcDiagnostic>,
}

impl<'a> SymbolTester<'a> {
    pub(super) fn new_at_root(
        parent: &'a SemanticTester,
        semantic: Semantic<'a>,
        target: &str,
    ) -> Self {
        let decl = semantic.scopes().get_binding(semantic.scopes().root_scope_id(), target);
        let data = decl.map_or_else(
            || Err(OxcDiagnostic::error(format!("Could not find declaration for {target}"))),
            Ok,
        );

        SymbolTester {
            parent,
            semantic: Rc::new(semantic),
            target_symbol_name: target.to_string(),
            test_result: data,
        }
    }

    pub(super) fn new_unique(
        parent: &'a SemanticTester,
        semantic: Semantic<'a>,
        target: &str,
    ) -> Self {
        let symbols_with_target_name: Vec<_> = semantic
            .scopes()
            .iter_bindings()
            .filter(|(_, _, name)| name.as_str() == target)
            .collect();
        let data = match symbols_with_target_name.len() {
            0 => Err(OxcDiagnostic::error(format!("Could not find declaration for {target}"))),
            1 => Ok(symbols_with_target_name
                .iter()
                .map(|&(_, symbol_id, _)| symbol_id)
                .next()
                .unwrap()),
            n if n > 1 => Err(OxcDiagnostic::error(format!(
                "Couldn't uniquely resolve symbol id for target {target}; {n} symbols with that name are declared in the source."
            ))),
            _ => unreachable!(),
        };

        SymbolTester {
            parent,
            semantic: Rc::new(semantic),
            target_symbol_name: target.to_string(),
            test_result: data,
        }
    }

    /// Get inner resources without consuming `self`
    pub fn inner(&self) -> (Rc<Semantic<'a>>, SymbolId) {
        (Rc::clone(&self.semantic), *self.test_result.as_ref().unwrap())
    }

    pub(super) fn new_first_binding(
        parent: &'a SemanticTester,
        semantic: Semantic<'a>,
        target: &str,
    ) -> Self {
        let symbols_with_target_name: Option<(ScopeId, SymbolId, &CompactStr)> =
            semantic.scopes().iter_bindings().find(|(_, _, name)| name.as_str() == target);

        let data = match symbols_with_target_name {
            Some((_, symbol_id, _)) => Ok(symbol_id),
            None => Err(OxcDiagnostic::error(format!("Could not find declaration for {target}"))),
        };

        SymbolTester {
            parent,
            semantic: Rc::new(semantic),
            target_symbol_name: target.to_string(),
            test_result: data,
        }
    }

    /// Checks if the resolved symbol contains all flags in `flags`, using [`SymbolFlags::contains()`]
    pub fn contains_flags(mut self, flags: SymbolFlags) -> Self {
        self.test_result = match self.test_result {
            Ok(symbol_id) => {
                let found_flags = self.semantic.symbols().get_flags(symbol_id);
                if found_flags.contains(flags) {
                    Ok(symbol_id)
                } else {
                    Err(OxcDiagnostic::error(format!(
                        "Expected {} to contain flags {:?}, but it had {:?}",
                        self.target_symbol_name, flags, found_flags
                    )))
                }
            }
            err => err,
        };
        self
    }

    pub fn intersects_flags(mut self, flags: SymbolFlags) -> Self {
        self.test_result = match self.test_result {
            Ok(symbol_id) => {
                let found_flags = self.semantic.symbols().get_flags(symbol_id);
                if found_flags.intersects(flags) {
                    Ok(symbol_id)
                } else {
                    Err(OxcDiagnostic::error(format!(
                        "Expected {} to intersect with flags {:?}, but it had {:?}",
                        self.target_symbol_name, flags, found_flags
                    )))
                }
            }
            err => err,
        };
        self
    }

    /// Check that this symbol has a certain number of read [`Reference`]s
    ///
    /// References that are both read and write are counted.
    pub fn has_number_of_reads(self, ref_count: usize) -> Self {
        self.has_number_of_references_where(ref_count, Reference::is_read)
    }

    /// Check that this symbol has a certain number of write [`Reference`]s.
    ///
    /// References that are both read and write are counted.
    pub fn has_number_of_writes(self, ref_count: usize) -> Self {
        self.has_number_of_references_where(ref_count, Reference::is_write)
    }

    /// Check that this symbol has a certain number of [`Reference`]s of any kind.
    pub fn has_number_of_references(self, ref_count: usize) -> Self {
        self.has_number_of_references_where(ref_count, |_| true)
    }

    /// Check that this symbol has a certain number of [`Reference`]s that meet
    /// some criteria established by a predicate.
    pub fn has_number_of_references_where<F>(mut self, ref_count: usize, filter: F) -> Self
    where
        F: FnMut(&Reference) -> bool,
    {
        self.test_result = match self.test_result {
            Ok(symbol_id) => {
                let refs = {
                    self.semantic.symbols().get_resolved_reference_ids(symbol_id).iter().map(
                        |&reference_id| self.semantic.symbols().get_reference(reference_id).clone(),
                    )
                };
                let num_accepted = refs.filter(filter).count();
                if num_accepted == ref_count {
                    Ok(symbol_id)
                } else {
                    Err(OxcDiagnostic::error(format!(
                        "Expected to find {ref_count} acceptable references, but only found {num_accepted}"
                    )))
                }
            }
            e => e,
        };
        self
    }

    /// Check that this symbol is exported.
    ///
    /// Export status is checked using the symbol's [`SymbolFlags`], not by
    /// checking the [`oxc_semantic::ModuleRecord`].
    ///
    /// For the inverse of this assertion, use [`SymbolTester::is_not_exported`].
    #[allow(clippy::wrong_self_convention)]
    pub fn is_exported(mut self) -> Self {
        self.test_result = match self.test_result {
            Ok(symbol_id) => {
                let binding = self.target_symbol_name.clone();
                if self.semantic.symbols().get_flags(symbol_id).is_export() {
                    Ok(symbol_id)
                } else {
                    Err(OxcDiagnostic::error(format!(
                        "Expected {binding} to be exported with SymbolFlags::Export"
                    )))
                }
            }
            e => e,
        };
        self
    }

    /// Check that this symbol is not exported.
    ///
    /// Export status is checked using the symbol's [`SymbolFlags`], not by
    /// checking the [`oxc_semantic::ModuleRecord`].
    ///
    /// For the inverse of this assertion, use [`SymbolTester::is_exported`].
    #[allow(clippy::wrong_self_convention)]
    pub fn is_not_exported(mut self) -> Self {
        self.test_result = match self.test_result {
            Ok(symbol_id) => {
                let binding = self.target_symbol_name.clone();
                if self.semantic.symbols().get_flags(symbol_id).contains(SymbolFlags::Export) {
                    Err(OxcDiagnostic::error(format!(
                        "Expected {binding} to not be exported. Symbol has export flag."
                    )))
                } else {
                    Ok(symbol_id)
                }
            }
            e => e,
        };
        self
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn is_in_scope(mut self, expected_flags: ScopeFlags) -> Self {
        let target_name: &str = self.target_symbol_name.as_ref();
        self.test_result = match self.test_result {
            Ok(symbol_id) => {
                let scope_id = self.semantic.symbol_scope(symbol_id);
                let scope_flags = self.semantic.scopes().get_flags(scope_id);
                if scope_flags.contains(expected_flags) {
                    Ok(symbol_id)
                } else {
                    Err(OxcDiagnostic::error(format!(
                        "Binding {target_name} is not in a scope with expected flags.\n\tExpected: {expected_flags:?}\n\tActual: {scope_flags:?}"
                    )))
                }
            }
            e => e,
        };
        self
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn is_not_in_scope(mut self, excluded_flags: ScopeFlags) -> Self {
        let target_name: &str = self.target_symbol_name.as_ref();
        self.test_result = match self.test_result {
            Ok(symbol_id) => {
                let scope_id = self.semantic.symbol_scope(symbol_id);
                let scope_flags = self.semantic.scopes().get_flags(scope_id);
                if scope_flags.contains(excluded_flags) {
                    Err(OxcDiagnostic::error(format!(
                        "Binding {target_name} is in a scope with excluded flags.\n\tExpected: not {excluded_flags:?}\n\tActual: {scope_flags:?}"
                    )))
                } else {
                    Ok(symbol_id)
                }
            }
            e => e,
        };
        self
    }

    /// Complete the test case. Will panic if any of the previously applied
    /// assertions failed.
    pub fn test(self) {
        let res: Result<_, _> = self.into();

        res.unwrap();
    }
}

impl<'a> Expect<(Rc<Semantic<'a>>, SymbolId), bool> for SymbolTester<'a> {
    fn expect<'e, F>(self, expectation: F) -> Self
    where
        F: FnOnce((Rc<Semantic<'a>>, SymbolId)) -> bool,
    {
        let Ok(symbol_id) = self.test_result else { return self };
        let did_pass = expectation((Rc::clone(&self.semantic), symbol_id));
        if did_pass {
            self
        } else {
            Self { test_result: Err(OxcDiagnostic::error("Expectation failed")), ..self }
        }
    }
}

impl<'a> Expect<(Rc<Semantic<'a>>, SymbolId), Result<(), &'static str>> for SymbolTester<'a> {
    fn expect<'e, F>(self, expectation: F) -> Self
    where
        F: FnOnce((Rc<Semantic<'a>>, SymbolId)) -> Result<(), &'static str>,
    {
        let Ok(symbol_id) = self.test_result else { return self };
        let did_pass = expectation((Rc::clone(&self.semantic), symbol_id));
        if let Err(e) = did_pass {
            Self { test_result: Err(OxcDiagnostic::error(e)), ..self }
        } else {
            self
        }
    }
}
impl<'a> Expect<(Rc<Semantic<'a>>, SymbolId), Result<(), OxcDiagnostic>> for SymbolTester<'a> {
    fn expect<'e, F>(self, expectation: F) -> Self
    where
        F: FnOnce((Rc<Semantic<'a>>, SymbolId)) -> Result<(), OxcDiagnostic>,
    {
        let Ok(symbol_id) = self.test_result else { return self };
        let did_pass = expectation((Rc::clone(&self.semantic), symbol_id));
        if let Err(e) = did_pass {
            Self { test_result: Err(e), ..self }
        } else {
            self
        }
    }
}

impl<'a> From<SymbolTester<'a>> for Result<(), Error> {
    fn from(val: SymbolTester<'a>) -> Self {
        let source_code = val.parent.source_text.to_string();
        val.test_result.map(|_| {}).map_err(|e| e.with_source_code(source_code))
    }
}
