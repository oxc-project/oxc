use oxc_ast::ast::{BlockStatement, FunctionBody, Statement, SwitchCase};
use oxc_ecmascript::ToBoolean;

/// `StatementReturnStatus` describes whether the CFG corresponding to
/// the statement is termitated by return statement in all/some/nome of
/// its exit blocks.
///
/// For example, an "if" statement is terminated by explicit return if and only if either:
/// 1. the test is always true and the consequent is terminated by explicit return
/// 2. the test is always false and the alternate is terminated by explicit return
/// 3. both the consequent and the alternate is terminated by explicit return
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum StatementReturnStatus {
    /// Only explicit return on all paths
    AlwaysExplicit,
    /// Only implicit return on all paths
    AlwaysImplicit,
    /// Explicit or implicit return on all paths (no un-returned paths)
    AlwaysMixed,

    /// Only explicit return on some paths
    SomeExplicit,
    /// Only implicit return on some paths
    SomeImplicit,
    /// Explicit and implicit return on some paths
    SomeMixed,

    /// No return on all paths
    NotReturn,
}

impl StatementReturnStatus {
    /// Join the status of two branches. Similar to a logical *and* operation.
    ///
    /// E.g.,
    /// if (test) {
    ///   return a // `AlwaysExplicit`
    /// } else {
    ///   var a = 0; // `NotReturn`
    /// }
    ///
    /// will produce `SomeExplicit` for the whole if statement
    pub fn join(self, rhs: Self) -> Self {
        let must_return = self.must_return() && rhs.must_return();
        let explicit = self.may_return_explicit() || rhs.may_return_explicit();
        let implicit = self.may_return_implicit() || rhs.may_return_implicit();

        Self::create(must_return, explicit, implicit)
    }

    /// Union the status of two sequential statements. Similar to a logical *or* operation.
    ///
    /// E.g.,
    /// {
    ///   if (test) {
    ///     return a;
    ///   } // `SomeExplicit`
    ///
    ///   return // `AlwaysImplicit`
    /// }
    ///
    /// will produce `AlwaysMixed` for the block statement.
    pub fn union(self, rhs: Self) -> Self {
        let must_return = self.must_return() || rhs.must_return();
        let explicit = self.may_return_explicit() || rhs.may_return_explicit();
        let implicit = self.may_return_implicit() || rhs.may_return_implicit();

        Self::create(must_return, explicit, implicit)
    }

    fn create(must_return: bool, maybe_explicit: bool, maybe_implicit: bool) -> Self {
        match (must_return, maybe_explicit, maybe_implicit) {
            (true, true, true) => Self::AlwaysMixed,
            (true, true, false) => Self::AlwaysExplicit,
            (true, false, true) => Self::AlwaysImplicit,
            (false, true, true) => Self::SomeMixed,
            (false, true, false) => Self::SomeExplicit,
            (false, false, true) => Self::SomeImplicit,
            (false, false, false) => Self::NotReturn,
            (true, false, false) => unreachable!(),
        }
    }

    pub fn must_return(self) -> bool {
        matches!(self, Self::AlwaysExplicit | Self::AlwaysImplicit | Self::AlwaysMixed)
    }

    pub fn may_return_explicit(self) -> bool {
        matches!(
            self,
            Self::AlwaysExplicit | Self::AlwaysMixed | Self::SomeExplicit | Self::SomeMixed
        )
    }

    pub fn may_return_implicit(self) -> bool {
        matches!(
            self,
            Self::AlwaysImplicit | Self::AlwaysMixed | Self::SomeImplicit | Self::SomeMixed
        )
    }
}

pub fn check_function_body(function: &FunctionBody) -> StatementReturnStatus {
    // function body can be viewed as a block statement, but we don't
    // short-circuit to catch all the possible returns.
    // E.g.
    // foo.map(() => {
    //  return 1;
    //  return; // Error
    // })
    let mut status = StatementReturnStatus::NotReturn;
    for stmt in &function.statements {
        status = status.union(check_statement(stmt));
    }

    status
}

/// Return checkers runs a Control Flow-like Analysis on a statement to see if it
/// always returns on all paths of execution.
pub fn check_statement(statement: &Statement) -> StatementReturnStatus {
    match statement {
        Statement::ReturnStatement(ret) => {
            if ret.argument.is_some() {
                StatementReturnStatus::AlwaysExplicit
            } else {
                StatementReturnStatus::AlwaysImplicit
            }
        }

        Statement::IfStatement(stmt) => {
            let test = &stmt.test;
            let left = check_statement(&stmt.consequent);
            let right =
                stmt.alternate.as_ref().map_or(StatementReturnStatus::NotReturn, check_statement);

            test.to_boolean().map_or_else(|| left.join(right), |val| if val { left } else { right })
        }

        Statement::WhileStatement(stmt) => {
            let test = &stmt.test;
            let inner_return = check_statement(&stmt.body);
            if test.to_boolean() == Some(true) {
                inner_return
            } else {
                inner_return.join(StatementReturnStatus::NotReturn)
            }
        }

        // do while loop always executes at least once
        Statement::DoWhileStatement(stmt) => check_statement(&stmt.body),

        // A switch statement always return if:
        // 1. Every branch that eventually breaks out of the switch breaks via return
        // 2. There is a default case that returns
        Statement::SwitchStatement(stmt) => {
            let mut case_statuses = vec![];
            let mut default_case_status = StatementReturnStatus::NotReturn;

            let mut current_case_status = StatementReturnStatus::NotReturn;
            for case in &stmt.cases {
                let branch_terminated = check_switch_case(case, &mut current_case_status);

                if case.is_default_case() {
                    default_case_status = current_case_status;
                    // Cases below the default case are not considered.
                    break;
                } else if branch_terminated {
                    case_statuses.push(current_case_status);
                    current_case_status = StatementReturnStatus::NotReturn;
                } // Falls through to next case, accumulating lattice
            }

            case_statuses.iter().fold(default_case_status, |accum, &lattice| accum.join(lattice))
        }

        Statement::BlockStatement(stmt) => check_block_statement(stmt),

        Statement::LabeledStatement(stmt) => check_statement(&stmt.body),

        Statement::WithStatement(stmt) => check_statement(&stmt.body),

        Statement::TryStatement(stmt) => {
            let mut status = check_block_statement(&stmt.block);
            if let Some(catch) = &stmt.handler {
                status = status.join(check_block_statement(&catch.body));
            }
            if let Some(finally) = &stmt.finalizer {
                status = status.union(check_block_statement(finally));
            }
            status
        }

        _ => StatementReturnStatus::NotReturn,
    }
}

/// Checks whether this switch case falls in:
/// 1. always return explicitly
/// 2. always return at least implicitly
/// 3. might not return and break out of the switch
/// 4. might not return and fall through
pub fn check_switch_case(
    case: &SwitchCase,
    accum: &mut StatementReturnStatus, /* Lattice accumulated from previous branches */
) -> bool {
    for s in &case.consequent {
        // This case is over
        if let Statement::BreakStatement(_) = s {
            return true;
        }

        let status = check_statement(s);
        *accum = accum.union(status);

        if accum.must_return() {
            return true;
        }
    }

    // This branch does not either return or break. Fall through
    false
}

pub fn check_block_statement(block: &BlockStatement) -> StatementReturnStatus {
    let mut all_statements_status = StatementReturnStatus::NotReturn;

    for s in &block.body {
        // The only case where we can see break is if the block is inside a loop,
        // which means the loop does not return
        if let Statement::BreakStatement(_) = s {
            break;
        }

        let current_stmt_status = check_statement(s);
        all_statements_status = all_statements_status.union(current_stmt_status);
        if all_statements_status.must_return() {
            break;
        }
    }

    all_statements_status
}

#[cfg(test)]
mod tests {
    use oxc_allocator::Allocator;
    use oxc_ast::ast::Program;
    use oxc_parser::Parser;
    use oxc_span::SourceType;

    use super::*;

    fn parse_statement_and_test(source: &'static str, expected: StatementReturnStatus) {
        let source_type = SourceType::default();
        let alloc = Allocator::default();
        let parser = Parser::new(&alloc, source, source_type);
        let ret = parser.parse();
        assert!(ret.errors.is_empty());

        // The program is a function declaration with a single statement
        let program = ret.program;
        let Program { body, .. } = program;
        let stmt = body.first().unwrap();
        let Statement::FunctionDeclaration(func) = stmt else { unreachable!() };

        let first_statement = &func.body.as_ref().unwrap().statements[0];

        test_match_expected(first_statement, expected);
    }

    fn test_match_expected(statement: &Statement, expected: StatementReturnStatus) {
        let actual = check_statement(statement);

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_switch_always_explicit() {
        // Return Explicit
        let always_explicit = r#"
    function d() {
      switch (a) {
        case "C":
          switch (b) {
            case "A":
              var a = 1;
            default:
              return 123;
          }
        default:
          return 1;
      }
    }
  "#;
        parse_statement_and_test(always_explicit, StatementReturnStatus::AlwaysExplicit);
    }

    #[test]
    fn test_switch_always_implicit() {
        let always_implicit = r#"
    function d() {
      switch (a) {
        case "C":
          switch (b) {
            case "A":
              var a = 1;
            default:
              return;
          }
        default:
          return;
      }
    }
    "#;
        parse_statement_and_test(always_implicit, StatementReturnStatus::AlwaysImplicit);
    }

    #[test]
    fn test_switch_always_mixed() {
        let always_mixed = r#"
        function d() {
          switch (a) {
            case "C":
              switch (b) {
                case "A":
                  var a = 1;
                default:
                  return 123;
              }
            default:
              return;
          }
        }
        "#;
        parse_statement_and_test(always_mixed, StatementReturnStatus::AlwaysMixed);
    }

    #[test]
    fn test_switch_some_mixed() {
        let source = r#"
      function foo() {
        switch (a) {
          case "C":
            return 1;
          case "B":
            return;
        }
      }
    "#;

        parse_statement_and_test(source, StatementReturnStatus::SomeMixed);
    }

    #[test]
    fn test_switch_some_explicit() {
        let source = r#"
      function foo() {
        switch (a) {
          case "C":
            return 1;
        }
      }
    "#;

        parse_statement_and_test(source, StatementReturnStatus::SomeExplicit);
    }

    #[test]
    fn test_if_always_true() {
        let always_true = r"
      function foo() {
        if (true) return 1;
        else {
          var a = 123;
          console.log(a);
        }
      }
    ";
        parse_statement_and_test(always_true, StatementReturnStatus::AlwaysExplicit);
    }

    #[test]
    fn test_if_always_false() {
        let always_false = r"
        function foo() {
          if (false) {
            var a = 123;
          } else {
            return 123;
          }
        }
      ";
        parse_statement_and_test(always_false, StatementReturnStatus::AlwaysExplicit);
    }

    #[test]
    fn test_if_non_static() {
        let non_static = r"
        function foo() {
          if (a) {
            return 123;
          } else {
            var c = 0;
          }
        }
      ";
        parse_statement_and_test(non_static, StatementReturnStatus::SomeExplicit);
    }

    #[test]
    fn test_block() {
        // The block statement could: return a, return, or does not return in the end
        let source = r"
        function foo() {
          {
            if (a) {
              return a;
            }

            if (b) {
              return;
            }
          }
        }
      ";

        parse_statement_and_test(source, StatementReturnStatus::SomeMixed);
    }

    #[test]
    fn test_while_true() {
        let source = "
        function foo() {
          while (true) {
            return;
          }
        }
      ";
        parse_statement_and_test(source, StatementReturnStatus::AlwaysImplicit);
    }
}
