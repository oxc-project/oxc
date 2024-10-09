import { camelToSnake } from './utils.mjs';

/**
 * @param {import('./parse.mjs').Types} types
 */
export default function generateScopesCollectorCode(types) {
  let methods = '';
  for (const type of Object.values(types)) {
    if (type.kind === 'enum' || !type.scopeArgs) continue;

    const extraParams = type.scopeArgs.flags === 'flags' ? ', _flags: ScopeFlags' : '';
    methods += `
      #[inline]
      fn visit_${camelToSnake(type.name)}(&mut self, it: &${type.rawName}${extraParams}) {
        self.add_scope(&it.scope_id);
      }
    `;
  }

  return `
    use std::cell::Cell;

    #[allow(clippy::wildcard_imports)]
    use oxc_ast::{ast::*, visit::Visit};
    use oxc_syntax::scope::{ScopeFlags, ScopeId};

    /// Visitor that locates all child scopes.
    /// NB: Child scopes only, not grandchild scopes.
    /// Does not do full traversal - stops each time it hits a node with a scope.
    pub(crate) struct ChildScopeCollector {
      pub(crate) scope_ids: Vec<ScopeId>,
    }

    impl ChildScopeCollector {
      pub(crate) fn new() -> Self {
        Self { scope_ids: vec![] }
      }

      pub(crate) fn add_scope(&mut self, scope_id: &Cell<Option<ScopeId>>) {
        self.scope_ids.push(scope_id.get().unwrap());
      }
    }

    impl<'a> Visit<'a> for ChildScopeCollector {
      ${methods}

      #[inline]
      fn visit_finally_clause(&mut self, it: &BlockStatement<'a>) {
        self.add_scope(&it.scope_id);
      }
    }
  `;
}
