use std::fmt;
use std::hint::cold_path;

/// A LimitTracker enforces a particular limit within the parser. It keeps
/// track of utilization so that we can report how close to a limit we
/// approached over the lifetime of the tracker.
/// ```rust
/// use oxc_graphql_parser::{Allocator, Parser};
///
/// let query = "
/// {
///     animal
///     ...snackSelection
///     ... on Pet {
///       playmates {
///         count
///       }
///     }
/// }
/// ";
/// // Create a new instance of a parser given a query and a
/// // recursion limit
/// let allocator = Allocator::default();
/// let parser = Parser::new(&allocator, query).recursion_limit(4);
/// // Parse the query, and return an AST.
/// let ast = parser.parse();
/// // Retrieve the limits
/// let usage = ast.recursion_limit();
/// // Print out some of the usage details to see what happened during
/// // our parse. `limit` just reports the limit we set, `high` is the
/// // high-water mark of recursion usage.
/// println!("{:?}", usage);
/// println!("{:?}", usage.limit);
/// println!("{:?}", usage.high);
/// // Check that are no errors. These are not part of the AST.
/// assert_eq!(0, ast.errors().len());
///
/// // Get the document root node
/// let doc = ast.document();
/// // ... continue
/// ```
#[derive(PartialEq, Eq, Clone, Copy)]
pub struct LimitTracker {
    pub(crate) current: usize,
    /// High Water mark for this limit
    pub high: usize,
    /// Limit.
    pub limit: usize,
}

impl LimitTracker {
    pub fn new(limit: usize) -> Self {
        Self { current: 0, high: 0, limit }
    }

    /// Return whether the limit was reached
    #[must_use]
    pub fn check_and_increment(&mut self) -> bool {
        self.current += 1;
        if self.current > self.high {
            self.high = self.current;
        }
        let reached = self.current > self.limit;
        if reached {
            cold_path();
            // Caller is gonna return early, keep increments and decrements balanced:
            self.decrement()
        }
        reached
    }

    pub fn decrement(&mut self) {
        self.current -= 1;
    }
}

impl fmt::Debug for LimitTracker {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "recursion limit: {}, high: {}", self.limit, self.high)
    }
}
