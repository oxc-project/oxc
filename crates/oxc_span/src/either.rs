use std::{borrow::Borrow, fmt, ops::Deref};

/// Either is a symmetrical, general-purpose union type with a [left](`Either::Left`) and a
/// [right](`Either::Right`) variant. It treats its variants the same way and without preference.
/// For representing a successful and error value, use [`Result`] instead.
///
/// This enum is equivalent to `L | R` in TypeScript.
///
/// ## Examples
/// ```
/// use oxc_span::{compact_str::MAX_INLINE_LEN, Atom, CompactStr, Either};
///
/// let source_code = "let x = 42;";
/// // if source_code can be inlined into a CompactStr, use that since it's more efficient.
/// // otherwise, borrow the source code as an Atom to avoid allocations.
/// let s: Either<Atom<'_>, CompactStr> = if source_code.len() > CompactStr::MAX_INLINE_LEN {
///   Either::Left(Atom::from(source_code))
/// } else {
///   Either::Right(CompactStr::new_const(source_code))
/// };
/// // source code is less than 16, so CompactStr gets used
/// assert!(s.is_right());      
/// // Both left and right variants can be borrowed as a &str, so `s` can as well.
/// assert_eq!(source_code, s.as_str());
/// ```
#[derive(Debug)]
pub enum Either<L, R> {
    /// A value of type `L`.
    Left(L),
    /// A value of type `R`.
    Right(R),
}

impl<L: Clone, R: Clone> Clone for Either<L, R> {
    fn clone(&self) -> Self {
        match self {
            Self::Left(val) => Self::Left(val.clone()),
            Self::Right(val) => Self::Right(val.clone()),
        }
    }
}

impl<L: Copy, R: Copy> Copy for Either<L, R> {}

// downcast methods
impl<L, R> Either<L, R> {
    /// Returns `true` if `self` contains a left value.
    ///
    /// # Examples
    pub fn is_left(&self) -> bool {
        matches!(self, Self::Left(_))
    }

    /// Get the left value if it exists. Returns [`None`] if `self` contains a right value.
    pub fn as_left(&self) -> Option<&L> {
        match self {
            Self::Left(val) => Some(val),
            Self::Right(_) => None,
        }
    }

    /// Get the left value if it exists, consuming `self` in the process. Returns [`None`] If
    /// `self` contains a right value.
    pub fn into_left(self) -> Option<L> {
        match self {
            Self::Left(val) => Some(val),
            Self::Right(_) => None,
        }
    }

    /// Returns `true` if `self` contains a right value.
    pub fn is_right(&self) -> bool {
        !self.is_left()
    }

    /// Get the right value if it exists. Returns [`None`] if `self` contains a left value.
    pub fn as_right(&self) -> Option<&R> {
        match self {
            Self::Left(_) => None,
            Self::Right(val) => Some(val),
        }
    }

    /// Get the left value if it exists, consuming `self` in the process. Returns [`None`] if
    /// `self` contains a right value.
    pub fn into_right(self) -> Option<R> {
        match self {
            Self::Left(_) => None,
            Self::Right(val) => Some(val),
        }
    }
}

// conversion methods
impl<L, R> Either<L, R> {
    pub fn map_left<V, F: FnOnce(L) -> V>(self, f: F) -> Either<V, R> {
        match self {
            Self::Left(val) => Either::Left(f(val)),
            Self::Right(val) => Either::Right(val),
        }
    }

    pub fn map_right<V, F: FnOnce(R) -> V>(self, f: F) -> Either<L, V> {
        match self {
            Self::Left(val) => Either::Left(val),
            Self::Right(val) => Either::Right(f(val)),
        }
    }

    pub fn unwrap_left(self) -> L {
        match self {
            Self::Left(val) => val,
            Self::Right(_) => panic!("called `Either::unwrap_left()` on a `Right` value"),
        }
    }

    pub fn unwrap_right(self) -> R {
        match self {
            Self::Left(_) => panic!("called `Either::unwrap_right()` on a `Left` value"),
            Self::Right(val) => val,
        }
    }
}

impl<T> Either<T, T> {
    pub fn map<F, U>(self, f: F) -> Either<U, U>
    where
        F: FnOnce(T) -> U,
    {
        match self {
            Self::Left(l) => Either::Left(f(l)),
            Self::Right(r) => Either::Right(f(r)),
        }
    }
}

// comparison methods
impl<L, R> PartialEq for Either<L, R>
where
    L: PartialEq,
    R: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Left(l1), Self::Left(l2)) => l1.eq(l2),
            (Self::Right(r1), Self::Right(r2)) => r1.eq(r2),
            _ => false,
        }
    }

    fn ne(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Left(l1), Self::Left(l2)) => l1.ne(l2),
            (Self::Right(r1), Self::Right(r2)) => r1.ne(r2),
            _ => true,
        }
    }
}

impl<L, R> Eq for Either<L, R>
where
    L: Eq,
    R: Eq,
{
}

impl<L, R> PartialOrd for Either<L, R>
where
    L: PartialOrd,
    R: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Self::Left(l1), Self::Left(l2)) => l1.partial_cmp(l2),
            (Self::Right(r1), Self::Right(r2)) => r1.partial_cmp(r2),
            _ => None,
        }
    }
}

// ref/deref methods
impl<L, R> Either<L, R> {
    pub fn as_ref(&self) -> Either<&L, &R> {
        match self {
            Self::Left(val) => Either::Left(val),
            Self::Right(val) => Either::Right(val),
        }
    }
}

impl<L, R> Either<L, R>
where
    L: AsRef<str>,
    R: AsRef<str>,
{
    pub fn as_str(&self) -> &str {
        match self {
            Self::Left(val) => val.as_ref(),
            Self::Right(val) => val.as_ref(),
        }
    }

    pub fn len(&self) -> usize {
        self.as_str().len()
    }
}

impl<L, R> Borrow<str> for Either<L, R>
where
    L: AsRef<str>,
    R: AsRef<str>,
{
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl<L, R> AsRef<str> for Either<L, R>
where
    L: AsRef<str>,
    R: AsRef<str>,
{
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<L, R> Either<L, R>
where
    L: Deref,
    R: Deref,
{
    pub fn as_deref(&self) -> Either<&L::Target, &R::Target> {
        match self {
            Self::Left(val) => Either::Left(val),
            Self::Right(val) => Either::Right(val),
        }
    }
}

impl<L: fmt::Display, R: fmt::Display> fmt::Display for Either<L, R> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Left(l) => l.fmt(f),
            Self::Right(r) => r.fmt(f),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{compact_str::MAX_INLINE_LEN, Atom, CompactStr};

    #[test]
    fn test_as_str() {
        let source_code = "let x = 42;";
        // if source_code can be inlined into a CompactStr, use that since it's more efficient.
        // otherwise, borrow the source code as an Atom to avoid allocations.
        let s: Either<Atom<'_>, CompactStr> = if source_code.len() > MAX_INLINE_LEN {
            Either::Left(Atom::from(source_code))
        } else {
            Either::Right(CompactStr::new_const(source_code))
        };
        // source code is less than 16, so CompactStr gets used
        assert!(s.is_right());
        assert_eq!(source_code, s.as_str());
    }
}
