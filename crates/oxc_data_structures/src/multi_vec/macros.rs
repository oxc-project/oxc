//! [`multi_vec!`] macro, generating a named struct-of-arrays table type wrapping a [`MultiVec`].
//!
//! The macro's expansion contains almost no logic. All of it - allocation, layout, pointer
//! arithmetic, bounds checks, clone / drop loops - lives in [`MultiVec`] and the sibling modules,
//! as ordinary generic Rust. The only unsafe code the macro generates is the `Fields`-family
//! impls - pure pointer casts and calls to the `utils` helpers, with no arithmetic and
//! no control flow.
//!
//! [`MultiVec`]: super::MultiVec

/// Create a struct-of-arrays table type.
///
/// A generated table stores each field of the struct in its own array (column), all the columns
/// within a single allocation, indexed by a typed ID.
///
/// ```
/// use oxc_data_structures::multi_vec::multi_vec;
/// use oxc_index::define_index_type;
///
/// define_index_type! { pub struct ScopeId = u32; }
///
/// multi_vec! {
///     pub table ScopeTable<ScopeId, Scope>;
///
///     pub struct Scope {
///         pub parent_id: Option<ScopeId>,
///         pub depth: u32,
///     }
/// }
///
/// let mut table = ScopeTable::new();
/// let root_id = table.push(Scope { parent_id: None, depth: 0 });
/// let child_id = table.push(Scope { parent_id: Some(root_id), depth: 1 });
///
/// // Access one element's field, or a whole column at once.
/// assert_eq!(*table.parent_id(child_id), Some(root_id));
/// assert_eq!(table.depths(), &[0, 1]);
/// ```
///
/// # Generated items
///
/// For the declaration above, the macro generates:
///
/// * `Scope` - the plain struct, exactly as written. `push` takes one, and consuming iteration
///   yields them back.
/// * `ScopeTable` - the table itself, storing `Scope`s field-by-field, indexed by `ScopeId`.
/// * `ScopeRef` / `ScopeMut` - views of one element, with a `&T` / `&mut T` per field.
///   Returned by `get` / `get_mut`.
/// * `ScopeSlices` / `ScopeSlicesMut` - views of the whole table, with an [`IndexSlice`]
///   (one whole column) per field. Returned by `slices` / `slices_mut`.
/// * `ScopeTableIter` / `ScopeTableIterMut` / `ScopeTableIntoIter` - the iterator types,
///   yielding `ScopeRef`s / `ScopeMut`s / owned `Scope`s.
///
/// View type names are derived from the struct's name, and iterator type names from the table's.
///
/// The types are defined directly in the invoking scope (module scope or inside a function),
/// each with the visibility given on the `table` declaration. Everything else the macro
/// generates is hidden inside an anonymous `const` block, and adds no other names to the scope.
///
/// # Methods
///
/// ```
/// # use oxc_data_structures::multi_vec::multi_vec;
/// # oxc_index::define_index_type! { pub struct ScopeId = u32; }
/// # multi_vec! {
/// #     pub table ScopeTable<ScopeId, Scope>;
/// #
/// #     pub struct Scope {
/// #         pub parent_id: Option<ScopeId>,
/// #         pub depth: u32,
/// #     }
/// # }
/// // Creation. `new` is `const`. Capacity grows as needed - `with_capacity` / `reserve`
/// // preallocate.
/// let mut table = ScopeTable::with_capacity(2);
/// assert!(table.is_empty());
///
/// // `push` returns the new element's ID.
/// let root_id = table.push(Scope { parent_id: None, depth: 0 });
/// let child_id = table.push(Scope { parent_id: Some(root_id), depth: 1 });
/// assert_eq!(table.len(), 2);
/// table.reserve(8);
///
/// // One element - `get` / `get_mut` return a view with a reference to every field.
/// let child = table.get(child_id);
/// assert_eq!(*child.parent_id, Some(root_id));
/// *table.get_mut(child_id).depth += 1;
///
/// // One field of one element - an accessor pair per field.
/// assert_eq!(*table.depth(child_id), 2);
/// *table.depth_mut(child_id) -= 1;
///
/// // One field of all elements - a plural accessor pair per field, returning `IndexSlice`s
/// // indexed by `ScopeId`, not `usize`. (`slices` / `slices_mut` return all columns at once.)
/// assert_eq!(table.depths(), &[0, 1]);
/// table.depths_mut()[root_id] = 5;
/// assert_eq!(table.slices().depths.len(), 2);
///
/// // Iteration - `iter` / `iter_mut` yield the view types. `&table` / `&mut table` also work.
/// let total: u32 = table.iter().map(|scope| *scope.depth).sum();
/// assert_eq!(total, 6);
/// for scope in table.iter_mut() {
///     *scope.depth += 1;
/// }
///
/// // `iter_ids` yields every element's ID. It does not borrow the table, so the table can be
/// // mutated while it is held.
/// let ids: Vec<ScopeId> = table.iter_ids().collect();
/// assert_eq!(ids, [root_id, child_id]);
///
/// // Enumerated iteration pairs each element with its ID
/// // (also `iter_mut_enumerated` / `into_iter_enumerated`).
/// for (id, scope) in table.iter_enumerated() {
///     assert_eq!(*table.depth(id), *scope.depth);
/// }
///
/// // Consuming the table yields owned elements.
/// let scopes: Vec<Scope> = table.into_iter().collect();
/// assert_eq!(scopes.len(), 2);
/// ```
///
/// Also generated:
///
/// * `get_unchecked` / `get_unchecked_mut` - `unsafe` variants which skip the bounds check.
/// * `MAX_CAPACITY` - the capacity limit (the index type's range, capped by the maximum
///   allocation size). Exceeding it panics.
/// * A `Default` impl. `Clone` and `Debug` impls if derived - see below.
///
/// # Lifetimes
///
/// The struct's fields may borrow. Declare the lifetimes at the start of the `table`
/// declaration's generics, and apply them to the item type - the struct's lifetime
/// params, with the same names, in its declaration order.
///
/// ```
/// # use oxc_data_structures::multi_vec::multi_vec;
/// # oxc_index::define_index_type! { pub struct AliasId = u32; }
/// multi_vec! {
///     #[derive(Clone, Debug)]
///     table Aliases<'a, 'b, AliasId, Alias<'a, 'b>>;
///
///     struct Alias<'a, 'b> {
///         name: &'a str,
///         alias_name: &'b str,
///     }
/// }
///
/// let name = String::from("hello");
/// let mut aliases = Aliases::new();
/// let id = aliases.push(Alias { name: &name, alias_name: "hi" });
/// assert_eq!(*aliases.name(id), "hello");
/// for alias in &aliases {
///     assert_eq!(*alias.alias_name, "hi");
/// }
/// ```
///
/// Each generated type takes the lifetimes it needs:
///
/// * The table type takes the table's lifetimes - `Aliases<'a, 'b>`.
/// * The reference views and the borrowing iterators take a borrow lifetime, then the struct's -
///   `AliasRef<'v, 'a, 'b>`, `AliasesIter<'v, 'a, 'b>`. The consuming iterator takes just the
///   struct's - `AliasesIntoIter<'a, 'b>`.
/// * The slice views take a borrow lifetime, then the *table's* (their fields name the key
///   type) - `AliasSlices<'v, 'a, 'b>`.
///
/// All are covariant in the struct's lifetimes, like `Vec` is in `T`'s.
///
/// The two declarations must use the same lifetime names - the struct's lifetimes
/// (under their own names) parameterize the generated view types and per-field methods.
/// Different lifetime names are a compile-time error:
///
/// ```compile_fail,E0261
/// # use oxc_data_structures::multi_vec::multi_vec;
/// # oxc_index::define_index_type! { pub struct AliasId = u32; }
/// multi_vec! {
///     // `'x` does not match the struct's `'a` - error
///     table Aliases<'x, AliasId, Alias<'x>>;
///
///     struct Alias<'a> {
///         name: &'a str,
///     }
/// }
/// ```
///
/// The key type may also take lifetime arguments (e.g. a branded ID type), declared in
/// the same leading list when not `'static`. (`oxc_index::Idx` currently requires
/// `'static`, so non-`'static` key lifetimes await an `Idx` relaxation.)
///
/// The borrowed data must outlive the table *strictly* - the table's drop code runs while
/// the borrows must still be valid (unlike `Vec`, which is exempted from this rule by
/// unstable machinery). In practice, declare the table *after* the data it borrows,
/// so it is dropped first. This fails because `name` is dropped before `aliases`:
///
/// ```compile_fail,E0597
/// # use oxc_data_structures::multi_vec::multi_vec;
/// # oxc_index::define_index_type! { pub struct AliasId = u32; }
/// # multi_vec! {
/// #     table Aliases<'a, AliasId, Alias<'a>>;
/// #
/// #     struct Alias<'a> {
/// #         name: &'a str,
/// #     }
/// # }
/// let mut aliases = Aliases::new();
/// let name = String::from("hello"); // declared after `aliases` - dropped before it
/// aliases.push(Alias { name: &name });
/// ```
///
/// # Cloning
///
/// The generated types implement `Clone` only if you derive it:
///
/// * `#[derive(Clone)]` on the `table` declaration makes the table cloneable, and derives `Clone`
///   on the struct too for you. Do not derive `Clone` on the struct yourself.
/// * All field types must be `Clone`, as usual for a derive.
///
/// ```
/// # use oxc_data_structures::multi_vec::multi_vec;
/// # oxc_index::define_index_type! { struct Id = u32; }
/// multi_vec! {
///     #[derive(Clone)]
///     table CloneTable<Id, Item>;
///
///     struct Item { value: u32 }
/// }
///
/// let mut table = CloneTable::new();
/// let id = table.push(Item { value: 1 });
/// let clone = table.clone();
/// assert_eq!(*clone.value(id), 1);
///
/// // The struct is `Clone` too - the macro derived it
/// let _item = Item { value: 2 }.clone();
/// ```
///
/// Cloning a table clones column by column, running each field type's `Clone` impl element by element.
/// Every field's `Clone` is honored, even a custom impl on a `Copy` type.
/// For trivially-cloneable field types the compiler reduces a column's clone to a single `memcpy`.
///
/// The struct's own `Clone` impl is never invoked when cloning a table - elements are stored
/// as scattered field values, so there is never a whole struct value to clone. This is why the
/// macro insists on supplying the struct's `Clone` itself - the *derived*, field-wise impl
/// is exactly what cloning the table does, and a manual `impl Clone` with different behavior
/// cannot exist alongside it:
///
/// ```compile_fail,E0119
/// # use oxc_data_structures::multi_vec::multi_vec;
/// # oxc_index::define_index_type! { struct Id = u32; }
/// multi_vec! {
///     #[derive(Clone)]
///     table CloneTable<Id, Item>;
///
///     struct Item { value: u32 }
/// }
///
/// // Conflicts with the macro's derived impl - error
/// impl Clone for Item {
///     fn clone(&self) -> Self { Item { value: self.value + 1 } }
/// }
/// ```
///
/// For the same reason, deriving `Clone` on the struct yourself conflicts with the
/// macro's derive:
///
/// ```compile_fail,E0119
/// # use oxc_data_structures::multi_vec::multi_vec;
/// # oxc_index::define_index_type! { struct Id = u32; }
/// multi_vec! {
///     #[derive(Clone)]
///     table CloneTable<Id, Item>;
///
///     // The macro already derives `Clone` on the struct - error
///     #[derive(Clone)]
///     struct Item { value: u32 }
/// }
/// ```
///
/// # Debug
///
/// Likewise, the generated types implement `Debug` only if you derive it:
///
/// * `#[derive(Debug)]` on the `table` declaration makes the table and the view types
///   (e.g. `ScopeRef`) implement `Debug`, and derives `Debug` on the struct for you.
///   As with `Clone`, do not derive `Debug` on the struct yourself.
/// * All field types must be `Debug`, as usual for a derive.
///
/// The table prints as a map from each element's ID to the element, formatted through
/// the shared-reference view - e.g. `{ 0: ScopeRef { parent_id: None, ... }, 1: ... }`.
///
/// ```compile_fail,E0119
/// # use oxc_data_structures::multi_vec::multi_vec;
/// # oxc_index::define_index_type! { struct Id = u32; }
/// multi_vec! {
///     #[derive(Debug)]
///     table DebugTable<Id, Item>;
///
///     // The macro already derives `Debug` on the struct - error
///     #[derive(Debug)]
///     struct Item { value: u32 }
/// }
/// ```
///
/// # Dropping
///
/// Field types which need dropping (e.g. `String`) are fully supported - the stored field
/// values are dropped when the table is dropped. But elements only ever exist as scattered
/// field values, so the struct's own `Drop` impl (if it has one) is never invoked.
/// Do not implement `Drop` on the struct.
///
/// # Pluralization
///
/// The whole-column accessors and the slice views fields' names are pluralized - each concerns
/// a whole column, so e.g. a `parent_id` field produces `parent_ids`.
///
/// By default the plural is formed by appending `s`. Where that produces the wrong name
/// (e.g. `flags` -> `flagss`), specify the plural with a `#[plural(...)]` attribute on the field:
///
/// ```
/// # use oxc_data_structures::multi_vec::multi_vec;
/// # oxc_index::define_index_type! { struct Id = u32; }
/// multi_vec! {
///     table StatusTable<Id, Status>;
///
///     struct Status {
///         #[plural(all_flags)]
///         flags: u16,
///     }
/// }
///
/// let mut table = StatusTable::new();
/// table.push(Status { flags: 3 });
/// assert_eq!(table.all_flags(), &[3]);
/// ```
///
/// The attribute may appear anywhere among the field's doc comments.
///
/// # Field visibility
///
/// Each field carries the visibility written on it in the `struct` - private if bare, `pub`,
/// `pub(crate)`, and so on. That visibility is applied to the field in the struct and in every
/// view type, and to all four of the field's accessor methods (`.name()` / `.name_mut()` /
/// `.names()` / `.names_mut()`). As always, a field's effective visibility is bounded by
/// its struct's.
///
/// ```
/// mod inner {
///     # use oxc_data_structures::multi_vec::multi_vec;
///     # oxc_index::define_index_type! { pub struct Id = u32; }
///     multi_vec! {
///         pub table Items<Id, Item>;
///
///         pub struct Item {
///             secret: u32,           // private to `inner`
///             pub value: u32,        // public
///         }
///     }
///
///     // Built here because the `secret` field is private to this module.
///     pub fn make() -> (Items, Id) {
///         let mut items = Items::new();
///         let id = items.push(Item { secret: 1, value: 10 });
///         (items, id)
///     }
/// }
///
/// let (items, id) = inner::make();
/// // `value` and its accessor are public, so reachable here.
/// assert_eq!(*items.value(id), 10);
/// ```
///
/// A private field's accessor is not callable from outside the defining module:
///
/// ```compile_fail,E0624
/// mod inner {
///     # use oxc_data_structures::multi_vec::multi_vec;
///     # oxc_index::define_index_type! { pub struct Id = u32; }
///     multi_vec! {
///         pub table Items<Id, Item>;
///
///         pub struct Item {
///             secret: u32, // private to `inner`
///         }
///     }
/// }
///
/// let items = inner::Items::new();
/// let id = inner::Id::from_raw(0);
/// let _ = items.secret(id); // error: method `secret` is private
/// ```
///
/// The `struct`'s own visibility must match the `table`'s. A mismatch (here `pub` table,
/// private struct) is a compile-time error:
///
/// ```compile_fail,E0080
/// # use oxc_data_structures::multi_vec::multi_vec;
/// # oxc_index::define_index_type! { pub struct Id = u32; }
/// multi_vec! {
///     pub table Items<Id, Item>;
///
///     // Missing `pub` - error
///     struct Item {
///         value: u32,
///     }
/// }
/// ```
///
/// # Attributes
///
/// Attributes (including doc comments) on the `table` declaration are applied to the generated
/// table type, and attributes on the `struct` to the generated struct - but not to the other
/// generated types, so e.g. `#[cfg(...)]` will not work as expected.
///
/// `Clone` and `Debug` are the only derives accepted on the `table` declaration.
/// Any other derive is a compile-time error:
///
/// ```compile_fail
/// # use oxc_data_structures::multi_vec::multi_vec;
/// # oxc_index::define_index_type! { struct Id = u32; }
/// multi_vec! {
///     // `PartialEq` is not supported - error
///     #[derive(Clone, PartialEq)]
///     table PartialEqTable<Id, Item>;
///
///     // Other derives are fine on the struct - it is a plain struct.
///     // (But not `Clone` / `Debug` when the table derives them - see "Cloning".)
///     #[derive(PartialEq)]
///     struct Item { value: u32 }
/// }
/// ```
///
/// Fields accept only doc comments and `#[plural(...)]`. Doc comments are applied to the
/// struct's field, and to the corresponding fields of the reference views. Any other attribute
/// is a compile-time error - it would apply only to the generated struct's field, not to the
/// fields of the generated view types:
///
/// ```compile_fail
/// # use oxc_data_structures::multi_vec::multi_vec;
/// # oxc_index::define_index_type! { struct Id = u32; }
/// multi_vec! {
///     table ItemTable<Id, Item>;
///
///     struct Item {
///         #[serde(skip)] // not supported - error
///         value: u32,
///     }
/// }
/// ```
///
/// A duplicate `#[plural(...)]` is also a compile-time error:
///
/// ```compile_fail
/// # use oxc_data_structures::multi_vec::multi_vec;
/// # oxc_index::define_index_type! { struct Id = u32; }
/// multi_vec! {
///     table ItemTable<Id, Item>;
///
///     struct Item {
///         #[plural(a)]
///         #[plural(b)] // duplicate - error
///         value: u32,
///     }
/// }
/// ```
///
/// # Restrictions
///
/// There must be at least one field, and at least one field must have non-zero size
/// (a table whose fields are all zero-sized types would have a zero-sized allocation).
/// Both are enforced when the table is defined, at check time:
///
/// ```compile_fail,E0080
/// # use oxc_data_structures::multi_vec::multi_vec;
/// # oxc_index::define_index_type! { struct Id = u32; }
/// // All fields are zero-sized types - error
/// multi_vec! {
///     table AllZstTable<Id, AllZst>;
///
///     struct AllZst {
///         x: (),
///         y: (),
///     }
/// }
/// ```
///
/// ```compile_fail
/// # use oxc_data_structures::multi_vec::multi_vec;
/// # oxc_index::define_index_type! { struct Id = u32; }
/// // `Empty` has no fields - error
/// multi_vec! {
///     table EmptyTable<Id, Empty>;
///
///     struct Empty {}
/// }
/// ```
///
/// The item type in the `table` declaration must be the struct declared below it.
/// Naming a different fields struct (e.g. another table's, which would otherwise compile,
/// silently binding the table to the wrong element type) is a compile-time error:
///
/// ```compile_fail,E0308
/// # use oxc_data_structures::multi_vec::multi_vec;
/// # oxc_index::define_index_type! { struct Id = u32; }
/// # multi_vec! {
/// #     table OtherTable<Id, Other>;
/// #
/// #     struct Other {
/// #         value: u32,
/// #     }
/// # }
/// // `Other` is another table's struct, not the one declared below - error
/// multi_vec! {
///     table WrongItemTable<Id, Other>;
///
///     struct Item {
///         value: u32,
///     }
/// }
/// ```
///
/// [`IndexSlice`]: oxc_index::IndexSlice
#[macro_export]
macro_rules! multi_vec {
    // Entry point.
    //
    // The `table` declaration's attributes are classified first, one at a time, by the
    // `@table_attrs` rules:
    //
    // * Ordinary attributes (doc comments) pass through to the generated table type.
    // * `#[derive(Clone)]` is recorded in `config`'s `clone` slot - it drives explicit
    //   `#[derive(Clone)]`s on the table type and the struct, and the `@generate_clone` rules.
    // * `#[derive(Debug)]` is recorded in the `debug` slot - it drives explicit
    //   `#[derive(Debug)]`s on the struct and view types, and gates the table's `Debug` impl.
    // * Any other derive is a compile error.
    //
    // An attribute with several derives (e.g. `#[derive(Clone, Debug)]`) is first split
    // into one attribute per derive.
    //
    // Fields are then normalized one at a time: `@normalize` matches one whole field -
    // attributes, name, and type - and builds its normalized record. The `@field`
    // rules then classify the attributes one at a time, refining the record in place:
    // doc comments into `field_comments` (emitted onto the struct field),
    // `#[plural(...)]` into `field_plural_name`, and anything else is a compile error.
    // Once every field is normalized, the `@generate` rule generates the output.
    // To add a new field attribute: add a record slot, a classify rule for it, and
    // pass the slot through the other `@field` rules.
    //
    // Attributes are captured as raw token trees, NOT `:meta` fragments - a captured
    // `:meta` fragment is opaque, so the classify rules could not tell `#[plural(...)]`
    // from `#[doc]`. (Doc comments reach the macro already lowered to `#[doc = "..."]`
    // attribute tokens, so they are matched structurally like any other attribute.)
    //
    // Normalized fields have the form
    // `{ field_vis = [...] field_name = [...] field_ty = [...] field_plural_name = [...]
    //    field_plural_name_mut = [...] field_comments = [...] }`.
    // `field_vis` is the field's visibility (from the `struct` field, e.g. `pub` /
    // `pub(crate)` / empty), captured as `:vis` and threaded through as an opaque `:tt`
    // like `field_ty`. `@generate` re-parses it as `:vis` and applies it to the field in
    // every generated type and to the field's accessor methods.
    // `field_plural_name` is a single token tree - either the ident from a `#[plural(...)]`
    // attribute, or `[<field_name s>]`, which `paste!` concatenates to e.g. `parent_ids`.
    // (`field_plural_name_mut` is the same with `_mut` appended.)
    //
    // The table's generics are lifetimes (optional), then the key, then the item type.
    //
    // The key is matched structurally - `$key_name:ident` plus optional lifetime
    // arguments - not as one `:ty`. After the optional lifetime list, the matcher must
    // decide "another lifetime, or the key?" one token ahead, which is only allowed
    // between single-token fragments - a `ty` fragment there is a `local ambiguity`
    // error. The pieces are reassembled into `config`'s `key` slot, which `@normalize`
    // re-matches as a single `:ty` fragment (legal there, with nothing repeatable in
    // front of it), so the rest of the macro handles the key as one fragment.
    // `$key_name` is also recorded on its own, as the `key_name` slot - `paste!` derives
    // the `$id` method-argument name from it, which it could not do from a type.
    //
    // `$struct_ty` is `:ty`, matched by the real Rust parser, so `Thing<'a, 'b>>;` works -
    // the parser consumes the struct type's half of the trailing `>>` token and leaves the
    // other half for the matcher's closing `>`. (Token-by-token matchers cannot do this -
    // a literal `>` in a pattern does not match half of a `>>`.)
    (
        $(# [ $($table_attr:tt)* ])*
        $vis:vis table $table_name:ident<
            $( $($table_lifetime:lifetime,)+ )?
            $key_name:ident $(< $($key_lifetime:lifetime),+ $(,)? >)?,
            $struct_ty:ty
        >;

        $(#[$struct_attr:meta])*
        $struct_vis:vis struct $struct_name:ident $(< $($struct_lifetime:lifetime),+ $(,)? >)? {
            $($fields:tt)*
        }
    ) => {
        // The `struct`'s visibility must match the `table`'s - the macro applies one
        // visibility to every generated type, so the two declarations have to agree.
        // Checked here rather than in the matcher because `macro_rules!` cannot compare
        // two captured `:vis` fragments for equality - so the check is a compile-time
        // assertion over their `stringify!`d forms. `const_str_eq` is used because `str`'s
        // `==` is not `const`. Both visibilities go through the same `stringify!`, which has
        // already discarded whitespace (`pub(crate)` and `pub (crate)` stringify alike), so
        // the textual comparison is exact. Emitted alongside the recursive call, so it runs
        // exactly once per invocation and needs no plumbing through the internal rules.
        const _: () = ::std::assert!(
            $crate::str::const_str_eq(
                ::std::stringify!($vis),
                ::std::stringify!($struct_vis),
            ),
            "the `struct` visibility must match the `table` visibility \
             (both `pub`, both `pub(crate)`, both private, etc)",
        );

        $crate::multi_vec! {
            @table_attrs
            config = {
                key_ty = [ $key_name $(< $($key_lifetime),+ >)? ]
                key_name = [ $key_name ]
                vis = [ $vis ]
                table_name = [ $table_name ]
                table_lifetimes = [ $( $($table_lifetime),+ )? ]
                table_attrs = []
                struct_ty = [ $struct_ty ]
                struct_name = [ $struct_name ]
                struct_lifetimes = [ $( $($struct_lifetime),+ )? ]
                struct_attrs = [ $(#[$struct_attr])* ]
                clone = []
                debug = []
            }
            pending = [ $([ $($table_attr)* ])* ]
            fields = []
            rest = [ $($fields)* ]
        }
    };

    // Classify a `#[derive(Clone)]` on the `table` declaration.
    //
    // Record it in the `clone` slot, and do NOT pass it through to `table_attrs`.
    //
    // A full slot makes `@generate` emit `#[derive(Clone)]` explicitly on the table type
    // and the struct, and makes the `@generate_clone` rules generate the clone machinery
    // (a `CloneFields` impl).
    //
    // The table's derive resolves exactly when that machinery exists -
    // `MultiVec` implements `Clone` where the fields struct implements `CloneFields`.
    (
        @table_attrs
        config = {
            key_ty = $key_ty:tt
            key_name = $key_name:tt
            vis = $vis:tt
            table_name = $table_name:tt
            table_lifetimes = $table_lifetimes:tt
            table_attrs = $table_attrs:tt
            struct_ty = $struct_ty:tt
            struct_name = $struct_name:tt
            struct_lifetimes = $struct_lifetimes:tt
            struct_attrs = $struct_attrs:tt
            clone = []
            debug = $debug:tt
        }
        pending = [
            [ derive ( Clone $(,)? ) ]
            $($pending:tt)*
        ]
        fields = $fields:tt
        rest = $rest:tt
    ) => {
        $crate::multi_vec! {
            @table_attrs
            config = {
                key_ty = $key_ty
                key_name = $key_name
                vis = $vis
                table_name = $table_name
                table_lifetimes = $table_lifetimes
                table_attrs = $table_attrs
                struct_ty = $struct_ty
                struct_name = $struct_name
                struct_lifetimes = $struct_lifetimes
                struct_attrs = $struct_attrs
                clone = [ Clone ]
                debug = $debug
            }
            pending = [ $($pending)* ]
            fields = $fields
            rest = $rest
        }
    };

    // A 2nd `#[derive(Clone)]` on the `table` (the `clone` slot is already full) - reject.
    //
    // A real derive would get rustc's duplicate-impl error. The slot must produce the equivalent.
    (
        @table_attrs
        config = {
            key_ty = $_key_ty:tt
            key_name = $_key_name:tt
            vis = $_vis:tt
            table_name = $_table_name:tt
            table_lifetimes = $_table_lifetimes:tt
            table_attrs = $_table_attrs:tt
            struct_ty = $_struct_ty:tt
            struct_name = $_struct_name:tt
            struct_lifetimes = $_struct_lifetimes:tt
            struct_attrs = $_struct_attrs:tt
            clone = [ $_clone:ident ]
            debug = $_debug:tt
        }
        pending = [
            [ derive ( Clone $(,)? ) ]
            $($_pending:tt)*
        ]
        fields = $_fields:tt
        rest = $_rest:tt
    ) => {
        ::std::compile_error!("duplicate `#[derive(Clone)]` attribute");
    };

    // Classify a `#[derive(Debug)]` on the `table` declaration.
    //
    // Record it in the `debug` slot, and do NOT pass it through.
    // It cannot be a real derive - the struct's and view types' `Debug` derives must also
    // be conditional on it, which a derive on the table type alone could not achieve.
    //
    // A full slot makes `@generate` emit `#[derive(Debug)]` on the struct and view types,
    // and the table's `Debug` impl.
    (
        @table_attrs
        config = {
            key_ty = $key_ty:tt
            key_name = $key_name:tt
            vis = $vis:tt
            table_name = $table_name:tt
            table_lifetimes = $table_lifetimes:tt
            table_attrs = $table_attrs:tt
            struct_ty = $struct_ty:tt
            struct_name = $struct_name:tt
            struct_lifetimes = $struct_lifetimes:tt
            struct_attrs = $struct_attrs:tt
            clone = $clone:tt
            debug = []
        }
        pending = [
            [ derive ( Debug $(,)? ) ]
            $($pending:tt)*
        ]
        fields = $fields:tt
        rest = $rest:tt
    ) => {
        $crate::multi_vec! {
            @table_attrs
            config = {
                key_ty = $key_ty
                key_name = $key_name
                vis = $vis
                table_name = $table_name
                table_lifetimes = $table_lifetimes
                table_attrs = $table_attrs
                struct_ty = $struct_ty
                struct_name = $struct_name
                struct_lifetimes = $struct_lifetimes
                struct_attrs = $struct_attrs
                clone = $clone
                debug = [ Debug ]
            }
            pending = [ $($pending)* ]
            fields = $fields
            rest = $rest
        }
    };

    // A 2nd `#[derive(Debug)]` on the `table` (the `debug` slot is already full) - reject.
    //
    // A real derive would get rustc's duplicate-impl error. The slot must produce the equivalent.
    (
        @table_attrs
        config = {
            key_ty = $_key_ty:tt
            key_name = $_key_name:tt
            vis = $_vis:tt
            table_name = $_table_name:tt
            table_lifetimes = $_table_lifetimes:tt
            table_attrs = $_table_attrs:tt
            struct_ty = $_struct_ty:tt
            struct_name = $_struct_name:tt
            struct_lifetimes = $_struct_lifetimes:tt
            struct_attrs = $_struct_attrs:tt
            clone = $_clone:tt
            debug = [ $_debug:ident ]
        }
        pending = [
            [ derive ( Debug $(,)? ) ]
            $($_pending:tt)*
        ]
        fields = $_fields:tt
        rest = $_rest:tt
    ) => {
        ::std::compile_error!("duplicate `#[derive(Debug)]` attribute");
    };

    // A `#[derive(...)]` with more than one derive (e.g. `#[derive(Clone, Debug)]`) -
    // split off the first derive into its own attribute, and re-queue both halves, so
    // the rules above and below classify each derive separately.
    //
    // `$first:ident`, not `:path`: an `ident` capture stays transparent, so the re-queued
    // `[ derive ( $first ) ]` still matches the literal `Clone` / `Debug` in the rules
    // above. (A `:path` capture would become an opaque fragment, which they cannot match.
    // Path derives like `serde::Serialize` fall through to the rule below and are
    // rejected - only `Clone` and `Debug` are accepted anyway.)
    (
        @table_attrs
        config = $config:tt
        pending = [
            [ derive ( $first:ident , $($more:tt)+ ) ]
            $($pending:tt)*
        ]
        fields = $fields:tt
        rest = $rest:tt
    ) => {
        $crate::multi_vec! {
            @table_attrs
            config = $config
            pending = [
                [ derive ( $first ) ]
                [ derive ( $($more)+ ) ]
                $($pending)*
            ]
            fields = $fields
            rest = $rest
        }
    };

    // Any other derive on the `table` - reject. The table type wraps a `MultiVec`, which
    // supports only `Clone` and `Debug`.
    // (Must come after the `Clone` / `Debug` rules and the splitting rule above, which
    // this rule would also match.)
    (
        @table_attrs
        config = $_config:tt
        pending = [
            [ derive ( $($derives:tt)* ) ]
            $($_pending:tt)*
        ]
        fields = $_fields:tt
        rest = $_rest:tt
    ) => {
        ::std::compile_error!(::std::concat!(
            "`multi_vec!` tables accept only `#[derive(Clone)]` and `#[derive(Debug)]`, \
             not `#[derive(",
            ::std::stringify!($($derives)*),
            ")]`",
        ));
    };

    // Any other attribute on the `table` (doc comments etc.) - pass it through to the
    // generated table type.
    // (Must come after the derive rules above: this rule would also match a derive.)
    (
        @table_attrs
        config = {
            key_ty = $key_ty:tt
            key_name = $key_name:tt
            vis = $vis:tt
            table_name = $table_name:tt
            table_lifetimes = $table_lifetimes:tt
            table_attrs = [ $($table_attrs:tt)* ]
            struct_ty = $struct_ty:tt
            struct_name = $struct_name:tt
            struct_lifetimes = $struct_lifetimes:tt
            struct_attrs = $struct_attrs:tt
            clone = $clone:tt
            debug = $debug:tt
        }
        pending = [
            [ $($attr:tt)* ]
            $($pending:tt)*
        ]
        fields = $fields:tt
        rest = $rest:tt
    ) => {
        $crate::multi_vec! {
            @table_attrs
            config = {
                key_ty = $key_ty
                key_name = $key_name
                vis = $vis
                table_name = $table_name
                table_lifetimes = $table_lifetimes
                table_attrs = [ $($table_attrs)* #[$($attr)*] ]
                struct_ty = $struct_ty
                struct_name = $struct_name
                struct_lifetimes = $struct_lifetimes
                struct_attrs = $struct_attrs
                clone = $clone
                debug = $debug
            }
            pending = [ $($pending)* ]
            fields = $fields
            rest = $rest
        }
    };

    // All table attributes classified - start normalizing the fields.
    (
        @table_attrs
        config = $config:tt
        pending = []
        fields = $fields:tt
        rest = $rest:tt
    ) => {
        $crate::multi_vec! {
            @normalize
            config = $config
            fields = $fields
            rest = $rest
        }
    };

    // Start normalizing the next field: build its normalized record up front, in the
    // exact form `@generate` consumes, then refine it in place as the `@field`
    // rules classify the attributes.
    //
    // `ctx` bundles the state which the classify rules carry through unexamined.
    // `field_plural_name` starts out empty, meaning "no `#[plural(...)]` seen yet";
    // if it is still empty once all attributes are classified, a dedicated rule fills
    // in the default plural name (append `s`).
    (
        @normalize
        config = $config:tt
        fields = $fields:tt
        rest = [
            $(# [ $($field_attr:tt)* ])*
            $field_vis:vis $field_name:ident: $field_ty:ty
            $(, $($rest:tt)*)?
        ]
    ) => {
        $crate::multi_vec! {
            @field
            ctx = {
                config = $config
                fields = $fields
                rest = [ $($($rest)*)? ]
            }
            field = {
                field_vis = [ $field_vis ]
                field_name = [ $field_name ]
                field_ty = [ $field_ty ]
                field_plural_name = []
                field_plural_name_mut = []
                field_comments = []
            }
            pending = [ $([ $($field_attr)* ])* ]
        }
    };

    // Classify a doc comment (`#[doc ...]`): collect it into `field_comments`, which
    // `@generate` emits onto the struct's field.
    (
        @field
        ctx = $ctx:tt
        field = {
            field_vis = $field_vis:tt
            field_name = $field_name:tt
            field_ty = $field_ty:tt
            field_plural_name = $field_plural_name:tt
            field_plural_name_mut = $field_plural_name_mut:tt
            field_comments = [ $($field_comments:tt)* ]
        }
        pending = [
            [ doc $($body:tt)* ]
            $($pending:tt)*
        ]
    ) => {
        $crate::multi_vec! {
            @field
            ctx = $ctx
            field = {
                field_vis = $field_vis
                field_name = $field_name
                field_ty = $field_ty
                field_plural_name = $field_plural_name
                field_plural_name_mut = $field_plural_name_mut
                field_comments = [
                    $($field_comments)*
                    #[doc $($body)*]
                ]
            }
            pending = [ $($pending)* ]
        }
    };

    // Classify a `#[plural(...)]` attribute: set the plural name.
    (
        @field
        ctx = $ctx:tt
        field = {
            field_vis = $field_vis:tt
            field_name = $field_name:tt
            field_ty = $field_ty:tt
            field_plural_name = []
            field_plural_name_mut = []
            field_comments = $field_comments:tt
        }
        pending = [
            [ plural ( $field_plural_name:ident ) ]
            $($pending:tt)*
        ]
    ) => {
        $crate::multi_vec! {
            @field
            ctx = $ctx
            field = {
                field_vis = $field_vis
                field_name = $field_name
                field_ty = $field_ty
                field_plural_name = [ $field_plural_name ]
                field_plural_name_mut = [ [<$field_plural_name _mut>] ]
                field_comments = $field_comments
            }
            pending = [ $($pending)* ]
        }
    };

    // A 2nd `#[plural(...)]` on the same field (`field_plural_name` is already full) - reject.
    (
        @field
        ctx = $_ctx:tt
        field = {
            field_vis = $_field_vis:tt
            field_name = $_field_name:tt
            field_ty = $_field_ty:tt
            field_plural_name = [ $_field_plural_name:tt ]
            field_plural_name_mut = $_field_plural_name_mut:tt
            field_comments = $_field_comments:tt
        }
        pending = [
            [ plural $($_body:tt)* ]
            $($_pending:tt)*
        ]
    ) => {
        ::std::compile_error!("duplicate `#[plural(...)]` attribute");
    };

    // A malformed `#[plural ...]` (not a single identifier in parentheses) - reject.
    // (A well-formed `#[plural(...)]` was already consumed by the rules above.)
    (
        @field
        ctx = $_ctx:tt
        field = $_field:tt
        pending = [
            [ plural $($_body:tt)* ]
            $($_pending:tt)*
        ]
    ) => {
        ::std::compile_error!(
            "`#[plural(...)]` expects a single identifier, e.g. `#[plural(all_flags)]`"
        );
    };

    // Any other attribute - reject. It would apply only to the generated struct's field,
    // not to the fields of the generated view types, which is a footgun (`#[cfg(...)]`
    // especially so).
    (
        @field
        ctx = $_ctx:tt
        field = $_field:tt
        pending = [
            [ $($body:tt)* ]
            $($_pending:tt)*
        ]
    ) => {
        ::std::compile_error!(::std::concat!(
            "`multi_vec!` fields accept only doc comments and `#[plural(...)]` on struct fields, not `#[",
            ::std::stringify!($($body)*),
            "]`",
        ));
    };

    // All attributes classified, but no `#[plural(...)]` was given - fill in the default
    // plural name (append `s`), and re-run, so the rule below pushes the field.
    (
        @field
        ctx = $ctx:tt
        field = {
            field_vis = $field_vis:tt
            field_name = [ $field_name:ident ]
            field_ty = $field_ty:tt
            field_plural_name = []
            field_plural_name_mut = []
            field_comments = $field_comments:tt
        }
        pending = []
    ) => {
        $crate::multi_vec! {
            @field
            ctx = $ctx
            field = {
                field_vis = $field_vis
                field_name = [ $field_name ]
                field_ty = $field_ty
                field_plural_name = [ [<$field_name s>] ]
                field_plural_name_mut = [ [<$field_name s_mut>] ]
                field_comments = $field_comments
            }
            pending = []
        }
    };

    // All attributes classified - push the completed record, and hand the remaining
    // fields back to `@normalize`.
    //
    // Must come after the fill rule above: this rule would also match a record whose
    // `field_plural_name` is still empty, so such records must be filled first.
    (
        @field
        ctx = {
            config = $config:tt
            fields = [ $($fields:tt)* ]
            rest = $rest:tt
        }
        field = $field:tt
        pending = []
    ) => {
        $crate::multi_vec! {
            @normalize
            config = $config
            fields = [
                $($fields)*
                $field
            ]
            rest = $rest
        }
    };

    // All fields normalized, but there are none - reject with a clear error.
    // (Without this rule, an empty field list fails to match any rule, producing an
    // inscrutable macro-expansion error.)
    (
        @normalize
        config = $_config:tt
        fields = []
        rest = []
    ) => {
        ::std::compile_error!("`multi_vec!` requires at least one field");
    };

    // All fields normalized - build the derived config slots, and generate the output.
    // (Must come after the empty-fields rule above, which would also match here.)
    //
    // The lifetime lists are precompiled here into the token groups `@generate` emits
    // (see the slot comments on the `@generate` rule), so the nested-optional shape -
    // `$(< $($table_lifetime),+ >)?` etc., emitting nothing when the list is empty -
    // is written once per form here, instead of at every one of `@generate`'s many
    // use sites.
    (
        @normalize
        config = {
            // The key's tokens, re-matched as one `:ty` fragment - see the key note on
            // the entry rule.
            key_ty = [ $key_ty:ty ]
            key_name = [ $key_name:ident ]
            vis = $vis:tt
            table_name = [ $table_name:ident ]
            table_lifetimes = [ $( $($table_lifetime:lifetime),+ )? ]
            table_attrs = $table_attrs:tt
            struct_ty = $struct_ty:tt
            struct_name = [ $struct_name:ident ]
            struct_lifetimes = [ $( $($struct_lifetime:lifetime),+ )? ]
            struct_attrs = $struct_attrs:tt
            clone = $clone:tt
            debug = $debug:tt
        }
        fields = $fields:tt
        rest = []
    ) => {
        $crate::multi_vec! {
            @generate
            config = {
                key_ty = [ $key_ty ]
                key_name = [ $key_name ]
                id = [ [<$key_name:snake>] ]
                vis = $vis
                table_name = [ $table_name ]
                table_lifetimes = [ $( $($table_lifetime),+ )? ]
                table_generics = [ $(< $($table_lifetime),+ >)? ]
                table_ref_generics = [ < 'v $(, $($table_lifetime),+ )? > ]
                table_attrs = $table_attrs
                struct_ty = $struct_ty
                struct_name = [ $struct_name ]
                struct_generics = [ $(< $($struct_lifetime),+ >)? ]
                struct_ref_generics = [ < 'v $(, $($struct_lifetime),+ )? > ]
                struct_attrs = $struct_attrs
                // The two reference-view types, name and generics together.
                struct_ref_ty = [ [<$struct_name Ref>] < 'v $(, $($struct_lifetime),+ )? > ]
                struct_mut_ty = [ [<$struct_name Mut>] < 'v $(, $($struct_lifetime),+ )? > ]
                // The two slice-view types. Unlike the reference views, these take the
                // table's lifetimes: their fields name the key (`IndexSlice<$key_ty, ...>`).
                struct_slices_ty = [ [<$struct_name Slices>] < 'v $(, $($table_lifetime),+ )? > ]
                struct_slices_mut_ty = [ [<$struct_name SlicesMut>] < 'v $(, $($table_lifetime),+ )? > ]
                // The three iterator types, name and generics together (the wrappers take
                // the struct's lifetimes). `paste!` concatenates the `[<...>]` in `@generate`.
                table_iter_ty = [ [<$table_name Iter>] < 'v $(, $($struct_lifetime),+ )? > ]
                table_iter_mut_ty = [ [<$table_name IterMut>] < 'v $(, $($struct_lifetime),+ )? > ]
                table_into_iter_ty = [ [<$table_name IntoIter>] $(< $($struct_lifetime),+ >)? ]
                clone = $clone
                debug = $debug
            }
            fields = $fields
        }
    };

    // Generate the output: the struct, the table type, the view types, the iterators, and all their impls.
    //
    // Lifetime plumbing: the `*_generics` / `*_lifetimes*` slots are precompiled token groups
    // `@normalize` builds from the two declarations' lifetime lists (see the slot comments
    // below). Each is empty - emitting nothing - when the corresponding list is empty.
    //
    // Which type takes which lifetimes: the table type takes the table's, the struct and the
    // reference views / iterators take the struct's (as do the `Fields` / `CloneFields` impls),
    // and the slice views take the table's (as does the `SliceFields` impl). The item type
    // (`$struct_ty`, e.g. `Thing<'a>`) is how the table instantiates the struct's lifetimes
    // from its own.
    //
    // The two declarations must use the same lifetime names
    // (`assert_item_type_is_the_struct` below rejects a mismatch), so the field types -
    // written with the struct's names - can appear directly in the table impl's
    // per-field methods.
    //
    // Comments below show the content/expansion for this example invocation:
    //
    // ```
    // multi_vec! {
    //     /// Lots of things.
    //     #[derive(Clone, Debug)]
    //     pub table Things<'k, 'a, ThingId<'k>, Thing<'a>>;
    //
    //     /// The thing itself.
    //     #[derive(Clone, Debug)]
    //     pub struct Thing<'a> {
    //         /// ID of the thing
    //         pub(crate) id: u32,
    //         /// Name of the thing
    //         pub name: &'a str,
    //     }
    // }
    // ```
    (
        @generate
        config = {
            // `ThingId<'k>`
            key_ty = [ $key_ty:ty ]
            // `ThingId`
            key_name = [ $key_name:ident ]
            // `thing_id`
            id = [ $id:tt ]
            // `pub`
            vis = [ $vis:vis ]

            // `Things`
            table_name = [ $table_name:ident ]
            // `'k, 'a` (bare - the `use<...>` capture list)
            table_lifetimes = [ $($table_lifetimes:tt)* ]
            // `<'k, 'a>` (the table type, its `impl` binders, and `IntoIter`)
            table_generics = [ $($table_generics:tt)* ]
            // `<'v, 'k, 'a>` (the views, iterators, and borrowing methods)
            table_ref_generics = [ $($table_ref_generics:tt)* ]
            // `/// Lots of things.` `#[derive(Clone)]`
            table_attrs = [ $(#[$table_attr:meta])* ]

            // `Thing<'a>`
            struct_ty = [ $struct_ty:ty ]
            // `Thing`
            struct_name = [ $struct_name:ident ]
            // `<'a>`
            struct_generics = [ $($struct_generics:tt)* ]
            // `<'v, 'a>`
            struct_ref_generics = [ $($struct_ref_generics:tt)* ]
            // `/// The thing itself.` `#[derive(Clone, Debug)]`
            struct_attrs = [ $(#[$struct_attr:meta])* ]

            // `ThingRef<'v, 'a>`
            struct_ref_ty = [ $($struct_ref_ty:tt)* ]
            // `ThingMut<'v, 'a>`
            struct_mut_ty = [ $($struct_mut_ty:tt)* ]
            // `ThingSlices<'v, 'k, 'a>` (the slice views take the table's lifetimes)
            struct_slices_ty = [ $($struct_slices_ty:tt)* ]
            // `ThingSlicesMut<'v, 'k, 'a>`
            struct_slices_mut_ty = [ $($struct_slices_mut_ty:tt)* ]

            // `ThingsIter<'v, 'a>`
            table_iter_ty = [ $($table_iter_ty:tt)* ]
            // `ThingsIterMut<'v, 'a>`
            table_iter_mut_ty = [ $($table_iter_mut_ty:tt)* ]
            // `ThingsIntoIter<'a>`
            table_into_iter_ty = [ $($table_into_iter_ty:tt)* ]

            // `Clone` (empty if not derived - derived on the struct, drives `@generate_clone`)
            clone = [ $($clone:ident)? ]
            // `Debug` (empty if not derived - derived on the struct and view types, drives `@generate_debug`)
            debug = [ $($debug:ident)? ]
        }
        fields = [ $(
            {
                // `pub`
                field_vis = [ $field_vis:vis ]
                // `name`
                field_name = [ $field_name:ident ]
                // `&'a str`
                field_ty = [ $field_ty:ty ]
                // `[<name s>]` (`paste!` converts to `names`)
                field_plural_name = [ $field_plural_name:tt ]
                // `[<name s_mut>]` (`paste!` converts to `names_mut`)
                field_plural_name_mut = [ $field_plural_name_mut:tt ]
                // `/// Name of the thing`
                field_comments = [ $(#[$field_comment:meta])* ]
            }
        )+ ]
    ) => {
        $crate::multi_vec::__private::paste! {
            // The table's `#[derive(Clone)]` is emitted here from the `clone` slot, not passed
            // through with its other attributes. It works as a real derive  - `MultiVec`
            // implements `Clone` where the fields struct implements `CloneFields`, which
            // `@generate_clone` implements exactly when the same slot is full.
            //
            // No `#[derive(Debug)]` equivalent here. A derive would print the wrapper -
            // `Things { inner: MultiVec { len: 2, .. } }` - not the elements.
            // The table's `Debug` impl is instead written by hand in `@generate_debug`,
            // printing the elements as a map.
            //
            // ```
            // /// Lots of things.
            // #[derive(Clone)]
            // #[derive(Default)]
            // pub struct Things<'k, 'a> {
            //     inner: MultiVec<ThingId<'k>, Thing<'a>>,
            // }
            // ```
            $(#[$table_attr])*
            $(#[derive($clone)])?
            #[derive(Default)]
            #[allow(dead_code, clippy::allow_attributes)]
            $vis struct $table_name $($table_generics)* {
                inner: $crate::multi_vec::__private::MultiVec<$key_ty, $struct_ty>,
            }

            // The struct's `Clone` / `Debug` derives are emitted by the macro itself, from the
            // `table` declaration's derives - the user must not repeat them on the struct.
            // See "Cloning" / "Debug" in the macro docs.
            //
            // ```
            // /// The thing itself.
            // #[derive(Clone)]
            // #[derive(Debug)]
            // pub struct Thing<'a> {
            //     /// ID of the thing
            //     pub(crate) id: u32,
            //     /// Name of the thing
            //     pub name: &'a str,
            // }
            // ```
            $(#[$struct_attr])*
            $(#[derive($clone)])?
            $(#[derive($debug)])?
            #[allow(dead_code, clippy::allow_attributes)]
            $vis struct $struct_name $($struct_generics)* {
                $(
                    $(#[$field_comment])*
                    $field_vis $field_name: $field_ty,
                )+
            }

            // ```
            // /// References to one element of a [`Things`].
            // ///
            // /// Returned by [`Things::get`].
            // #[derive(Debug)]
            // pub struct ThingRef<'v, 'a> {
            //     /// ID of the thing
            //     pub(crate) id: &'v u32,
            //     /// Name of the thing
            //     pub name: &'v &'a str,
            // }
            // ```
            #[doc = concat!("References to one element of a [`", stringify!($table_name), "`].")]
            #[doc = ""]
            #[doc = concat!("Returned by [`", stringify!($table_name), "::get`].")]
            $(#[derive($debug)])?
            #[allow(dead_code, clippy::allow_attributes)]
            $vis struct $($struct_ref_ty)* {
                $(
                    $(#[$field_comment])*
                    $field_vis $field_name: &'v $field_ty,
                )+
            }

            // ```
            // /// Mutable references to one element of a [`Things`].
            // ///
            // /// Returned by [`Things::get_mut`].
            // #[derive(Debug)]
            // pub struct ThingMut<'v, 'a> {
            //     /// ID of the thing
            //     pub(crate) id: &'v mut u32,
            //     /// Name of the thing
            //     pub name: &'v mut &'a str,
            // }
            // ```
            #[doc = concat!("Mutable references to one element of a [`", stringify!($table_name), "`].")]
            #[doc = ""]
            #[doc = concat!("Returned by [`", stringify!($table_name), "::get_mut`].")]
            $(#[derive($debug)])?
            #[allow(dead_code, clippy::allow_attributes)]
            $vis struct $($struct_mut_ty)* {
                $(
                    $(#[$field_comment])*
                    $field_vis $field_name: &'v mut $field_ty,
                )+
            }

            // ```
            // /// Slices over each field array of a [`Things`].
            // ///
            // /// Returned by [`Things::slices`].
            // #[derive(Debug)]
            // pub struct ThingSlices<'v, 'k, 'a> {
            //     pub(crate) ids: &'v IndexSlice<ThingId<'k>, [u32]>,
            //     pub names: &'v IndexSlice<ThingId<'k>, [&'a str]>,
            // }
            // ```
            #[doc = concat!("Slices over each field array of a [`", stringify!($table_name), "`].")]
            #[doc = ""]
            #[doc = concat!("Returned by [`", stringify!($table_name), "::slices`].")]
            $(#[derive($debug)])?
            #[allow(dead_code, clippy::allow_attributes)]
            $vis struct $($struct_slices_ty)* {
                $(
                    $field_vis $field_plural_name: &'v $crate::multi_vec::__private::IndexSlice<$key_ty, [$field_ty]>,
                )+
            }

            // ```
            // /// Mutable slices over each field array of a [`Things`].
            // ///
            // /// Returned by [`Things::slices_mut`].
            // #[derive(Debug)]
            // pub struct ThingSlicesMut<'v, 'k, 'a> {
            //     pub(crate) ids: &'v mut IndexSlice<ThingId<'k>, [u32]>,
            //     pub names: &'v mut IndexSlice<ThingId<'k>, [&'a str]>,
            // }
            // ```
            #[doc = concat!("Mutable slices over each field array of a [`", stringify!($table_name), "`].")]
            #[doc = ""]
            #[doc = concat!("Returned by [`", stringify!($table_name), "::slices_mut`].")]
            $(#[derive($debug)])?
            #[allow(dead_code, clippy::allow_attributes)]
            $vis struct $($struct_slices_mut_ty)* {
                $(
                    $field_vis $field_plural_name: &'v mut $crate::multi_vec::__private::IndexSlice<$key_ty, [$field_ty]>,
                )+
            }

            // ```
            // /// Iterator over a [`Things`]'s elements, yielding a [`ThingRef`] for each.
            // ///
            // /// Returned by [`Things::iter`].
            // #[must_use = "iterators are lazy and do nothing unless consumed"]
            // pub struct ThingsIter<'v, 'a>(Iter<'v, Thing<'a>>);
            // ```
            #[doc = concat!("Iterator over a [`", stringify!($table_name), "`]'s elements, yielding a [`", stringify!($struct_name), "Ref`] for each.")]
            #[doc = ""]
            #[doc = concat!("Returned by [`", stringify!($table_name), "::iter`].")]
            #[must_use = "iterators are lazy and do nothing unless consumed"]
            #[allow(dead_code, clippy::allow_attributes)]
            $vis struct $($table_iter_ty)* (
                $crate::multi_vec::__private::Iter<'v, $struct_ty>,
            );

            // ```
            // /// Iterator over a [`Things`]'s elements, yielding a [`ThingMut`] for each.
            // ///
            // /// Returned by [`Things::iter_mut`].
            // #[must_use = "iterators are lazy and do nothing unless consumed"]
            // pub struct ThingsIterMut<'v, 'a>(IterMut<'v, Thing<'a>>);
            // ```
            #[doc = concat!("Iterator over a [`", stringify!($table_name), "`]'s elements, yielding a [`", stringify!($struct_name), "Mut`] for each.")]
            #[doc = ""]
            #[doc = concat!("Returned by [`", stringify!($table_name), "::iter_mut`].")]
            #[must_use = "iterators are lazy and do nothing unless consumed"]
            #[allow(dead_code, clippy::allow_attributes)]
            $vis struct $($table_iter_mut_ty)* (
                $crate::multi_vec::__private::IterMut<'v, $struct_ty>,
            );

            // ```
            // /// Iterator over a [`Things`]'s elements, yielding each element as an owned [`Thing`].
            // ///
            // /// Returned by [`Things`]'s [`IntoIterator`] impl.
            // #[must_use = "iterators are lazy and do nothing unless consumed"]
            // pub struct ThingsIntoIter<'a>(IntoIter<Thing<'a>>);
            // ```
            #[doc = concat!("Iterator over a [`", stringify!($table_name), "`]'s elements, yielding each element as an owned [`", stringify!($struct_name), "`].")]
            #[doc = ""]
            #[doc = concat!("Returned by [`", stringify!($table_name), "`]'s [`IntoIterator`] impl.")]
            #[must_use = "iterators are lazy and do nothing unless consumed"]
            #[allow(dead_code, clippy::allow_attributes)]
            $vis struct $($table_into_iter_ty)* (
                $crate::multi_vec::__private::IntoIter<$struct_ty>,
            );

            // All the impls, and the items only they need (imports, `FIELD_COUNT`,
            // the compile-time assertions, and the type aliases), live in this anonymous
            // `const` block. It keeps them out of the invoking scope, while the impls
            // still apply to the types above as normal (and rustdoc still documents them).
            // Only the types themselves must be defined outside it, to be nameable by the user.
            #[allow(
                dead_code,
                forgetting_copy_types,
                private_interfaces,
                clippy::extra_unused_lifetimes,
                clippy::inline_always,
                clippy::macro_metavars_in_unsafe,
                clippy::undocumented_unsafe_blocks,
                clippy::allow_attributes
            )]
            const _: () = {
                use ::std::{iter::FusedIterator, ptr::NonNull};

                use $crate::multi_vec::__private as __p;

                // Number of fields in the struct
                const FIELD_COUNT: usize = [$(stringify!($field_name)),+].len();

                // Verify that the item type named in the `table` declaration is the struct
                // declared below it, with the struct's own lifetimes applied. (Without this
                // check, a `table` declaration naming a *different* struct which also
                // implements `Fields` - e.g. another table's struct - would compile,
                // silently binding this table to the wrong element type.)
                //
                // Never called. A `*mut` pointer is invariant in its pointee, so returning
                // `item_ptr` requires the two types to be *exactly* equal, lifetimes
                // included. A wrong struct name fails with a type mismatch (E0308), a wrong
                // number of lifetimes with E0107, elided lifetimes with E0621, and swapped /
                // repeated / `'static` lifetime applications fail borrow checking
                // ("lifetime may not live long enough"). The fn declares the *struct's*
                // lifetime names, so a `table` declaration whose item type uses different
                // names fails with E0261 (undeclared lifetime) - the two declarations must
                // use the same names.
                //
                // ```
                // fn assert_item_type_is_the_struct<'a>(item_ptr: *mut Thing<'a>) -> *mut Thing<'a> {
                //     item_ptr
                // }
                // ```
                fn assert_item_type_is_the_struct$($struct_generics)*(
                    item_ptr: *mut $struct_ty,
                ) -> *mut $struct_name$($struct_generics)* {
                    item_ptr
                }

                // `impl<'k, 'a> Things<'k, 'a> {`
                impl$($table_generics)* $table_name$($table_generics)* {
                    #[doc = concat!("Maximum capacity of a `", stringify!($table_name), "`.")]
                    ///
                    /// Capacity is limited by the index type's range, and the maximum
                    /// allocation size (`isize::MAX` bytes).
                    pub const MAX_CAPACITY: usize = __p::MultiVec::<$key_ty, $struct_ty>::MAX_CAPACITY;

                    #[doc = concat!("Create a new empty `", stringify!($table_name), "`. Does not allocate.")]
                    #[inline(always)]
                    pub const fn new() -> Self {
                        Self { inner: __p::MultiVec::new() }
                    }

                    #[doc = concat!("Create a new `", stringify!($table_name), "` with capacity for `capacity` elements.")]
                    ///
                    /// Does not allocate if `capacity == 0`.
                    ///
                    /// # Panics
                    ///
                    /// Panics if `capacity` exceeds [`MAX_CAPACITY`].
                    ///
                    /// [`MAX_CAPACITY`]: Self::MAX_CAPACITY
                    #[inline(always)]
                    pub fn with_capacity(capacity: usize) -> Self {
                        Self { inner: __p::MultiVec::with_capacity(capacity) }
                    }

                    /// Returns the number of elements.
                    #[inline(always)]
                    pub fn len(&self) -> usize {
                        self.inner.len()
                    }

                    /// Returns `true` if there are no elements.
                    #[inline(always)]
                    pub fn is_empty(&self) -> bool {
                        self.inner.is_empty()
                    }

                    #[doc = concat!("Push a [`", stringify!($struct_name), "`] to this [`", stringify!($table_name), "`].")]
                    #[doc = concat!("Returns the [`", stringify!($key_name), "`] of the new element.")]
                    ///
                    /// # Panics
                    ///
                    /// Panics if the table is already full to [`MAX_CAPACITY`].
                    ///
                    /// [`MAX_CAPACITY`]: Self::MAX_CAPACITY
                    #[inline(always)]
                    // `pub fn push(&mut self, value: Thing<'a>) -> ThingId<'k> {`
                    pub fn push(&mut self, value: $struct_ty) -> $key_ty {
                        self.inner.push(value)
                    }

                    /// Reserve capacity for at least `additional` more elements.
                    ///
                    /// # Panics
                    ///
                    /// Panics if the required capacity exceeds [`MAX_CAPACITY`].
                    ///
                    /// [`MAX_CAPACITY`]: Self::MAX_CAPACITY
                    #[inline(always)]
                    pub fn reserve(&mut self, additional: usize) {
                        self.inner.reserve(additional);
                    }

                    #[doc = concat!("Get references to the element at `", stringify!($id), "`.")]
                    ///
                    /// # Panics
                    ///
                    #[doc = concat!("Panics if `", stringify!($id), "` is out of bounds.")]
                    #[inline(always)]
                    // `pub fn get<'v>(&'v self, thing_id: ThingId<'k>) -> ThingRef<'v, 'a> {`
                    pub fn get<'v>(&'v self, $id: $key_ty) -> $($struct_ref_ty)* {
                        self.inner.get($id)
                    }

                    #[doc = concat!("Get mutable references to the element at `", stringify!($id), "`.")]
                    ///
                    /// # Panics
                    ///
                    #[doc = concat!("Panics if `", stringify!($id), "` is out of bounds.")]
                    #[inline(always)]
                    // `pub fn get_mut<'v>(&'v mut self, thing_id: ThingId<'k>) -> ThingMut<'v, 'a> {`
                    pub fn get_mut<'v>(&'v mut self, $id: $key_ty) -> $($struct_mut_ty)* {
                        self.inner.get_mut($id)
                    }

                    #[doc = concat!("Get references to the element at `", stringify!($id), "`,")]
                    #[doc = concat!("without checking that `", stringify!($id), "` is in bounds.")]
                    ///
                    /// # SAFETY
                    ///
                    #[doc = concat!("`", stringify!($id), "` must be in bounds - less than [`len`].")]
                    ///
                    /// [`len`]: Self::len
                    #[inline(always)]
                    // `pub unsafe fn get_unchecked<'v>(&'v self, thing_id: ThingId<'k>) -> ThingRef<'v, 'a> {`
                    pub unsafe fn get_unchecked<'v>(&'v self, $id: $key_ty) -> $($struct_ref_ty)* {
                        // SAFETY: Caller guarantees `$id` is in bounds
                        unsafe { self.inner.get_unchecked($id) }
                    }

                    #[doc = concat!("Get mutable references to the element at `", stringify!($id), "`,")]
                    #[doc = concat!("without checking that `", stringify!($id), "` is in bounds.")]
                    ///
                    /// # SAFETY
                    ///
                    #[doc = concat!("`", stringify!($id), "` must be in bounds - less than [`len`].")]
                    ///
                    /// [`len`]: Self::len
                    #[inline(always)]
                    // `pub unsafe fn get_unchecked_mut<'v>(&'v mut self, thing_id: ThingId<'k>) -> ThingMut<'v, 'a> {`
                    pub unsafe fn get_unchecked_mut<'v>(&'v mut self, $id: $key_ty) -> $($struct_mut_ty)* {
                        // SAFETY: Caller guarantees `$id` is in bounds
                        unsafe { self.inner.get_unchecked_mut($id) }
                    }

                    /// Get slices over all field arrays.
                    #[inline(always)]
                    // `pub fn slices<'v>(&'v self) -> ThingSlices<'v, 'k, 'a> {`
                    pub fn slices<'v>(&'v self) -> $($struct_slices_ty)* {
                        self.inner.slices()
                    }

                    /// Get mutable slices over all field arrays.
                    #[inline(always)]
                    // `pub fn slices_mut<'v>(&'v mut self) -> ThingSlicesMut<'v, 'k, 'a> {`
                    pub fn slices_mut<'v>(&'v mut self) -> $($struct_slices_mut_ty)* {
                        self.inner.slices_mut()
                    }

                    /// Iterate over all valid indices.
                    ///
                    /// The returned iterator does not borrow `self` (the `use<...>` clause omits
                    /// the `&self` lifetime, opting out of edition 2024's capture-everything default).
                    /// It snapshots `len`, so the table can be mutated while the iterator lives.
                    /// IDs of elements pushed after the `iter_ids` call are not included.
                    #[inline(always)]
                    // `pub fn iter_ids(&self) -> impl ExactSizeIterator<Item = ThingId<'k>> + FusedIterator + use<'k, 'a> {`
                    pub fn iter_ids(&self) -> impl ExactSizeIterator<Item = $key_ty> + FusedIterator + use<$($table_lifetimes)*> {
                        self.inner.iter_ids()
                    }

                    #[doc = concat!("Iterate over all elements, yielding a [`", stringify!($struct_name), "Ref`] (references to every field) for each.")]
                    #[inline(always)]
                    // ```
                    // pub fn iter<'v>(&'v self) -> ThingsIter<'v, 'a> {
                    //     ThingsIter(self.inner.iter())
                    // }
                    // ```
                    pub fn iter<'v>(&'v self) -> $($table_iter_ty)* {
                        [<$table_name Iter>](self.inner.iter())
                    }

                    #[doc = concat!("Iterate over all elements, yielding a [`", stringify!($struct_name), "Mut`] (mutable references to every field) for each.")]
                    #[inline(always)]
                    // ```
                    // pub fn iter_mut<'v>(&'v mut self) -> ThingsIterMut<'v, 'a> {
                    //     ThingsIterMut(self.inner.iter_mut())
                    // }
                    // ```
                    pub fn iter_mut<'v>(&'v mut self) -> $($table_iter_mut_ty)* {
                        [<$table_name IterMut>](self.inner.iter_mut())
                    }

                    #[doc = concat!("Iterate over all elements, yielding each element's [`", stringify!($key_name), "`] and a [`", stringify!($struct_name), "Ref`].")]
                    #[inline(always)]
                    // `pub fn iter_enumerated<'v>(&'v self) -> impl ExactSizeIterator<Item = (ThingId<'k>, ThingRef<'v, 'a>)> + FusedIterator {`
                    pub fn iter_enumerated<'v>(&'v self) -> impl ExactSizeIterator<Item = ($key_ty, $($struct_ref_ty)*)> + FusedIterator {
                        self.inner.iter_enumerated()
                    }

                    #[doc = concat!("Iterate over all elements, yielding each element's [`", stringify!($key_name), "`] and a [`", stringify!($struct_name), "Mut`].")]
                    #[inline(always)]
                    // `pub fn iter_mut_enumerated<'v>(&'v mut self) -> impl ExactSizeIterator<Item = (ThingId<'k>, ThingMut<'v, 'a>)> + FusedIterator {`
                    pub fn iter_mut_enumerated<'v>(&'v mut self) -> impl ExactSizeIterator<Item = ($key_ty, $($struct_mut_ty)*)> + FusedIterator {
                        self.inner.iter_mut_enumerated()
                    }

                    #[doc = concat!("Consume the table, yielding each element's [`", stringify!($key_name), "`] and the element as an owned [`", stringify!($struct_name), "`].")]
                    #[inline(always)]
                    // `pub fn into_iter_enumerated(self) -> impl ExactSizeIterator<Item = (ThingId<'k>, Thing<'a>)> + FusedIterator {`
                    pub fn into_iter_enumerated(self) -> impl ExactSizeIterator<Item = ($key_ty, $struct_ty)> + FusedIterator {
                        self.inner.into_iter_enumerated()
                    }

                    // Per-field accessor methods e.g. `.parent_id(id)` / `.parent_id_mut(id)`
                    $(
                        #[doc = concat!("Get reference to the `", stringify!($field_name), "` field of the element at `", stringify!($id), "`.")]
                        ///
                        /// # Panics
                        ///
                        #[doc = concat!("Panics if `", stringify!($id), "` is out of bounds.")]
                        #[inline(always)]
                        // `pub fn name(&self, thing_id: ThingId<'k>) -> &&'a str {` (one per field)
                        $field_vis fn $field_name(&self, $id: $key_ty) -> &$field_ty {
                            self.get($id).$field_name
                        }

                        #[doc = concat!("Get mutable reference to the `", stringify!($field_name), "` field of the element at `", stringify!($id), "`.")]
                        ///
                        /// # Panics
                        ///
                        #[doc = concat!("Panics if `", stringify!($id), "` is out of bounds.")]
                        #[inline(always)]
                        // `pub fn name_mut(&mut self, thing_id: ThingId<'k>) -> &mut &'a str {`
                        $field_vis fn [<$field_name _mut>](&mut self, $id: $key_ty) -> &mut $field_ty {
                            self.get_mut($id).$field_name
                        }
                    )+

                    // Per-field slice methods e.g. `.parent_ids()` / `.parent_ids_mut()`
                    $(
                        #[doc = concat!("Get slice of `", stringify!($field_name), "` fields of all elements.")]
                        #[inline(always)]
                        // `pub fn names(&self) -> &IndexSlice<ThingId<'k>, [&'a str]> {`
                        $field_vis fn $field_plural_name(&self) -> &__p::IndexSlice<$key_ty, [$field_ty]> {
                            self.slices().$field_plural_name
                        }

                        #[doc = concat!("Get mutable slice of `", stringify!($field_name), "` fields of all elements.")]
                        #[inline(always)]
                        // `pub fn names_mut(&mut self) -> &mut IndexSlice<ThingId<'k>, [&'a str]> {`
                        $field_vis fn $field_plural_name_mut(&mut self) -> &mut __p::IndexSlice<$key_ty, [$field_ty]> {
                            self.slices_mut().$field_plural_name
                        }
                    )+
                }

                // `impl<'k, 'a> IntoIterator for Things<'k, 'a> {`
                impl$($table_generics)* IntoIterator for $table_name$($table_generics)* {
                    type Item = $struct_ty;
                    type IntoIter = $($table_into_iter_ty)*;

                    #[doc = concat!("Consume the table, yielding each element as an owned [`", stringify!($struct_name), "`], reassembled from its stored fields.")]
                    #[inline(always)]
                    // ```
                    // fn into_iter(self) -> ThingsIntoIter<'a> {
                    //     ThingsIntoIter(self.inner.into_iter())
                    // }
                    // ```
                    fn into_iter(self) -> $($table_into_iter_ty)* {
                        [<$table_name IntoIter>](self.inner.into_iter())
                    }
                }

                // `impl<'v, 'k, 'a> IntoIterator for &'v Things<'k, 'a> {`
                impl$($table_ref_generics)* IntoIterator for &'v $table_name$($table_generics)* {
                    type Item = $($struct_ref_ty)*;
                    type IntoIter = $($table_iter_ty)*;

                    #[inline(always)]
                    // `fn into_iter(self) -> ThingsIter<'v, 'a> {`
                    fn into_iter(self) -> $($table_iter_ty)* {
                        self.iter()
                    }
                }

                // `impl<'v, 'k, 'a> IntoIterator for &'v mut Things<'k, 'a> {`
                impl$($table_ref_generics)* IntoIterator for &'v mut $table_name$($table_generics)* {
                    type Item = $($struct_mut_ty)*;
                    type IntoIter = $($table_iter_mut_ty)*;

                    #[inline(always)]
                    // `fn into_iter(self) -> ThingsIterMut<'v, 'a> {`
                    fn into_iter(self) -> $($table_iter_mut_ty)* {
                        self.iter_mut()
                    }
                }

                // Implement `Debug` on the table type if `#[derive(Debug)]` is present.
                // Prints as a map from ID to element e.g. `{ 0: ScopeRef { ... }, 1: ... }`.
                // `@generate_debug` expands to nothing when `debug` is empty.
                // (A sub-rule, not a `$( ... )?` group gated on the `debug` slot: all
                // metavariables directly under a repetition must repeat the same number
                // of times, so the generics lists cannot appear under a `$debug`-driven
                // repetition - `$debug` repeats once, the lists per lifetime.)
                //
                // Braces, not brackets, delimit the generics slots here (and on
                // `@generate_clone` below): this invocation's tokens pass through the
                // surrounding `paste!`, and a bracketed generics list would start with
                // `[ <`, which `paste!` treats as its own `[< ... >]` paste syntax and
                // mangles.
                $crate::multi_vec! {
                    @generate_debug
                    debug = [ $($debug)? ]
                    table_name = [ $table_name ]
                    table_generics = { $($table_generics)* }
                }

                // The struct's lifetime params (if any) are declared by `field_layouts`, and
                // erased when it is called in `SHAPE`'s initializer (a `Layout` does not
                // depend on lifetimes). `SHAPE` must stay a free const - free consts are
                // evaluated when checked, so `Shape::new`'s compile-time rejection of
                // all-zero-sized field sets fires when the table is *defined*.
                // An associated const is only evaluated when used.
                //
                // ```
                // const fn field_layouts<'a>() -> [Layout; FIELD_COUNT] {
                //     [Layout::new::<u32>(), Layout::new::<&'a str>()]
                // }
                // ```
                const fn field_layouts$($struct_generics)*() -> [__p::Layout; FIELD_COUNT] {
                    [ $( __p::Layout::new::<$field_ty>() ),+ ]
                }
                const SHAPE: __p::Shape<[usize; FIELD_COUNT]> = __p::Shape::new(field_layouts());

                // Implement `Fields` on the struct type (e.g. `Scope`).
                // The methods are a thin layer of type casting - they cast the `NonNull<u8>`
                // pointers to the field types, and read / write / drop / create references
                // through them. This impl binds only the struct's lifetimes
                // (`$struct_generics`), which are all its view types need.
                //
                // `unsafe impl<'a> Fields for Thing<'a> {`
                unsafe impl$($struct_generics)* __p::Fields for $struct_ty {
                    // `type Ref<'v> = ThingRef<'v, 'a> where Self: 'v;`
                    type Ref<'v> = $($struct_ref_ty)* where Self: 'v;

                    // `type Mut<'v> = ThingMut<'v, 'a> where Self: 'v;`
                    type Mut<'v> = $($struct_mut_ty)* where Self: 'v;

                    type Array<T: Copy> = [T; FIELD_COUNT];

                    const SHAPE: __p::Shape<[usize; FIELD_COUNT]> = SHAPE;

                    // ```
                    // unsafe fn write(self, ptrs: [NonNull<u8>; FIELD_COUNT]) {
                    //     let [id, name] = ptrs;
                    //     unsafe {
                    //         id.cast::<u32>().write(self.id);
                    //         name.cast::<&'a str>().write(self.name);
                    //     }
                    // }
                    // ```
                    #[inline]
                    unsafe fn write(self, ptrs: [NonNull<u8>; FIELD_COUNT]) {
                        let [ $($field_name),+ ] = ptrs;
                        unsafe {
                            $(
                                $field_name.cast::<$field_ty>().write(self.$field_name);
                            )+
                        }
                    }

                    // ```
                    // unsafe fn create_owned(ptrs: [NonNull<u8>; FIELD_COUNT]) -> Self {
                    //     let [id, name] = ptrs;
                    //     unsafe {
                    //         Self {
                    //             id: id.cast::<u32>().read(),
                    //             name: name.cast::<&'a str>().read(),
                    //         }
                    //     }
                    // }
                    // ```
                    #[inline]
                    unsafe fn create_owned(ptrs: [NonNull<u8>; FIELD_COUNT]) -> Self {
                        let [ $($field_name),+ ] = ptrs;
                        unsafe {
                            Self {
                                $(
                                    $field_name: $field_name.cast::<$field_ty>().read(),
                                )+
                            }
                        }
                    }

                    // ```
                    // unsafe fn create_ref<'v>(ptrs: [NonNull<u8>; FIELD_COUNT]) -> ThingRef<'v, 'a> {
                    //     let [id, name] = ptrs;
                    //     unsafe {
                    //         ThingRef {
                    //             id: id.cast::<u32>().as_ref(),
                    //             name: name.cast::<&'a str>().as_ref(),
                    //         }
                    //     }
                    // }
                    // ```
                    #[inline]
                    unsafe fn create_ref<'v>(ptrs: [NonNull<u8>; FIELD_COUNT]) -> $($struct_ref_ty)* {
                        let [ $($field_name),+ ] = ptrs;
                        unsafe {
                            [<$struct_name Ref>] {
                                $(
                                    $field_name: $field_name.cast::<$field_ty>().as_ref(),
                                )+
                            }
                        }
                    }

                    // ```
                    // unsafe fn create_mut<'v>(ptrs: [NonNull<u8>; FIELD_COUNT]) -> ThingMut<'v, 'a> {
                    //     let [id, name] = ptrs;
                    //     unsafe {
                    //         ThingMut {
                    //             id: id.cast::<u32>().as_mut(),
                    //             name: name.cast::<&'a str>().as_mut(),
                    //         }
                    //     }
                    // }
                    // ```
                    #[inline]
                    unsafe fn create_mut<'v>(ptrs: [NonNull<u8>; FIELD_COUNT]) -> $($struct_mut_ty)* {
                        let [ $($field_name),+ ] = ptrs;
                        unsafe {
                            [<$struct_name Mut>] {
                                $(
                                    $field_name: $field_name.cast::<$field_ty>().as_mut(),
                                )+
                            }
                        }
                    }

                    // ```
                    // unsafe fn drop_columns(ptrs: [NonNull<u8>; FIELD_COUNT], len: usize) {
                    //     let [id, name] = ptrs;
                    //     unsafe {
                    //         drop_column::<u32>(id, len);
                    //         drop_column::<&'a str>(name, len);
                    //     }
                    // }
                    // ```
                    #[inline]
                    unsafe fn drop_columns(ptrs: [NonNull<u8>; FIELD_COUNT], len: usize) {
                        let [ $($field_name),+ ] = ptrs;
                        unsafe {
                            $(
                                __p::drop_column::<$field_ty>($field_name, len);
                            )+
                        }
                    }
                }

                // Implement `SliceFields` on the struct type (e.g. `Scope`).
                //
                // Unlike the `Fields` impl above (which binds only the struct's lifetimes,
                // `$struct_generics`), this binds the *table's* - `$table_generics`, i.e. the
                // key's lifetimes as well as the struct's - so the key type in the slice views
                // (`ThingSlices`'s `IndexSlice<ThingId<'k>, _>` fields) is nameable here. See
                // the `SliceFields` trait docs for why the views can't live on `Fields`.
                //
                // `unsafe impl<'k, 'a> SliceFields<ThingId<'k>> for Thing<'a> {`
                unsafe impl$($table_generics)* __p::SliceFields<$key_ty> for $struct_ty {
                    // `type Slices<'v> = ThingSlices<'v, 'k, 'a> where Self: 'v;`
                    type Slices<'v> = $($struct_slices_ty)* where Self: 'v;

                    // `type SlicesMut<'v> = ThingSlicesMut<'v, 'k, 'a> where Self: 'v;`
                    type SlicesMut<'v> = $($struct_slices_mut_ty)* where Self: 'v;

                    // ```
                    // unsafe fn create_slices<'v>(ptrs: [NonNull<u8>; FIELD_COUNT], len: usize) -> ThingSlices<'v, 'k, 'a> {
                    //     let [id, name] = ptrs;
                    //     unsafe {
                    //         ThingSlices {
                    //             ids: index_slice_from_raw_parts(id, len),
                    //             names: index_slice_from_raw_parts(name, len),
                    //         }
                    //     }
                    // }
                    // ```
                    #[inline]
                    unsafe fn create_slices<'v>(ptrs: [NonNull<u8>; FIELD_COUNT], len: usize) -> $($struct_slices_ty)* {
                        let [ $($field_name),+ ] = ptrs;
                        unsafe {
                            [<$struct_name Slices>] {
                                $(
                                    $field_plural_name: __p::index_slice_from_raw_parts($field_name, len),
                                )+
                            }
                        }
                    }

                    // ```
                    // unsafe fn create_slices_mut<'v>(ptrs: [NonNull<u8>; FIELD_COUNT], len: usize) -> ThingSlicesMut<'v, 'k, 'a> {
                    //     let [id, name] = ptrs;
                    //     unsafe {
                    //         ThingSlicesMut {
                    //             ids: index_slice_from_raw_parts_mut(id, len),
                    //             names: index_slice_from_raw_parts_mut(name, len),
                    //         }
                    //     }
                    // }
                    // ```
                    #[inline]
                    unsafe fn create_slices_mut<'v>(ptrs: [NonNull<u8>; FIELD_COUNT], len: usize) -> $($struct_slices_mut_ty)* {
                        let [ $($field_name),+ ] = ptrs;
                        unsafe {
                            [<$struct_name SlicesMut>] {
                                $(
                                    $field_plural_name: __p::index_slice_from_raw_parts_mut($field_name, len),
                                )+
                            }
                        }
                    }
                }

                // Implement `CloneFields` on the struct type (e.g. `Scope`).
                // `@generate_clone` expands to nothing when `clone` is empty.
                $crate::multi_vec! {
                    @generate_clone
                    clone = [ $($clone)? ]
                    struct_name = [ $struct_name ]
                    struct_generics = { $($struct_generics)* }
                    fields = [ $(
                        {
                            field_name = [ $field_name ]
                            field_ty = [ $field_ty ]
                        }
                    )+ ]
                }

                // Iterator of refs (e.g. `ScopeTableIter`)

                // `impl<'v, 'a> Iterator for ThingsIter<'v, 'a> {`
                impl$($struct_ref_generics)* Iterator for $($table_iter_ty)* {
                    // `type Item = ThingRef<'v, 'a>`
                    type Item = $($struct_ref_ty)*;

                    #[inline(always)]
                    // `fn next(&mut self) -> Option<ThingRef<'v, 'a>> {`
                    fn next(&mut self) -> Option<$($struct_ref_ty)*> {
                        self.0.next()
                    }

                    #[inline(always)]
                    fn size_hint(&self) -> (usize, Option<usize>) {
                        self.0.size_hint()
                    }
                }

                // `impl<'v, 'a> ExactSizeIterator for ThingsIter<'v, 'a> {}`
                impl$($struct_ref_generics)* ExactSizeIterator for $($table_iter_ty)* {}

                // `impl<'v, 'a> FusedIterator for ThingsIter<'v, 'a> {}`
                impl$($struct_ref_generics)* FusedIterator for $($table_iter_ty)* {}

                // Iterator of mut refs (e.g. `ScopeTableIterMut`)

                // `impl<'v, 'a> Iterator for ThingsIterMut<'v, 'a> {`
                impl$($struct_ref_generics)* Iterator for $($table_iter_mut_ty)* {
                    // `type Item = ThingMut<'v, 'a>`
                    type Item = $($struct_mut_ty)*;

                    #[inline(always)]
                    // `fn next(&mut self) -> Option<ThingMut<'v, 'a>> {`
                    fn next(&mut self) -> Option<$($struct_mut_ty)*> {
                        self.0.next()
                    }

                    #[inline(always)]
                    fn size_hint(&self) -> (usize, Option<usize>) {
                        self.0.size_hint()
                    }
                }

                // `impl<'v, 'a> ExactSizeIterator for ThingsIterMut<'v, 'a> {}`
                impl$($struct_ref_generics)* ExactSizeIterator for $($table_iter_mut_ty)* {}

                // `impl<'v, 'a> FusedIterator for ThingsIterMut<'v, 'a> {}`
                impl$($struct_ref_generics)* FusedIterator for $($table_iter_mut_ty)* {}

                // Iterator of owned items (e.g. `ScopeTableIntoIter`)

                // `impl<'a> Iterator for ThingsIntoIter<'a> {`
                impl$($struct_generics)* Iterator for $($table_into_iter_ty)* {
                    // `type Item = Thing<'a>`
                    type Item = $struct_ty;

                    #[inline(always)]
                    // `fn next(&mut self) -> Option<Thing<'a>> {`
                    fn next(&mut self) -> Option<$struct_ty> {
                        self.0.next()
                    }

                    #[inline(always)]
                    fn size_hint(&self) -> (usize, Option<usize>) {
                        self.0.size_hint()
                    }
                }

                // `impl<'a> ExactSizeIterator for ThingsIntoIter<'a> {}`
                impl$($struct_generics)* ExactSizeIterator for $($table_into_iter_ty)* {}

                // `impl<'a> FusedIterator for ThingsIntoIter<'a> {}`
                impl$($struct_generics)* FusedIterator for $($table_into_iter_ty)* {}
            };
        }
    };

    // Generate the clone machinery - the `CloneFields` impl - for a table declared
    // with `#[derive(Clone)]`.
    //
    // Tables without the derive get no clone machinery at all.
    // `MultiVec` is `Clone` only where its fields struct implements `CloneFields`,
    // so the `#[derive(Clone)]` passed through to the table type resolves exactly
    // when this impl exists.
    //
    // Invoked by `@generate`, unconditionally, from inside the generated `const` block,
    // so the block's imports and `FIELD_COUNT` are in scope.
    //
    // The rule below expands to nothing when the `clone` slot is empty.
    // The choice must be made by rule matching, in a separate rule. `@generate` could only
    // make it with a `$( ... )?` group gated on the `clone` slot, and this impl's per-field
    // code cannot live inside one - all metavariables directly under a repetition must
    // repeat the same number of times, and the field list repeats per field while `$clone`
    // repeats once.
    (
        @generate_clone
        clone = [ Clone ]
        struct_name = [ $struct_name:ident ]
        struct_generics = { $($struct_generics:tt)* }
        fields = [ $(
            {
                field_name = [ $field_name:ident ]
                field_ty = [ $field_ty:ty ]
            }
        )+ ]
    ) => {
        // Implement `CloneFields` on the struct type (e.g. `Scope`)
        //
        // `unsafe impl<'a> CloneFields for Thing<'a> {`
        unsafe impl$($struct_generics)* __p::CloneFields for $struct_name$($struct_generics)* {
            #[inline]
            // ```
            // unsafe fn clone_columns(src_and_dst_ptrs: [SrcAndDstPtrs; FIELD_COUNT], len: usize) {
            //     let [id, name] = src_and_dst_ptrs;
            //     let drop_guards = unsafe {
            //         (
            //             clone_column::<u32>(id, len),
            //             clone_column::<&'a str>(name, len),
            //         )
            //     };
            //     std::mem::forget(drop_guards);
            // }
            // ```
            unsafe fn clone_columns(src_and_dst_ptrs: [__p::SrcAndDstPtrs; FIELD_COUNT], len: usize) {
                let [ $($field_name),+ ] = src_and_dst_ptrs;

                // Each column's `clone_column` call returns a `ColumnDropGuard`.
                // The guards are held in a tuple until all columns are cloned, then forgotten -
                // so if a column's `clone` panics, unwinding drops all values cloned so far,
                // in this column (by `clone_column`'s internal guard) and in earlier columns
                // (by the already-built guards of the partially-evaluated tuple).
                //
                // TODO: It'd be ideal to clone fields in memory order not declaration order,
                // for the same reason that `MultiVec::grow` does - though that'd be a tiny optimization.
                let drop_guards = unsafe {
                    (
                        $(
                            __p::clone_column::<$field_ty>($field_name, len),
                        )+
                    )
                };
                std::mem::forget(drop_guards);
            }
        }
    };

    // No `#[derive(Clone)]` on the `table` - no clone machinery.
    (
        @generate_clone
        clone = []
        struct_name = $_struct_name:tt
        struct_generics = $_struct_generics:tt
        fields = $_fields:tt
    ) => {};

    // Generate the table's `Debug` impl, for a table declared with `#[derive(Debug)]`.
    // (The struct's and view types' `Debug` derives are emitted at their definitions.)
    //
    // Invoked by `@generate`, unconditionally, from inside the generated `const` block,
    // so the block's imports are in scope.
    //
    // The rule below expands to nothing when the `debug` slot is empty.
    // The choice must be made by rule matching, in a separate rule -
    // see the invocation in `@generate` for why a `$( ... )?` group cannot make it.
    (
        @generate_debug
        debug = [ $_debug:ident ]
        table_name = [ $table_name:ident ]
        table_generics = { $($table_generics:tt)* }
    ) => {
        use std::fmt;

        // `impl<'k, 'a> fmt::Debug for Things<'k, 'a> {`
        impl$($table_generics)* fmt::Debug for $table_name$($table_generics)* {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_map().entries(self.iter_enumerated()).finish()
            }
        }
    };

    // No `#[derive(Debug)]` on the `table` - no `Debug` machinery.
    (
        @generate_debug
        debug = []
        table_name = $_table_name:tt
        table_generics = $_table_generics:tt
    ) => {};
}
pub use multi_vec;
