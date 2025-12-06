use std::cmp;

use oxc_ast_macros::ast_meta;
use oxc_estree::{
    CompactFixesJSSerializer, CompactFixesTSSerializer, CompactJSSerializer, CompactTSSerializer,
    Concat2, ESTree, JsonSafeString, PrettyFixesJSSerializer, PrettyFixesTSSerializer,
    PrettyJSSerializer, PrettyTSSerializer, Serializer, StructSerializer,
};
use oxc_span::GetSpan;

use crate::ast::*;

pub mod basic;
pub mod js;
pub mod jsx;
pub mod literal;
pub mod ts;
use basic::{EmptyArray, Null};

/// Main serialization methods for `Program`.
///
/// Note: 8 separate methods for the different serialization options, rather than 1 method
/// with behavior controlled by flags
/// (e.g. `fn to_estree_json(&self, with_ts: bool, pretty: bool, fixes: bool)`)
/// to avoid bloating binary size.
///
/// Most consumers (and Oxc crates) will use only 1 of these methods, so we don't want to needlessly
/// compile all 8 serializers when only 1 is used.
///
/// Initial capacity for serializer's buffer is an estimate based on our benchmark fixtures
/// of ratio of source text size to JSON size.
///
/// | File                       | Compact TS | Compact JS | Pretty TS | Pretty JS |
/// |----------------------------|------------|------------|-----------|-----------|
/// | antd.js                    |         10 |          9 |        76 |        72 |
/// | cal.com.tsx                |         10 |          9 |        40 |        37 |
/// | checker.ts                 |          7 |          6 |        27 |        24 |
/// | pdf.mjs                    |         13 |         12 |        71 |        67 |
/// | RadixUIAdoptionSection.jsx |         10 |          9 |        45 |        44 |
/// |----------------------------|------------|------------|-----------|-----------|
/// | Maximum                    |         13 |         12 |        76 |        72 |
///
/// It's better to over-estimate than under-estimate, as having to grow the buffer is expensive,
/// so have gone on the generous side.
const JSON_CAPACITY_RATIO_COMPACT: usize = 16;
const JSON_CAPACITY_RATIO_PRETTY: usize = 80;

impl Program<'_> {
    /// Serialize AST to ESTree JSON, including TypeScript fields.
    pub fn to_estree_ts_json(&self, ranges: bool, loc: bool) -> String {
        let capacity = self.source_text.len() * JSON_CAPACITY_RATIO_COMPACT;
        let mut serializer = CompactTSSerializer::with_capacity_and_loc(capacity, ranges, loc);
        self.serialize(&mut serializer);
        serializer.into_string()
    }

    /// Serialize AST to ESTree JSON, without TypeScript fields.
    pub fn to_estree_js_json(&self, ranges: bool, loc: bool) -> String {
        let capacity = self.source_text.len() * JSON_CAPACITY_RATIO_COMPACT;
        let mut serializer = CompactJSSerializer::with_capacity_and_loc(capacity, ranges, loc);
        self.serialize(&mut serializer);
        serializer.into_string()
    }

    /// Serialize AST to pretty-printed ESTree JSON, including TypeScript fields.
    pub fn to_pretty_estree_ts_json(&self, ranges: bool, loc: bool) -> String {
        let capacity = self.source_text.len() * JSON_CAPACITY_RATIO_PRETTY;
        let mut serializer = PrettyTSSerializer::with_capacity_and_loc(capacity, ranges, loc);
        self.serialize(&mut serializer);
        serializer.into_string()
    }

    /// Serialize AST to pretty-printed ESTree JSON, without TypeScript fields.
    pub fn to_pretty_estree_js_json(&self, ranges: bool, loc: bool) -> String {
        let capacity = self.source_text.len() * JSON_CAPACITY_RATIO_PRETTY;
        let mut serializer = PrettyJSSerializer::with_capacity_and_loc(capacity, ranges, loc);
        self.serialize(&mut serializer);
        serializer.into_string()
    }

    /// Serialize AST to ESTree JSON, including TypeScript fields, with list of fixes.
    pub fn to_estree_ts_json_with_fixes(&self, ranges: bool, loc: bool) -> String {
        let capacity = self.source_text.len() * JSON_CAPACITY_RATIO_COMPACT;
        let serializer = CompactFixesTSSerializer::with_capacity_and_loc(capacity, ranges, loc);
        serializer.serialize_with_fixes(self)
    }

    /// Serialize AST to ESTree JSON, without TypeScript fields, with list of fixes.
    pub fn to_estree_js_json_with_fixes(&self, ranges: bool, loc: bool) -> String {
        let capacity = self.source_text.len() * JSON_CAPACITY_RATIO_COMPACT;
        let serializer = CompactFixesJSSerializer::with_capacity_and_loc(capacity, ranges, loc);
        serializer.serialize_with_fixes(self)
    }

    /// Serialize AST to pretty-printed ESTree JSON, including TypeScript fields, with list of fixes.
    pub fn to_pretty_estree_ts_json_with_fixes(&self, ranges: bool, loc: bool) -> String {
        let capacity = self.source_text.len() * JSON_CAPACITY_RATIO_PRETTY;
        let serializer = PrettyFixesTSSerializer::with_capacity_and_loc(capacity, ranges, loc);
        serializer.serialize_with_fixes(self)
    }

    /// Serialize AST to pretty-printed ESTree JSON, without TypeScript fields, with list of fixes.
    pub fn to_pretty_estree_js_json_with_fixes(&self, ranges: bool, loc: bool) -> String {
        let capacity = self.source_text.len() * JSON_CAPACITY_RATIO_PRETTY;
        let serializer = PrettyFixesJSSerializer::with_capacity_and_loc(capacity, ranges, loc);
        serializer.serialize_with_fixes(self)
    }
}

/// Serializer for `Program`.
///
/// In TS AST, set start span to start of first directive or statement.
/// This is required because unlike Acorn, TS-ESLint excludes whitespace and comments
/// from the `Program` start span.
/// See <https://github.com/oxc-project/oxc/pull/10134> for more info.
///
/// Special case where first statement is an `ExportNamedDeclaration` or `ExportDefaultDeclaration`
/// exporting a class with decorators, where one of the decorators is before `export`.
/// In these cases, the span of the statement starts after the span of the decorators.
/// e.g. `@dec export class C {}` - `ExportNamedDeclaration` span start is 5, `Decorator` span start is 0.
/// `Program` span start is 0 (not 5).
#[ast_meta]
#[estree(raw_deser = "
    const localAstId = astId;

    const start = IS_TS ? 0 : DESER[u32](POS_OFFSET.span.start),
        end = DESER[u32](POS_OFFSET.span.end);

    const program = parent = {
        type: 'Program',
        body: null,
        sourceType: DESER[ModuleKind](POS_OFFSET.source_type.module_kind),
        hashbang: null,
        /* IF COMMENTS */
        get comments() {
            // Check AST in buffer is still the same AST (buffers are reused)
            if (localAstId !== astId) throw new Error('Comments are only accessible while linting the file');
            // Deserialize the comments.
            // Replace this getter with the comments array, so we don't deserialize twice.
            const comments = DESER[Vec<Comment>](POS_OFFSET.comments);
            Object.defineProperty(this, 'comments', { value: comments });
            return comments;
        },
        /* END_IF */
        /* IF LINTER */
        get tokens() {
            if (tokens === null) initTokens();
            return tokens;
        },
        /* END_IF */
        start,
        end,
        ...(RANGE && { range: [start, end] }),
        ...(PARENT && { parent: null }),
    };

    program.hashbang = DESER[Option<Hashbang>](POS_OFFSET.hashbang);

    const body = program.body = DESER[Vec<Directive>](POS_OFFSET.directives);
    body.push(...DESER[Vec<Statement>](POS_OFFSET.body));

    if (IS_TS) {
        let start;
        if (body.length > 0) {
            const first = body[0];
            start = first.start;
            if (first.type === 'ExportNamedDeclaration' || first.type === 'ExportDefaultDeclaration') {
                const { declaration } = first;
                if (
                    declaration !== null && declaration.type === 'ClassDeclaration'
                    && declaration.decorators.length > 0
                ) {
                    const decoratorStart = declaration.decorators[0].start;
                    if (decoratorStart < start) start = decoratorStart;
                }
            }
        } else {
            start = end;
        }

        if (RANGE) {
            program.start = program.range[0] = start;
        } else {
            program.start = start;
        }
    }

    if (PARENT) parent = null;

    program
")]
pub struct ProgramConverter<'a, 'b>(pub &'b Program<'a>);

impl ESTree for ProgramConverter<'_, '_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let program = self.0;

        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("Program"));
        state.serialize_field("body", &Concat2(&program.directives, &program.body));
        state.serialize_field("sourceType", &program.source_type.module_kind());
        state.serialize_field("hashbang", &program.hashbang);

        let span = if S::INCLUDE_TS_FIELDS {
            Span::new(get_ts_start_span(program), program.span.end)
        } else {
            program.span
        };
        state.serialize_span(span);

        state.end();
    }
}

fn get_ts_start_span(program: &Program<'_>) -> u32 {
    if let Some(first_directive) = program.directives.first() {
        return first_directive.span.start;
    }

    let Some(first_stmt) = program.body.first() else {
        // Program contains no statements or directives. Span start = span end.
        return program.span.end;
    };

    let start = first_stmt.span().start;
    match first_stmt {
        Statement::ExportNamedDeclaration(decl) => {
            if let Some(Declaration::ClassDeclaration(class)) = &decl.declaration
                && let Some(decorator) = class.decorators.first()
            {
                return cmp::min(start, decorator.span.start);
            }
        }
        Statement::ExportDefaultDeclaration(decl) => {
            if let ExportDefaultDeclarationKind::ClassDeclaration(class) = &decl.declaration
                && let Some(decorator) = class.decorators.first()
            {
                return cmp::min(start, decorator.span.start);
            }
        }
        _ => {}
    }
    start
}

/// Serialize `value` field of `Comment`.
///
/// This serializer does not work for JSON serializer, because there's no access to source text
/// in `fn serialize`. But in any case, comments often contain characters which need escaping in JSON,
/// which is slow, so it's probably faster to transfer comments as NAPI types (which we do).
///
/// This meta type is only present for raw transfer, which can transfer faster.
#[ast_meta]
#[estree(
    ts_type = "string",
    raw_deser = "SOURCE_TEXT.slice(THIS.start + 2, THIS.end - (THIS.type === 'Line' ? 0 : 2))"
)]
pub struct CommentValue<'b>(#[expect(dead_code)] pub &'b Comment);

impl ESTree for CommentValue<'_> {
    #[expect(clippy::unimplemented)]
    fn serialize<S: Serializer>(&self, _serializer: S) {
        unimplemented!();
    }
}
