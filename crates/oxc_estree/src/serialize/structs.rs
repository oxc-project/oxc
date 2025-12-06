use oxc_data_structures::code_buffer::CodeBuffer;

use super::{
    Config, ESTree, ESTreeSequenceSerializer, ESTreeSerializer, Formatter, Serializer,
    SerializerPrivate, TracePathPart,
};

/// Trait for struct serializers.
pub trait StructSerializer {
    type Config: Config;
    type Formatter: Formatter;

    /// Serialize struct field.
    ///
    /// `key` must not contain any characters which require escaping in JSON.
    fn serialize_field<T: ESTree + ?Sized>(&mut self, key: &'static str, value: &T);

    /// Serialize struct field which is JS syntax only (not in TS AST).
    ///
    /// This method behaves differently, depending on the serializer's `Config`:
    /// * `INCLUDE_TS_FIELDS == false`: Behaves same as `serialize_field`
    ///   i.e. the field is included in JSON.
    /// * `INCLUDE_TS_FIELDS == true`: Do nothing.
    ///   i.e. the field is skipped.
    ///
    /// `key` must not contain any characters which require escaping in JSON.
    fn serialize_js_field<T: ESTree + ?Sized>(&mut self, key: &'static str, value: &T);

    /// Serialize struct field which is TypeScript syntax.
    ///
    /// This method behaves differently, depending on the serializer's `Config`:
    /// * `INCLUDE_TS_FIELDS == true`: Behaves same as `serialize_field`
    ///   i.e. the field is included in JSON.
    /// * `INCLUDE_TS_FIELDS == false`: Do nothing.
    ///   i.e. the field is skipped.
    ///
    /// `key` must not contain any characters which require escaping in JSON.
    fn serialize_ts_field<T: ESTree + ?Sized>(&mut self, key: &'static str, value: &T);

    /// Serialize `Span`.
    ///
    /// * If `serializer.ranges() == true`, outputs `start`, `end`, and `range` fields.
    /// * If `serializer.loc() == true`, outputs `loc` field with line/column information.
    /// * Otherwise, outputs only `start` and `end`.
    fn serialize_span<S: ESTreeSpan>(&mut self, span: S);

    /// Serialize `Span` with line/column information.
    ///
    /// This is used when loc information is available from a translation table.
    fn serialize_span_with_loc<S: ESTreeSpan>(
        &mut self,
        span: S,
        start_loc: (u32, u32),
        end_loc: (u32, u32),
    );

    /// Finish serializing struct.
    fn end(self);

    /// Get whether output should contain `range` fields.
    fn ranges(&self) -> bool;

    /// Get whether output should contain `loc` fields.
    fn loc(&self) -> bool;
}

/// Serializer for structs.
///
/// This is returned by `ESTreeSerializer::serialize_struct`.
pub struct ESTreeStructSerializer<'s, C: Config, F: Formatter> {
    /// Serializer
    serializer: &'s mut ESTreeSerializer<C, F>,
    /// State of struct.
    /// Starts as `StructState::Empty`, transitions to `StructState::HasFields` on first field.
    state: StructState,
}

impl<'s, C: Config, F: Formatter> ESTreeStructSerializer<'s, C, F> {
    /// Create new [`ESTreeStructSerializer`].
    pub(super) fn new(mut serializer: &'s mut ESTreeSerializer<C, F>) -> Self {
        // Push item to `trace_path`. It will be replaced with a `TracePathPart::Key`
        // when serializing each field in the struct, and popped off again in `end` method.
        if C::FIXES {
            serializer.trace_path.push(TracePathPart::DUMMY);
        }

        serializer.buffer_mut().print_ascii_byte(b'{');

        Self { serializer, state: StructState::Empty }
    }
}

impl<C: Config, F: Formatter> StructSerializer for ESTreeStructSerializer<'_, C, F> {
    type Config = C;
    type Formatter = F;

    /// Serialize struct field.
    ///
    /// `key` must not contain any characters which require escaping in JSON.
    fn serialize_field<T: ESTree + ?Sized>(&mut self, key: &'static str, value: &T) {
        // Update last item in trace path to current key
        if C::FIXES {
            *self.serializer.trace_path.last_mut() = TracePathPart::Key(key);
        }

        let (buffer, formatter) = self.serializer.buffer_and_formatter_mut();
        if self.state == StructState::Empty {
            self.state = StructState::HasFields;
            formatter.before_first_element(buffer);
        } else {
            buffer.print_ascii_byte(b',');
            formatter.before_later_element(buffer);
        }

        buffer.print_ascii_byte(b'"');
        buffer.print_str(key);
        buffer.print_str("\":");
        formatter.before_field_value(buffer);
        value.serialize(&mut *self.serializer);
    }

    /// Serialize struct field which is JS syntax only (not in TS AST).
    ///
    /// This method behaves differently, depending on the serializer's `Config`:
    /// * `INCLUDE_TS_FIELDS == false`: Behaves same as `serialize_field`
    ///   i.e. the field is included in JSON.
    /// * `INCLUDE_TS_FIELDS == true`: Do nothing.
    ///   i.e. the field is skipped.
    ///
    /// `key` must not contain any characters which require escaping in JSON.
    #[inline(always)]
    fn serialize_js_field<T: ESTree + ?Sized>(&mut self, key: &'static str, value: &T) {
        if !C::INCLUDE_TS_FIELDS {
            self.serialize_field(key, value);
        }
    }

    /// Serialize struct field which is TypeScript syntax.
    ///
    /// This method behaves differently, depending on the serializer's `Config`:
    /// * `INCLUDE_TS_FIELDS == true`: Behaves same as `serialize_field`
    ///   i.e. the field is included in JSON.
    /// * `INCLUDE_TS_FIELDS == false`: Do nothing.
    ///   i.e. the field is skipped.
    ///
    /// `key` must not contain any characters which require escaping in JSON.
    #[inline(always)]
    fn serialize_ts_field<T: ESTree + ?Sized>(&mut self, key: &'static str, value: &T) {
        if C::INCLUDE_TS_FIELDS {
            self.serialize_field(key, value);
        }
    }

    /// Serialize `Span`.
    ///
    /// * If `serializer.ranges() == true`, outputs `start`, `end`, and `range` fields.
    /// * If `serializer.loc() == true`, outputs `loc` field with line/column information.
    /// * Otherwise, outputs only `start` and `end`.
    ///
    /// * Now automatically uses translation table if available via LocProvider
    fn serialize_span<S: ESTreeSpan>(&mut self, span: S) {
        let range = span.range();
        self.serialize_field("start", &range[0]);
        self.serialize_field("end", &range[1]);
        if self.serializer.ranges() {
            self.serialize_field("range", &range);
        }
        if self.serializer.loc() {
            // Try to get real location info from provider
            let loc_provider = self.serializer.config.loc_provider();
            if let (Some(start_loc), Some(end_loc)) = (
                loc_provider.offset_to_line_column(range[0]),
                loc_provider.offset_to_line_column(range[1]),
            ) {
                // Use real location information! 🎉
                self.serialize_field("loc", &SourceLocation { start: start_loc, end: end_loc });
            } else {
                // Fallback to placeholder (backward compatibility)
                self.serialize_field("loc", &SourceLocation { start: (0, 0), end: (0, 0) });
            }
        }
    }

    /// Serialize `Span` with line/column information.
    ///
    /// This is used when loc information is available from a translation table.
    fn serialize_span_with_loc<S: ESTreeSpan>(
        &mut self,
        span: S,
        start_loc: (u32, u32),
        end_loc: (u32, u32),
    ) {
        let range = span.range();
        self.serialize_field("start", &range[0]);
        self.serialize_field("end", &range[1]);
        if self.serializer.ranges() {
            self.serialize_field("range", &range);
        }
        if self.serializer.loc() {
            self.serialize_field("loc", &SourceLocation { start: start_loc, end: end_loc });
        }
    }

    /// Finish serializing struct.
    fn end(self) {
        let mut serializer = self.serializer;

        // Pop entry for this struct from `trace_path`
        if C::FIXES {
            // SAFETY: `trace_path` is pushed to in `new`, which is only way to create an `ESTreeStructSerializer`.
            // This method consumes the `ESTreeStructSerializer`, so this method can't be called more
            // times than `new`. So there must be an item to pop.
            unsafe { serializer.trace_path.pop_unchecked() };
        }

        let (buffer, formatter) = serializer.buffer_and_formatter_mut();
        if self.state == StructState::HasFields {
            formatter.after_last_element(buffer);
        }
        buffer.print_ascii_byte(b'}');
    }

    /// Get whether output should contain `range` fields.
    #[inline(always)]
    fn ranges(&self) -> bool {
        self.serializer.ranges()
    }

    /// Get whether output should contain `loc` fields.
    #[inline(always)]
    fn loc(&self) -> bool {
        self.serializer.loc()
    }
}

/// State of [`StructSerializer`].
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum StructState {
    Empty,
    HasFields,
}

/// Flattening struct serializer.
///
/// The only method it's legal to call on a `FlatStructSerializer` is `serialize_struct`.
///
/// # Example
/// ```
/// struct SomeNode {
///     // We want to flatten `Span` as if its fields were on `SomeNode`
///     span: Span,
///     whatever: u32,
/// }
///
/// struct Span {
///     start: u32,
///     end: u32,
/// }
///
/// impl ESTree for SomeNode {
///     fn serialize<S: Serializer>(&self, serializer: S) {
///         let mut state = serializer.serialize_struct();
///         self.inner.serialize(FlatStructSerializer(&mut state));
///         state.serialize_field("whatever", &self.whatever);
///         state.end();
///     }
/// }
///
/// impl ESTree for Span {
///     fn serialize<S: Serializer>(&self, serializer: S) {
///         let mut state = serializer.serialize_struct();
///         state.serialize_field("start", &self.start);
///         state.serialize_field("end", &self.end);
///         state.end();
///     }
/// }
/// ```
pub struct FlatStructSerializer<'p, P: StructSerializer>(pub &'p mut P);

impl<'p, P: StructSerializer> Serializer for FlatStructSerializer<'p, P> {
    /// `true` if output should contain TS fields
    const INCLUDE_TS_FIELDS: bool = P::Config::INCLUDE_TS_FIELDS;

    type StructSerializer = Self;
    type SequenceSerializer = ESTreeSequenceSerializer<'p, P::Config, P::Formatter>;

    /// Serialize struct.
    fn serialize_struct(self) -> Self {
        self
    }

    fn serialize_sequence(self) -> ESTreeSequenceSerializer<'p, P::Config, P::Formatter> {
        const {
            panic!("Cannot flatten a sequence into a struct");
        }
    }

    fn record_fix_path(&mut self) {
        const {
            panic!("Cannot call `record_fix_path` on a `FlatStructSerializer`");
        }
    }

    /// Get whether output should contain `range` fields.
    #[inline(always)]
    fn ranges(&self) -> bool {
        self.0.ranges()
    }

    /// Get whether output should contain `loc` fields.
    #[inline(always)]
    fn loc(&self) -> bool {
        self.0.loc()
    }
}

impl<P: StructSerializer> SerializerPrivate for FlatStructSerializer<'_, P> {
    type Formatter = P::Formatter;

    fn buffer_mut(&mut self) -> &mut CodeBuffer {
        const {
            panic!("Cannot flatten anything but a struct into another struct");
        }
    }

    fn buffer_and_formatter_mut(&mut self) -> (&mut CodeBuffer, &mut P::Formatter) {
        const {
            panic!("Cannot flatten anything but a struct into another struct");
        }
    }
}

impl<P: StructSerializer> StructSerializer for FlatStructSerializer<'_, P> {
    type Config = P::Config;
    type Formatter = P::Formatter;

    /// Serialize struct field.
    ///
    /// `key` must not contain any characters which require escaping in JSON.
    #[inline(always)]
    fn serialize_field<T: ESTree + ?Sized>(&mut self, key: &'static str, value: &T) {
        // Delegate to parent `StructSerializer`
        self.0.serialize_field(key, value);
    }

    /// Serialize struct field which is JS syntax only (not in TS AST).
    ///
    /// This method behaves differently, depending on the serializer's `Config`:
    /// * `INCLUDE_TS_FIELDS == false`: Behaves same as `serialize_field`
    ///   i.e. the field is included in JSON.
    /// * `INCLUDE_TS_FIELDS == true`: Do nothing.
    ///   i.e. the field is skipped.
    ///
    /// `key` must not contain any characters which require escaping in JSON.
    #[inline(always)]
    fn serialize_js_field<T: ESTree + ?Sized>(&mut self, key: &'static str, value: &T) {
        // Delegate to parent `StructSerializer`
        self.0.serialize_js_field(key, value);
    }

    /// Serialize struct field which is TypeScript syntax.
    ///
    /// This method behaves differently, depending on the serializer's `Config`:
    /// * `INCLUDE_TS_FIELDS == true`: Behaves same as `serialize_field`
    ///   i.e. the field is included in JSON.
    /// * `INCLUDE_TS_FIELDS == false`: Do nothing.
    ///   i.e. the field is skipped.
    ///
    /// `key` must not contain any characters which require escaping in JSON.
    #[inline(always)]
    fn serialize_ts_field<T: ESTree + ?Sized>(&mut self, key: &'static str, value: &T) {
        // Delegate to parent `StructSerializer`
        self.0.serialize_ts_field(key, value);
    }

    /// Serialize `Span`.
    ///
    /// * If `serializer.ranges() == true`, outputs `start`, `end`, and `range` fields.
    /// * If `serializer.loc() == true`, outputs `loc` field with line/column information.
    /// * Otherwise, outputs only `start` and `end`.
    fn serialize_span<S: ESTreeSpan>(&mut self, span: S) {
        self.0.serialize_span(span);
    }

    /// Serialize `Span` with line/column information.
    ///
    /// This is used when loc information is available from a translation table.
    fn serialize_span_with_loc<S: ESTreeSpan>(
        &mut self,
        span: S,
        start_loc: (u32, u32),
        end_loc: (u32, u32),
    ) {
        self.0.serialize_span_with_loc(span, start_loc, end_loc);
    }

    /// Finish serializing struct.
    fn end(self) {
        // No-op - there may be more fields to be added to the struct in the parent
    }

    /// Get whether output should contain `range` fields.
    #[inline(always)]
    fn ranges(&self) -> bool {
        self.0.ranges()
    }

    /// Get whether output should contain `loc` fields.
    #[inline(always)]
    fn loc(&self) -> bool {
        self.0.loc()
    }
}

/// Trait for `Span` to implement.
///
/// This is a workaround to avoid circular dependency. `oxc_span` crate depends on `oxc_estree` crate,
/// so we can't import `Span` directly from `oxc_span` here.
pub trait ESTreeSpan: Copy {
    fn range(self) -> [u32; 2];
}

/// Trait for providing location information (line/column) from offsets.
/// This allows serializers to optionally access translation tables without circular dependencies.
pub trait LocProvider {
    /// Convert UTF-8 offset to (line, column) where both are 0-based.
    /// Returns None if no translation is available.
    fn offset_to_line_column(&self, utf8_offset: u32) -> Option<(u32, u32)>;
}

/// Dummy implementation for when no translation is needed
pub struct NoLocProvider;

impl LocProvider for NoLocProvider {
    fn offset_to_line_column(&self, _utf8_offset: u32) -> Option<(u32, u32)> {
        None
    }
}

/// Source location information for ESTree loc field.
pub struct SourceLocation {
    pub start: (u32, u32), // (line, column) - 0-based
    pub end: (u32, u32),   // (line, column) - 0-based
}

impl ESTree for SourceLocation {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("start", &Position { line: self.start.0 + 1, column: self.start.1 });
        state.serialize_field("end", &Position { line: self.end.0 + 1, column: self.end.1 });
        state.end();
    }
}

/// Position information for ESTree loc.start and loc.end fields.
pub struct Position {
    pub line: u32,   // 1-based
    pub column: u32, // 0-based
}

impl ESTree for Position {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("line", &self.line);
        state.serialize_field("column", &self.column);
        state.end();
    }
}

#[cfg(test)]
mod tests {
    use super::super::{
        CompactJSSerializer, CompactTSSerializer, FlatStructSerializer, PrettyJSSerializer,
        PrettyTSSerializer, Serializer,
    };
    use super::*;

    #[test]
    fn serialize_struct() {
        struct Foo<'a> {
            n: f64,
            u: u32,
            bar: Bar<'a>,
            empty: Null,
            hello: Option<&'a str>,
            maybe_bar: Option<Bar<'a>>,
            maybe_not_bar: Option<Bar<'a>>,
        }

        struct Bar<'a> {
            yes: &'a str,
            no: &'a str,
        }

        struct Null;

        impl ESTree for Foo<'_> {
            fn serialize<S: Serializer>(&self, serializer: S) {
                let mut state = serializer.serialize_struct();
                state.serialize_field("n", &self.n);
                state.serialize_field("u", &self.u);
                state.serialize_field("bar", &self.bar);
                state.serialize_field("empty", &self.empty);
                state.serialize_field("hello", &self.hello);
                state.serialize_field("maybe_bar", &self.maybe_bar);
                state.serialize_field("maybe_not_bar", &self.maybe_not_bar);
                state.end();
            }
        }

        impl ESTree for Bar<'_> {
            fn serialize<S: Serializer>(&self, serializer: S) {
                let mut state = serializer.serialize_struct();
                state.serialize_field("yes", &self.yes);
                state.serialize_field("no", &self.no);
                state.end();
            }
        }

        impl ESTree for Null {
            fn serialize<S: Serializer>(&self, serializer: S) {
                let state = serializer.serialize_struct();
                state.end();
            }
        }

        let foo = Foo {
            n: 123.0,
            u: 12345,
            bar: Bar { yes: "yup", no: "nope" },
            empty: Null,
            hello: Some("hi!"),
            maybe_bar: Some(Bar { yes: "hell yeah!", no: "not a chance in a million, mate" }),
            maybe_not_bar: None,
        };

        let mut serializer = CompactTSSerializer::default();
        foo.serialize(&mut serializer);
        let s = serializer.into_string();
        assert_eq!(
            &s,
            r#"{"n":123,"u":12345,"bar":{"yes":"yup","no":"nope"},"empty":{},"hello":"hi!","maybe_bar":{"yes":"hell yeah!","no":"not a chance in a million, mate"},"maybe_not_bar":null}"#
        );

        let mut serializer = PrettyTSSerializer::default();
        foo.serialize(&mut serializer);
        let s = serializer.into_string();
        assert_eq!(
            &s,
            r#"{
  "n": 123,
  "u": 12345,
  "bar": {
    "yes": "yup",
    "no": "nope"
  },
  "empty": {},
  "hello": "hi!",
  "maybe_bar": {
    "yes": "hell yeah!",
    "no": "not a chance in a million, mate"
  },
  "maybe_not_bar": null
}"#
        );
    }

    #[test]
    fn serialize_flattened_struct() {
        struct Outer {
            outer1: &'static str,
            inner: Inner,
            outer2: &'static str,
        }

        struct Inner {
            inner1: &'static str,
            innermost: Innermost,
            inner2: &'static str,
        }

        struct Innermost {
            innermost1: &'static str,
            innermost2: &'static str,
        }

        impl ESTree for Outer {
            fn serialize<S: Serializer>(&self, serializer: S) {
                let mut state = serializer.serialize_struct();
                state.serialize_field("outer1", &self.outer1);
                self.inner.serialize(FlatStructSerializer(&mut state));
                state.serialize_field("outer2", &self.outer2);
                state.end();
            }
        }

        impl ESTree for Inner {
            fn serialize<S: Serializer>(&self, serializer: S) {
                let mut state = serializer.serialize_struct();
                state.serialize_field("inner1", &self.inner1);
                self.innermost.serialize(FlatStructSerializer(&mut state));
                state.serialize_field("inner2", &self.inner2);
                state.end();
            }
        }

        impl ESTree for Innermost {
            fn serialize<S: Serializer>(&self, serializer: S) {
                let mut state = serializer.serialize_struct();
                state.serialize_field("innermost1", &self.innermost1);
                state.serialize_field("innermost2", &self.innermost2);
                state.end();
            }
        }

        let outer = Outer {
            outer1: "out1",
            inner: Inner {
                inner1: "in1",
                innermost: Innermost { innermost1: "inin1", innermost2: "inin2" },
                inner2: "in2",
            },
            outer2: "out2",
        };

        let mut serializer = CompactTSSerializer::default();
        outer.serialize(&mut serializer);
        let s = serializer.into_string();
        assert_eq!(
            &s,
            r#"{"outer1":"out1","inner1":"in1","innermost1":"inin1","innermost2":"inin2","inner2":"in2","outer2":"out2"}"#
        );

        let mut serializer = PrettyTSSerializer::default();
        outer.serialize(&mut serializer);
        let s = serializer.into_string();
        assert_eq!(
            &s,
            r#"{
  "outer1": "out1",
  "inner1": "in1",
  "innermost1": "inin1",
  "innermost2": "inin2",
  "inner2": "in2",
  "outer2": "out2"
}"#
        );
    }

    #[test]
    fn serialize_struct_with_or_without_ts() {
        struct Foo {
            js: u32,
            ts: u32,
            js_only: u32,
            more_js: u32,
        }

        impl ESTree for Foo {
            fn serialize<S: Serializer>(&self, serializer: S) {
                let mut state = serializer.serialize_struct();
                state.serialize_field("js", &self.js);
                state.serialize_ts_field("ts", &self.ts);
                state.serialize_js_field("jsOnly", &self.js_only);
                state.serialize_field("moreJs", &self.more_js);
                state.end();
            }
        }

        let foo = Foo { js: 1, ts: 2, js_only: 3, more_js: 4 };

        let mut serializer = CompactTSSerializer::default();
        foo.serialize(&mut serializer);
        let s = serializer.into_string();
        assert_eq!(&s, r#"{"js":1,"ts":2,"moreJs":4}"#);

        let mut serializer = PrettyTSSerializer::default();
        foo.serialize(&mut serializer);
        let s = serializer.into_string();
        assert_eq!(
            &s,
            r#"{
  "js": 1,
  "ts": 2,
  "moreJs": 4
}"#
        );

        let mut serializer = CompactJSSerializer::default();
        foo.serialize(&mut serializer);
        let s = serializer.into_string();
        assert_eq!(&s, r#"{"js":1,"jsOnly":3,"moreJs":4}"#);

        let mut serializer = PrettyJSSerializer::default();
        foo.serialize(&mut serializer);
        let s = serializer.into_string();
        assert_eq!(
            &s,
            r#"{
  "js": 1,
  "jsOnly": 3,
  "moreJs": 4
}"#
        );
    }
}
