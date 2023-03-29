use oxc_ast::ast::{BlockStatement, Statement, SwitchCase};

/// Lattice structure describe the behavior of a statement with respect to
/// returning from the function
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum StatementReturnLattice {
    // Always return
    AlwaysExplicit,
    AlwaysImplicit,
    AlwaysMixed,

    // Return on some paths
    SomeExplicit,
    SomeImplicit,
    SomeMixed,

    // Return on no path
    NotReturn,
}

impl StatementReturnLattice {
    pub fn join(self, rhs: Self) -> Self {
        let must_return = self.must_return() && rhs.must_return();
        let explicit = self.may_return_explicit() || rhs.may_return_explicit();
        let implicit = self.may_return_implicit() || rhs.may_return_implicit();

        Self::create(must_return, explicit, implicit)
    }

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

/// Return checkers runs a Control Flow-like Analysis on a statement to see if it
/// always returns on all paths of execution.
pub fn check_statement(statement: &Statement) -> StatementReturnLattice {
    match statement {
        Statement::ReturnStatement(ret) => {
            if ret.argument.is_some() {
                StatementReturnLattice::AlwaysExplicit
            } else {
                StatementReturnLattice::AlwaysImplicit
            }
        }

        Statement::IfStatement(stmt) => {
            let test = &stmt.test;
            let left = check_statement(&stmt.consequent);
            let right =
                stmt.alternate.as_ref().map_or(StatementReturnLattice::NotReturn, check_statement);

            test.get_boolean_value()
                .map_or_else(|| left.join(right), |val| if val { left } else { right })
        }

        Statement::WhileStatement(stmt) => {
            let test = &stmt.test;
            let inner_return = check_statement(&stmt.body);
            if let Some(val) = test.get_boolean_value() && val {
                    inner_return
                } else {
                    inner_return.join(StatementReturnLattice::NotReturn)
                }
        }

        // do while loop always executes at least once
        Statement::DoWhileStatement(stmt) => check_statement(&stmt.body),

        // A switch statement always return if:
        // 1. Every branch that eventually breaks out of the switch breaks via return
        // 2. There is a default case that returns
        Statement::SwitchStatement(stmt) => {
            let mut case_lattices = vec![];
            let mut default_case_lattice = StatementReturnLattice::NotReturn;

            let mut fallthrough_lattice = StatementReturnLattice::NotReturn;
            for case in &stmt.cases {
                let branch_terminated = check_switch_case(case, &mut fallthrough_lattice);

                if case.is_default_case() {
                    default_case_lattice = fallthrough_lattice;
                    // Cases below the default case are not considered.
                    break;
                } else if branch_terminated {
                    case_lattices.push(fallthrough_lattice);
                    fallthrough_lattice = StatementReturnLattice::NotReturn;
                } // Falls through to next case, accumulating lattice
            }

            case_lattices.iter().fold(default_case_lattice, |accum, &lattice| accum.join(lattice))
        }

        Statement::BlockStatement(stmt) => check_block_statement(stmt),

        Statement::LabeledStatement(stmt) => check_statement(&stmt.body),

        Statement::WithStatement(stmt) => check_statement(&stmt.body),

        Statement::TryStatement(stmt) => {
            let mut lattice = check_block_statement(&stmt.block);
            if let Some(catch) = &stmt.handler {
                lattice = lattice.join(check_block_statement(&catch.body));
            }
            if let Some(finally) = &stmt.finalizer {
                lattice = lattice.union(check_block_statement(finally));
            }
            lattice
        }

        _ => StatementReturnLattice::NotReturn,
    }
}

/// Checks whether this switch case falls in:
/// 1. always return explicitly
/// 2. always return at least implicitly
/// 3. might not return and break out of the switch
/// 4. might not return and fall through
pub fn check_switch_case(
    case: &SwitchCase,
    accum: &mut StatementReturnLattice, /* Lattice accumulated from previous branches */
) -> bool {
    for s in &case.consequent {
        // This case is over
        if let Statement::BreakStatement(_) = s {
            return true;
        }

        let lattice = check_statement(s);
        *accum = accum.union(lattice);

        if accum.must_return() {
            return true;
        }
    }

    // This branch does not either return or break. Fall through
    false
}

pub fn check_block_statement(block: &BlockStatement) -> StatementReturnLattice {
    let mut all_statements_lattice = StatementReturnLattice::NotReturn;

    for s in &block.body {
        // The only case where we can see break is if the block is inside a loop,
        // which means the loop does not return
        if let Statement::BreakStatement(_) = s {
            break;
        }

        let lattice = check_statement(s);
        all_statements_lattice = all_statements_lattice.union(lattice);
        if all_statements_lattice.must_return() {
            break;
        }
    }

    all_statements_lattice
}

#[cfg(test)]
mod tests {
    use oxc_allocator::Allocator;
    use oxc_ast::{
        ast::{Declaration, Program},
        SourceType,
    };
    use oxc_parser::Parser;

    use super::*;

    fn parse_statement_and_test(source: &'static str, expected: StatementReturnLattice) {
        let source_type = SourceType::default();
        let alloc = Allocator::default();
        let parser = Parser::new(&alloc, source, source_type);
        let ret = parser.parse();
        assert!(ret.errors.is_empty());

        // The program is a function declaration with a single statement
        let program = ret.program;
        let Program { body, span: _, directives: _, source_type: _ } = program;
        let stmt = body.first().unwrap();
        let Statement::Declaration(Declaration::FunctionDeclaration(func)) = stmt else { unreachable!() };

        let first_statement = &func.body.as_ref().unwrap().statements[0];

        test_match_expected(first_statement, expected);
    }

    fn test_match_expected(statement: &Statement, expected: StatementReturnLattice) {
        let actual = check_statement(statement);

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_switch_always_explicit() {
        // Return Explicit
        let always_explicit = r#"
    function() {
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
        parse_statement_and_test(always_explicit, StatementReturnLattice::AlwaysExplicit);
    }

    #[test]
    fn test_switch_always_implicit() {
        let always_implicit = r#"
    function() {
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
        parse_statement_and_test(always_implicit, StatementReturnLattice::AlwaysImplicit);
    }

    #[test]
    fn test_switch_always_mixed() {
        let always_mixed = r#"
        function() {
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
        parse_statement_and_test(always_mixed, StatementReturnLattice::AlwaysMixed);
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

        parse_statement_and_test(source, StatementReturnLattice::SomeMixed);
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

        parse_statement_and_test(source, StatementReturnLattice::SomeExplicit);
    }

    #[test]
    fn test_if_always_true() {
        let always_true = r#"
      function foo() {
        if (true) return 1;
        else {
          var a = 123;
          console.log(a);
        }
      }
    "#;
        parse_statement_and_test(always_true, StatementReturnLattice::AlwaysExplicit);
    }

    #[test]
    fn test_if_always_false() {
        let always_false = r#"
        function foo() {
          if (false) {
            var a = 123;
          } else {
            return 123;
          }
        }
      "#;
        parse_statement_and_test(always_false, StatementReturnLattice::AlwaysExplicit);
    }

    #[test]
    fn test_if_non_static() {
        let non_static = r#"
        function foo() {
          if (a) {
            return 123;
          } else {
            var c = 0;
          }
        }
      "#;
        parse_statement_and_test(non_static, StatementReturnLattice::SomeExplicit);
    }

    #[test]
    fn test_block() {
        // The block statement could: return a, return, or does not return in the end
        let source = r#"
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
      "#;

        parse_statement_and_test(source, StatementReturnLattice::SomeMixed);
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
        parse_statement_and_test(source, StatementReturnLattice::AlwaysImplicit);
    }
}
