// linter-internal module, but intellisense hyperlinks are useful
#![allow(rustdoc::private_intra_doc_links)]
use std::{
    ops::{Deref, Index},
    slice::SliceIndex,
};

use oxc_span::CompactStr;
use schemars::{
    r#gen::SchemaGenerator,
    schema::{ArrayValidation, InstanceType, Schema, SchemaObject},
    JsonSchema,
};
use serde::{Deserialize, Serialize};

/// Creates a [`Set`] containing the arguments.
///
/// Usage is identical to the [`vec!`] macro.
#[macro_export]
macro_rules! set {
    () => (
        $crate::utils::Set::default()
    );
    ($($x:expr),+ $(,)?) => (
        $crate::utils::Set::new([$($x),+])
    );
}

/// An ordered set of unique elements backed by a [`Vec`].
///
/// ## Why not use `HashSet` or `BTreeSet`?
///
/// We often find ourselves storing lists of unique strings and using
/// `Vec::contains` in lint rules. These sets are constructed once and queried
/// many times. A simple vec-based set has these advantages:
///
/// - `HashSet` has a higher space complexity - technically it is *O*(*n*), but
///   it has a high constant factor to reduce key collisions. This set is
///   exactly *O*(*n*).
/// - `BTreeSet` has a space complexity of *O*(*n*), but each key is pointed to.
///   This means searching requires *O*(*log(n)*) pointer dereferences. This set
///   stores all of its elements in the same block of memory, leading to much
///   better cache locality.
///
/// The major tradeoff here is that set construction is *O*(*nlog*(*n*)) and
/// insertion is *O*(*log*(*n*)). This is fine if the set is only built once,
/// but can lead to massive overhead if it is constantly added to. Because of
/// this, there is not insertion API. If you need to store keys while your lint
/// rule is running, prefer one of the two aforementioned set implementations.
///
/// ## Containment Checks
///
/// The main API for this set is [`contains`]. However, if you're storing
/// string-like keys and querying with string-like keys of a different type, you
/// can use [`contains_str`] to avoid allocations.
///
/// ```ignore
/// # use oxc_linter::utils::Set;
/// let set: Set<String> = Set::new(["foo".to_string(), "bar".to_string()]);
/// // `contains` without allocating a string
/// assert!(set.contains_str("foo"));
/// ```
///
/// ## Construction
///
/// [`Set`] implements [`FromIterator`], meaning you can easily `collect` into
/// it from any iterator.
///
/// ```ignore
/// # use oxc_linter::utils::Set;
/// let items = ["foo", "bar", "baz", "duplicate", "duplicate"];
/// let set: Set<_> = items.into_iter().collect();
/// assert_eq!(set.len(), 4); // duplicate is removed
/// ```
///
/// There is also a [`set!`] macro that works identically to [`vec!`].
/// ```ignore
/// # use oxc_linter::utils::{Set, set};
///
/// let s = set![1, 2, 3];
/// assert_eq!(s.len(), 3);
/// assert!(s.contains(&1));
/// ```
///
/// ## Usage in Rule Configs
///
/// [`Set<CompactStr>`] can be easily constructed from a [`serde_json::Value`] to make using
/// it in a config simpler. Note that you cannot create a [`Set<String>`] this
/// way - you should be using [`CompactStr`] anyways.
///
/// [`contains`]: Set::contains
/// [`contains_str`]: Set::contains_str
#[derive(Debug, Clone)]
pub struct Set<T>(Vec<T>);
// NOTE TO CONTRIBUTORS: do _NOT_ impl `DerefMut` for `Set`. We don't want to
// expose mutable methods that may break sorted/unique guarantees.

impl<T> Default for Set<T> {
    #[inline]
    fn default() -> Self {
        Self(Vec::new())
    }
}

impl<T> Deref for Set<T> {
    type Target = [T];

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.0.as_slice()
    }
}

impl<T> Set<T> {
    /// Creates a new set where entries are stored and deduplicated based on a
    /// comparison function. Useful for storing complex types that may not be
    /// inherently orderable.
    ///
    /// Containment checks for sets created this way can be done using [`contains_by`].
    ///
    /// [`contains_by`]: Set::contains_by
    pub fn new_by<I, F>(iter: I, compare: F) -> Self
    where
        I: IntoIterator<Item = T>,
        F: Fn(&T, &T) -> std::cmp::Ordering,
    {
        let mut vec = Vec::from_iter(iter);
        vec.sort_unstable_by(&compare);
        vec.dedup_by(|a, b| compare(a, b).is_eq());
        Self(vec)
    }

    /// Like [`contains`], but uses a custom comparison function to search for entries.
    ///
    /// This is very useful when storing complex types that may not be
    /// inherently orderable. You can create such sets using [`new_by`].
    ///
    /// [`contains`]: Set::contains
    /// [`new_by`]: Set::new_by
    pub fn contains_by<F>(&self, item: &T, compare: F) -> bool
    where
        F: Fn(&T, &T) -> std::cmp::Ordering,
    {
        self.0.binary_search_by(|el| compare(el, item)).is_ok()
    }

    /// Returns the number of elements in the set, also referred to
    /// as its 'length'.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// # use oxc_linter::utils::{Set, set};
    /// let a = set![1, 2, 3];
    /// assert_eq!(a.len(), 3);
    /// ```
    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns `true` if the set contains no elements.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// # use oxc_linter::utils::Set;
    /// let mut v = Set::new();
    /// assert!(v.is_empty());
    ///
    /// v.push(1);
    /// assert!(!v.is_empty());
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns a reference to an element or subslice depending on the type of
    /// index.
    ///
    /// - If given a position, returns a reference to the element at that
    ///   position or `None` if out of bounds.
    /// - If given a range, returns the subslice corresponding to that range,
    ///   or `None` if out of bounds.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// # use oxc_linter::utils::Set;
    /// let set = Set::new([10, 40, 30]);
    /// assert_eq!(Some(&30), set.get(1));
    /// assert_eq!(Some(&40), set.get(2));
    /// assert_eq!(Some(&[10, 30][..]), set.get(0..2));
    /// assert_eq!(None, set.get(3));
    /// assert_eq!(None, set.get(0..4));
    /// ```
    #[must_use]
    pub fn get<I>(&self, index: I) -> Option<&I::Output>
    where
        I: SliceIndex<[T]>,
    {
        self.0.get(index)
    }

    /// Returns an iterator over the set.
    ///
    /// The iterator yields all items from start to end.
    ///
    /// # Examples
    ///
    /// ```
    /// # use oxc_linter::utils::Set;
    /// let x = Set::new([1, 2, 4]);
    /// let mut iterator = x.iter();
    ///
    /// assert_eq!(iterator.next(), Some(&1));
    /// assert_eq!(iterator.next(), Some(&2));
    /// assert_eq!(iterator.next(), Some(&4));
    /// assert_eq!(iterator.next(), None);
    /// ```
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.0.iter()
    }
}

impl<T: Ord> Set<T> {
    /// Create a new set that contains the elements of the given iterator.
    ///
    /// The elements are sorted and deduplicated.
    ///
    /// ## Examples
    /// ```ignore
    /// # use oxc_linter::utils::Set;
    /// let set = Set::new([3, 3, 1, 1, 2]);
    /// assert_eq!(set.len(), 3);
    /// assert_eq!(set, Set::new([1, 2, 3]));
    /// ```
    #[must_use]
    pub fn new<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self::from_iter(iter)
    }

    /// Returns `true` if the [`Set`] contains an element with the given value.
    ///
    /// This operation is *O*(*log(n)*).
    ///
    /// If `T` can be borrowed as a [`str`] and want to search by a different
    /// kind of key, consider using [`Set::contains_str`].
    ///
    /// # Examples
    ///
    /// ```ignore
    /// # use oxc_linter::utils::Set;
    /// let set = Set::new([10, 40, 30]);
    /// assert!(v.contains(&30));
    /// assert!(!v.contains(&50));
    /// ```
    ///
    /// If you do not have a `&T`, but some other value that you can compare
    /// with one and `T` can be referenced as a [`str`] (for example, [`String`]
    /// implements [`PartialEq<str>`]), you can use [`Set::contains_str`]
    /// instead.
    ///
    /// ```ignore
    /// # use oxc_linter::utils::Set;
    /// let set = Set::new([String::from("hello"), String::from("world")]); // set of `String`
    /// assert!(set.contains_str("hello")); // search with `&str`
    /// assert!(!set.contains_str("hi"));
    /// ```
    pub fn contains(&self, item: &T) -> bool where {
        self.0.binary_search(item).is_ok()
    }
}

impl<T: AsRef<str> + Ord> Set<T> {
    /// A specialized implementation of [`Set::contains`] for string-like keys.
    /// Helps you perform containment checks without allocating a new [`String`].
    ///
    /// ## Example
    ///
    /// ```ignore
    /// # use oxc_linter::utils::Set;
    ///
    /// let set: Set<String> = Set::new([
    ///     "foo".to_string(),
    ///     "bar".to_string(),
    ///     "baz".to_string()
    /// ]);
    /// assert!(set.contains_str("foo")); // no String allocation
    /// ```
    pub fn contains_str<Q>(&self, item: Q) -> bool
    where
        Q: AsRef<str>,
    {
        let key = item.as_ref();
        self.0.binary_search_by(|el| el.as_ref().cmp(key)).is_ok()
    }
}

impl<T> Index<usize> for Set<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl<T: PartialEq> PartialEq for Set<T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T: Eq> Eq for Set<T> {}

impl<T: Ord> From<Vec<T>> for Set<T> {
    fn from(mut vec: Vec<T>) -> Self {
        vec.sort_unstable();
        vec.dedup();
        Self(vec)
    }
}

impl<T: Ord> FromIterator<T> for Set<T> {
    #[must_use]
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self::from(Vec::from_iter(iter))
    }
}

impl<T> IntoIterator for Set<T> {
    type Item = T;
    type IntoIter = <Vec<T> as IntoIterator>::IntoIter;

    #[must_use]
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'s, T> IntoIterator for &'s Set<T> {
    type Item = &'s T;
    type IntoIter = <&'s Vec<T> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl TryFrom<&serde_json::Value> for Set<CompactStr> {
    type Error = &'static str;

    fn try_from(value: &serde_json::Value) -> Result<Self, Self::Error> {
        value.as_array().map_or(Err("Value is not an array"), |arr| {
            Ok(arr.iter().filter_map(serde_json::Value::as_str).map(CompactStr::from).collect())
        })
    }
}

impl<T: Serialize> Serialize for Set<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de, T: Deserialize<'de> + Ord> Deserialize<'de> for Set<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let vec = Vec::<T>::deserialize(deserializer)?;
        Ok(Self::from(vec))
    }
}

impl<T: JsonSchema> JsonSchema for Set<T> {
    fn schema_name() -> String {
        format!("Set_of_{}", T::schema_name())
    }

    fn schema_id() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Owned(format!("Set<{}>", T::schema_id()))
    }

    fn json_schema(gen: &mut SchemaGenerator) -> Schema {
        SchemaObject {
            instance_type: Some(InstanceType::Array.into()),
            array: Some(Box::new(ArrayValidation {
                unique_items: Some(true),
                items: Some(gen.subschema_for::<T>().into()),
                ..Default::default()
            })),
            ..Default::default()
        }
        .into()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_empty() {
        let set: Set<u32> = Set::default();
        assert!(set.is_empty());
        assert_eq!(set.len(), 0);
        assert_eq!(set, set![]);
    }

    #[test]
    fn test_new() {
        let set = Set::new([3, 3, 1, 1, 2]);
        assert_eq!(set.len(), 3);
        assert_eq!(set, Set::new([1, 2, 3]));
    }

    #[test]
    fn test_contains() {
        let set = Set::new(["foo", "bar", "baz"]);

        assert!(set.contains(&"foo"));
        assert!(set.contains_str("foo"));

        let bar = "bar".to_string();
        assert!(set.contains(&bar.as_str()));
        assert!(set.contains_str(&bar));

        assert!(!set.contains_str("nope"));
    }

    #[test]
    fn test_set_with_derived_ordering() {
        #[allow(dead_code)]
        struct User {
            name: &'static str,
            age: u32,
            logged_in: bool,
        }

        let users = [
            User { name: "Alice", age: 25, logged_in: true },
            User { name: "Bob", age: 30, logged_in: false },
            User { name: "Charlie", age: 20, logged_in: true },
            User { name: "Charlie", age: 40, logged_in: false },
        ];
        let set = Set::new_by(users, |a, b| a.name.cmp(b.name));
        assert_eq!(set.len(), 3);
        assert_eq!(set[0].name, "Alice");
        let find_bob = User { name: "Bob", age: 30, logged_in: false };
        assert!(set.contains_by(&find_bob, |a, b| a.name.cmp(b.name),));
    }

    #[test]
    fn test_index() {
        let set = Set::new([10, 40, 30]);
        assert_eq!(Some(&30), set.get(1));
        assert_eq!(Some(&40), set.get(2));
        assert_eq!(Some(&[10, 30][..]), set.get(0..2));
        assert_eq!(None, set.get(3));
        assert_eq!(None, set.get(0..4));
    }
}
