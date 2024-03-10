use std::rc::Rc;

use oxc_diagnostics::{miette::miette, Error};
use oxc_semantic::{Reference, ScopeFlags, Semantic, SymbolFlags, SymbolId};
use oxc_span::Atom;

use super::{Expect, SemanticTester};

pub struct SymbolTester<'a> {
    parent: &'a SemanticTester<'a>,
    /// Reference to semantic analysis results, from [`SemanticTester`]
    semantic: Rc<Semantic<'a>>,
    /// Name of the subject symbol
    target_symbol_name: String,
    /// Symbol data, or error if not found
    test_result: Result<SymbolId, oxc_diagnostics::Error>,
}

impl<'a> SymbolTester<'a> {
    pub(super) fn new_at_root(
        parent: &'a SemanticTester,
        semantic: Semantic<'a>,
        target: &str,
    ) -> Self {
        let decl =
            semantic.scopes().get_binding(semantic.scopes().root_scope_id(), &Atom::from(target));
        let data = decl.map_or_else(|| Err(miette!("Could not find declaration for {target}")), Ok);

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
            0 => Err(miette!("Could not find declaration for {target}")),
            1 => Ok(symbols_with_target_name.iter().map(|(_, symbol_id, _)| *symbol_id).next().unwrap()),
            n if n > 1 => Err(miette!("Couldn't uniquely resolve symbol id for target {target}; {n} symbols with that name are declared in the source.")),
            _ => unreachable!()
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
                let found_flags = self.semantic.symbols().get_flag(symbol_id);
                if found_flags.contains(flags) {
                    Ok(symbol_id)
                } else {
                    Err(miette!(
                        "Expected {} to contain flags {:?}, but it had {:?}",
                        self.target_symbol_name,
                        flags,
                        found_flags
                    ))
                }
            }
            err => err,
        };
        self
    }

    pub fn intersects_flags(mut self, flags: SymbolFlags) -> Self {
        self.test_result = match self.test_result {
            Ok(symbol_id) => {
                let found_flags = self.semantic.symbols().get_flag(symbol_id);
                if found_flags.intersects(flags) {
                    Ok(symbol_id)
                } else {
                    Err(miette!(
                        "Expected {} to intersect with flags {:?}, but it had {:?}",
                        self.target_symbol_name,
                        flags,
                        found_flags
                    ))
                }
            }
            err => err,
        };
        self
    }

    pub fn has_number_of_reads(self, ref_count: usize) -> Self {
        self.has_number_of_references_where(ref_count, Reference::is_read)
    }

    pub fn has_number_of_writes(self, ref_count: usize) -> Self {
        self.has_number_of_references_where(ref_count, Reference::is_write)
    }

    pub fn has_number_of_references(self, ref_count: usize) -> Self {
        self.has_number_of_references_where(ref_count, |_| true)
    }

    pub fn has_number_of_references_where<F>(mut self, ref_count: usize, filter: F) -> Self
    where
        F: FnMut(&Reference) -> bool,
    {
        self.test_result = match self.test_result {
            Ok(symbol_id) => {
                let refs = {
                    self.semantic
                        .symbols()
                        .get_resolved_reference_ids(symbol_id)
                        .iter()
                        .map(|r_id| self.semantic.symbols().get_reference(*r_id).clone())
                };
                let num_accepted = refs.filter(filter).count();
                if num_accepted == ref_count {
                    Ok(symbol_id)
                } else {
                    Err(miette!("Expected to find {ref_count} acceptable references, but only found {num_accepted}"))
                }
            }
            e => e,
        };
        self
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn is_exported(mut self) -> Self {
        self.test_result = match self.test_result {
            Ok(symbol_id) => {
                let binding = self.target_symbol_name.clone();
                if self.semantic.module_record().exported_bindings.contains_key(binding.as_str())
                    && self.semantic.scopes().get_root_binding(&binding) == Some(symbol_id)
                {
                    Ok(symbol_id)
                } else {
                    Err(miette!("Expected {binding} to be exported."))
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
                    Err(miette!("Binding {target_name} is not in a scope with expected flags.\n\tExpected: {expected_flags:?}\n\tActual: {scope_flags:?}"))
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
                    Err(miette!("Binding {target_name} is in a scope with excluded flags.\n\tExpected: not {excluded_flags:?}\n\tActual: {scope_flags:?}"))
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
            Self { test_result: Err(miette!("Expectation failed")), ..self }
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
            Self { test_result: Err(miette!(e)), ..self }
        } else {
            self
        }
    }
}
impl<'a> Expect<(Rc<Semantic<'a>>, SymbolId), Result<(), Error>> for SymbolTester<'a> {
    fn expect<'e, F>(self, expectation: F) -> Self
    where
        F: FnOnce((Rc<Semantic<'a>>, SymbolId)) -> Result<(), Error>,
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
