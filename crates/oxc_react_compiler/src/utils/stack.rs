/// An immutable stack data structure supporting O(1) push/pop operations.
///
/// Port of `Utils/Stack.ts` from the React Compiler.
///
/// This is a persistent (immutable) linked list used as a stack.
/// Push and pop return new stack instances, sharing structure with the original.
use std::sync::Arc;

/// An immutable stack. Either empty or contains a value and a reference to the rest.
#[derive(Debug)]
pub enum Stack<T> {
    Empty,
    Node { value: T, next: Arc<Stack<T>> },
}

impl<T: Clone> Clone for Stack<T> {
    fn clone(&self) -> Self {
        match self {
            Stack::Empty => Stack::Empty,
            Stack::Node { value, next } => {
                Stack::Node { value: value.clone(), next: Arc::clone(next) }
            }
        }
    }
}

impl<T> Default for Stack<T> {
    fn default() -> Self {
        Self::empty()
    }
}

impl<T> Stack<T> {
    /// Creates a new empty stack.
    pub fn empty() -> Self {
        Stack::Empty
    }

    /// Creates a new stack with a single value.
    pub fn create(value: T) -> Self {
        Stack::Node { value, next: Arc::new(Stack::Empty) }
    }

    /// Returns the value at the top of the stack, or `None` if empty.
    pub fn value(&self) -> Option<&T> {
        match self {
            Stack::Empty => None,
            Stack::Node { value, .. } => Some(value),
        }
    }

    /// Returns `true` if the stack is empty.
    pub fn is_empty(&self) -> bool {
        matches!(self, Stack::Empty)
    }
}

impl<T: Clone> Stack<T> {
    /// Pushes a new value onto the stack, returning a new stack.
    pub fn push(&self, value: T) -> Self {
        Stack::Node { value, next: Arc::new(self.clone()) }
    }

    /// Pops the top value from the stack, returning the rest.
    /// If the stack is empty, returns an empty stack.
    pub fn pop(&self) -> Self {
        match self {
            Stack::Empty => Stack::Empty,
            Stack::Node { next, .. } => next.as_ref().clone(),
        }
    }
}

impl<T: PartialEq> Stack<T> {
    /// Returns `true` if the stack contains the given value.
    pub fn contains(&self, value: &T) -> bool {
        match self {
            Stack::Empty => false,
            Stack::Node { value: v, next } => v == value || next.contains(value),
        }
    }
}

impl<T> Stack<T> {
    /// Returns `true` if any element in the stack satisfies the predicate.
    pub fn find(&self, f: &impl Fn(&T) -> bool) -> bool {
        match self {
            Stack::Empty => false,
            Stack::Node { value, next } => {
                if f(value) {
                    true
                } else {
                    next.find(f)
                }
            }
        }
    }

    /// Calls the given function for each element in the stack.
    pub fn each(&self, f: &mut impl FnMut(&T)) {
        match self {
            Stack::Empty => {}
            Stack::Node { value, next } => {
                f(value);
                next.each(f);
            }
        }
    }

    /// Prints the stack elements using the given formatting function.
    pub fn print(&self, f: &impl Fn(&T) -> String) -> String {
        match self {
            Stack::Empty => String::new(),
            Stack::Node { value, next } => {
                let mut result = f(value);
                result.push_str(&next.print(f));
                result
            }
        }
    }
}
