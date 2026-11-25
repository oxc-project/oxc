//! Angular: the JS of a template (`{{ }}`, `[a]`, `[(a)]`, `(a)`, `*a`, `{{ }}` in another attribute's value, block parameters, `@let`),
//! formatted by the session's dispatcher as TS fragments, split as the parser reads it ([`AngularExpression`] and the rest):
//! each piece of code is dispatched, Angular's own syntax around them (pipes, comments, `;`, microsyntax, block parameters) prints here.
//! A pipe below the root, which TS reads otherwise, is dispatched as a name standing for it ([`Code::dispatch`]):
//! a partial support, JS lays the code out for a name there (apps/oxfmt/DIVERGENCES.md#js-in-angular-nested-pipe-as-name).
//! A piece the dispatcher does not format leaves what holds it as written: the value, a block's parameters, a `{{ }}`, a `@let`'s value up to its `;`;
//! as one naming a JS reserved word does (apps/oxfmt/DIVERGENCES.md#js-in-angular-reserved-word-as-written), which TS reads otherwise,
//! or holding a pipe the parser leaves in the code (apps/oxfmt/DIVERGENCES.md#js-in-angular-nested-pipe-as-written).

use oxc_allocator::ArenaVec;
use oxc_formatter_core::{
    Format, FormatElement, Placeholder, PlaceholderPiece,
    builders::{
        FormatWith, align, expand_parent, group, indent, line_suffix, soft_block_indent,
        soft_line_break, soft_line_break_or_space, soft_space_or_block_indent, space, text,
    },
    dispatch_fragment_ir, format_args, map_text_in_ir, write,
};
use oxc_htmlish_parser::{
    Span,
    ast::{
        AngularAction, AngularCode, AngularExpression, Attribute, Block, BlockParameterKind,
        LetDeclaration, NestedPipe, NestedPipeParent, TemplateBinding, ValueInterpolation,
    },
};

use super::{
    HtmlishFormatter,
    attribute::{quoted_piece, unescaped},
    embed, format_ir, format_once, format_with,
};
use crate::context::HtmlishFormatContext;

/// Parent→child context for a JS fragment of an Angular template, dispatched as TS: what it is, and what this module prints after it.
///
/// The host translates it into JS's `JsRoot` and `JsEmbeddedIn`.
#[derive(Clone, Copy, Debug)]
pub struct JsInAngularPiece {
    pub part: JsInAngularTemplate,
    /// This module puts it in parens of its own, a pipe after them (`(a ? b : c) | p`, prettier#5389): JS's `ExpressionRoot::closes_paren`.
    pub closes_paren: bool,
}

/// What a JS fragment of an Angular template is ([`JsInAngularPiece`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JsInAngularTemplate {
    /// A piece of code in `{{ }}` (all of it, but its pipes and comment): an expression the interpolation lays out.
    /// `before_brace`: an object's `}` follows it, the last piece of a nested pipe that is an object's last value ([`NestedPipe::last_value`]).
    Interpolation { before_brace: bool },
    /// A binding's value (`[a]`, `[(a)]`, `bind-a`, a lone expression of `*a`) without a pipe or a comment:
    /// an expression in a `"`-quoted attribute value, whose strings print in `'`;
    /// an object, an array, a string or a template literal hugs the quotes.
    /// `tailwind_classes`: its strings are Tailwind classes (`TailwindAttributes::binds`).
    /// `alias`: the lone expression's alias, ` as x` after the code in its layout (`{⏎  a: 1,⏎} as x`, as Prettier prints it).
    Binding { tailwind_classes: bool, alias: Option<Span> },
    /// A `{{ }}` in another attribute's value (`title="Hi {{ name }}"`), or a piece of its code: an expression it lays out.
    /// `before_brace`: as [`Interpolation`](Self::Interpolation)'s.
    ValueInterpolation { before_brace: bool },
    /// A piece of code in a block's parameters or `@let`'s value, which this module lays out:
    /// no attribute's value, its strings in the configured quote.
    BlockPiece,
    /// A piece of code in an attribute's value, which this module lays out:
    /// of a binding with a pipe or a comment (its code, or a pipe's argument), a statement of `(a)`, an expression of `*a` among others.
    /// `tailwind_classes`: as [`Binding`](Self::Binding)'s.
    AttributePiece { tailwind_classes: bool },
}

impl JsInAngularTemplate {
    /// In a `"`-quoted attribute value: its quote entities read back and `"` escaped.
    pub const fn in_html_attribute(self) -> bool {
        !matches!(self, Self::Interpolation { .. } | Self::BlockPiece)
    }

    /// The length of the alias text the code ends in, as `dispatch_piece` joins it (JS's `ExpressionRoot::host_suffix`).
    pub const fn suffix_len(self) -> usize {
        match self {
            Self::Binding { alias: Some(alias), .. } => {
                ALIAS.len() + (alias.end - alias.start) as usize
            }
            _ => 0,
        }
    }

    /// Of a piece an object's `}` follows (`before_brace`) or not, where `}}` would end its `{{ }}`.
    const fn before_brace(self, before_brace: bool) -> Self {
        match self {
            Self::Interpolation { .. } => Self::Interpolation { before_brace },
            Self::ValueInterpolation { .. } => Self::ValueInterpolation { before_brace },
            part => part,
        }
    }
}

/// `;` and a line, between statements or a block's parameters.
pub(super) const SEPARATOR: FormatWith<fn(&mut HtmlishFormatter<'_, '_>)> =
    format_with(|f| write!(f, [";", soft_line_break_or_space()]));

/// What precedes an alias after the code ([`JsInAngularTemplate::Binding`]).
const ALIAS: &str = " as ";

/// A piece of code formatted as `part`, its alias after it ([`JsInAngularTemplate::suffix_len`] measures it);
/// `closes_paren`: [`JsInAngularPiece::closes_paren`]. `None` when the dispatcher leaves it as written.
fn dispatch_piece<'a>(
    part: JsInAngularTemplate,
    closes_paren: bool,
    piece: &'a str,
    f: &HtmlishFormatter<'_, 'a>,
) -> Option<Ir<'a>> {
    let piece = match part {
        JsInAngularTemplate::Binding { alias: Some(alias), .. } => {
            let alias = alias.slice(f.context().tree().source());
            let with_alias = f.allocator().alloc_concat_strs_array([piece, ALIAS, alias]);
            debug_assert_eq!(with_alias.len() - piece.len(), part.suffix_len());
            with_alias
        }
        _ => piece,
    };
    JsInAngularPiece { part, closes_paren }.dispatch(piece, f)
}

/// The only ways this module dispatches a piece: the dispatcher reads this context, never a bare [`JsInAngularTemplate`],
/// which it would take for no Angular piece (a program, with a `;`).
impl JsInAngularPiece {
    /// `piece` formatted as TS, an attribute value's quote entities read back.
    fn dispatch<'a>(self, piece: &'a str, f: &HtmlishFormatter<'_, 'a>) -> Option<Ir<'a>> {
        if self.part.in_html_attribute() {
            embed::dispatch_attribute_value("ts", &self, unescaped(piece, f), f)
        } else {
            dispatch_fragment_ir(f, "ts", piece, Some(&self))
        }
    }

    /// Writes an attribute's whole `value` formatted as TS, quoted; `false` to print it as written.
    fn write_attribute_value<'a>(self, value: &'a str, f: &mut HtmlishFormatter<'_, 'a>) -> bool {
        embed::write_attribute_value("ts", &self, value, f)
    }
}

type Ir<'a> = ArenaVec<'a, FormatElement<'a>>;

/// `{{ }}`'s expression formatted; `None` to print it as written.
pub fn format_interpolation<'a>(
    expression: Option<&AngularExpression<'_>>,
    f: &HtmlishFormatter<'_, 'a>,
) -> Option<impl Format<'a, HtmlishFormatContext<'a>> + use<'a>> {
    let part = JsInAngularTemplate::Interpolation { before_brace: false };
    Some(Parts::dispatch(expression?, part, f)?.format())
}

/// Writes a binding's (unescaped) `value` formatted, quoted; `false` to print it as written.
/// `tailwind_classes`: its strings are Tailwind classes.
pub fn write_binding<'a>(
    expression: Option<&AngularExpression<'_>>,
    value: &'a str,
    tailwind_classes: bool,
    f: &mut HtmlishFormatter<'_, 'a>,
) -> bool {
    let Some(expression) = expression else { return false };
    let part = JsInAngularTemplate::Binding { tailwind_classes, alias: None };
    // The whole value, its quote entities read back by the caller, which an object or an array hugs
    if expression.pipes.is_empty()
        && expression.comment.is_none()
        && expression.code.nested.is_empty()
    {
        return JsInAngularPiece { part, closes_paren: false }.write_attribute_value(value, f);
    }
    let Some(parts) = Parts::dispatch(expression, part, f) else { return false };
    // Pipes (in parens all of the code too: `(a | p)`) or a comment lay out around the code
    if parts.pipes.is_empty() && parts.comment.is_none() {
        embed::write_quoted(&parts.format(), f);
    } else {
        embed::write_quoted(&soft_block_indent(&parts.format()), f);
    }
    true
}

/// Writes an action's (`(a)`'s) statements formatted, quoted, `; ` between them (none after the last, which Angular reads alike),
/// each on a line of its own when they break; `false` to print it as written.
/// An object or an array does not hug the quotes (as Prettier prints it).
pub fn write_action(action: Option<&AngularAction<'_>>, f: &mut HtmlishFormatter<'_, '_>) -> bool {
    let Some(action) = action else { return false };
    let source = f.context().tree().source();
    let Some(statements) = action
        .statements
        .iter()
        .map(|statement| {
            let part = JsInAngularTemplate::AttributePiece { tailwind_classes: false };
            dispatch_piece(part, false, statement.span.slice(source), f)
        })
        .collect::<Option<Vec<_>>>()
    else {
        return false;
    };
    let comment = action.comment.map(|comment| comment.slice(source));
    let statements = format_once(move |f| {
        let chain = format_once(move |f| {
            f.join_with(SEPARATOR).entries(statements.into_iter().map(format_ir));
        });
        write!(f, group(&chain));
        write_comment(comment, f);
    });
    embed::write_quoted(&soft_block_indent(&statements), f);
    true
}

/// Writes another attribute's value (`title="Hi {{ name }}"`) quoted: its text as written, each `{{ }}` formatted,
/// on lines of its own when it breaks (as Prettier prints it); `false` to print it as written, where none formats.
pub fn write_value_interpolations<'a>(
    attribute: &Attribute<'_>,
    interpolations: &[ValueInterpolation<'_>],
    f: &mut HtmlishFormatter<'_, 'a>,
) -> bool {
    let source = f.context().tree().source();
    let Some(value) = attribute.value else { return false };
    // Each `{{ }}` formatted; one that is not prints as written, as the text around it
    let parts: Vec<_> = interpolations
        .iter()
        .filter_map(|interpolation| {
            let expression = interpolation.angular.as_deref()?;
            Some((
                interpolation,
                Parts::dispatch(
                    expression,
                    JsInAngularTemplate::ValueInterpolation { before_brace: false },
                    f,
                )?,
            ))
        })
        .collect();
    if parts.is_empty() {
        return false;
    }
    // The text around each `{{ }}`, as written, for `"` quotes
    let text_between = |start: u32, end: u32, f: &HtmlishFormatter<'_, 'a>| {
        quoted_piece(unescaped(&source[start as usize..end as usize], f), f)
    };
    let mut start = value.span.start;
    write!(f, "\"");
    for (interpolation, parts) in parts {
        let before = text_between(start, interpolation.expression.start - 2, f);
        let parts = parts.format();
        write!(
            f,
            [text(before), group(&format_args!("{{", soft_space_or_block_indent(&parts), "}}"))]
        );
        start = interpolation.expression.end + 2;
    }
    let after = text_between(start, value.span.end, f);
    write!(f, [text(after), "\""]);
    true
}

/// Writes a structural directive's value (`*ngFor`) formatted, quoted: its bindings as Prettier prints them,
/// `;` between them (a space before `of`), a `:` after a key but `of`, `then`, `else` and `track`,
/// each key as written (apps/oxfmt/DIVERGENCES.md#js-in-angular-microsyntax-keys-as-written);
/// the directive's own expression alone as a binding's value (`value`, its quote entities read back);
/// `false` to print it as written.
pub fn write_template_bindings<'a>(
    bindings: Option<&'a [TemplateBinding<'a>]>,
    value: &'a str,
    f: &mut HtmlishFormatter<'_, 'a>,
) -> bool {
    let source = f.context().tree().source();
    let bindings = match bindings {
        // `*ngFor`'s own expression, which is none
        Some(
            [
                TemplateBinding::Expression { key: None, expression: None, alias: None, .. },
                rest @ ..,
            ],
        ) => rest,
        Some(bindings) => bindings,
        None => return false,
    };
    match bindings {
        [] => return false,
        [TemplateBinding::Expression { key: None, angular, alias: None, .. }] => {
            return write_binding(angular.as_deref(), value, false, f);
        }
        // With an alias, it lays out as a binding's value, the alias in its layout;
        // with a pipe or a comment, which `Parts::dispatch` refuses, as the other bindings do
        [
            TemplateBinding::Expression {
                key: None,
                angular: Some(angular),
                alias: Some(alias),
                ..
            },
        ] => {
            let part =
                JsInAngularTemplate::Binding { tailwind_classes: false, alias: Some(*alias) };
            if let Some(parts) = Parts::dispatch(angular, part, f) {
                embed::write_quoted(&parts.format(), f);
                return true;
            }
        }
        _ => {}
    }
    // Each expression's, by binding
    let Some(parts) = bindings
        .iter()
        .map(|binding| match binding {
            // The directive's own key without an expression, which is none
            TemplateBinding::Expression { key: None, expression: None, .. } => None,
            TemplateBinding::Expression { expression: Some(_), angular, .. } => {
                let parts = Parts::dispatch(
                    angular.as_deref()?,
                    JsInAngularTemplate::AttributePiece { tailwind_classes: false },
                    f,
                )?;
                Some(Some(parts))
            }
            _ => Some(None),
        })
        .collect::<Option<Vec<_>>>()
    else {
        return false;
    };
    let directive_first = matches!(bindings[0], TemplateBinding::Expression { key: None, .. });
    let of_second = matches!(
        bindings.get(1),
        Some(TemplateBinding::Expression { key: Some(key), .. }) if key.slice(source) == "of"
    );
    let bindings = format_once(move |f| {
        for (i, (binding, parts)) in bindings.iter().zip(parts).enumerate() {
            let of = i == 1 && of_second;
            if i > 0 {
                if of {
                    write!(f, space());
                } else {
                    write!(f, [";", soft_line_break_or_space()]);
                }
            }
            match binding {
                TemplateBinding::Expression { key, alias, .. } => {
                    if let Some(key) = key.map(|key| key.slice(source)) {
                        // Prettier's `shouldNotPrintColon`; a key alone (`b` of `*ngIf="a; b"`) none
                        let no_colon = of
                            || (key == "track" && (of_second || (directive_first && i == 2)))
                            || (directive_first && matches!(key, "then" | "else" | "as"));
                        write!(f, text(key));
                        if parts.is_some() {
                            write!(f, if no_colon { " " } else { ": " });
                        }
                    }
                    if let Some(parts) = parts {
                        parts.write(f);
                    }
                    if let Some(alias) = alias {
                        write!(f, [ALIAS, text(alias.slice(source))]);
                    }
                }
                TemplateBinding::Let { name, value } => {
                    write!(f, ["let ", text(name.slice(source))]);
                    if let Some(value) = value {
                        write!(f, [" = ", text(value.slice(source))]);
                    }
                }
                TemplateBinding::As { key, alias } => {
                    write!(f, [text(key.slice(source)), ALIAS, text(alias.slice(source))]);
                }
            }
        }
    });
    // The items break with the value's group, every one on a line of its own (as Prettier prints them)
    embed::write_quoted(&soft_block_indent(&bindings), f);
    true
}

/// A block's parameters formatted, where every expression in them formats (`@if`'s, `@else if`'s, `@for`'s, `@switch`'s, `@case`'s):
/// in parens, `; ` between them, on lines of their own when they break (as Prettier prints them);
/// `None` to print them as written, as the other blocks' are, which hold no expression (`@defer (on viewport)`).
pub fn format_block_parameters<'a>(
    block: &'a Block<'a>,
    f: &HtmlishFormatter<'_, 'a>,
) -> Option<impl Format<'a, HtmlishFormatContext<'a>> + use<'a>> {
    if block.parameters.is_empty() {
        return None;
    }
    let source = f.context().tree().source();
    // Each expression's, by parameter
    let parts = block
        .parameters
        .iter()
        .map(|parameter| match &parameter.kind {
            BlockParameterKind::Plain(angular)
            | BlockParameterKind::ForOf { angular, .. }
            | BlockParameterKind::Track { angular, .. } => Some(Some(Parts::dispatch(
                angular.as_deref()?,
                JsInAngularTemplate::BlockPiece,
                f,
            )?)),
            BlockParameterKind::Let(_) | BlockParameterKind::As { .. } => Some(None),
        })
        .collect::<Option<Vec<_>>>()?;
    let parameters = format_once(move |f| {
        let mut join = f.join_with(SEPARATOR);
        for (parameter, parts) in block.parameters.iter().zip(parts) {
            join.entry(&format_once(move |f| {
                match &parameter.kind {
                    BlockParameterKind::Let(variables) => {
                        write!(f, "let ");
                        for (i, variable) in variables.iter().enumerate() {
                            if i > 0 {
                                write!(f, ", ");
                            }
                            let (name, value) =
                                (variable.name.slice(source), variable.value.slice(source));
                            write!(f, [text(name), " = ", text(value)]);
                        }
                    }
                    BlockParameterKind::As { alias } => {
                        write!(f, ["as ", text(alias.slice(source))]);
                    }
                    BlockParameterKind::ForOf { item, .. } => {
                        write!(f, [text(item.slice(source)), " of "]);
                    }
                    BlockParameterKind::Track { .. } => write!(f, "track "),
                    BlockParameterKind::Plain(_) => {}
                }
                if let Some(parts) = parts {
                    parts.write(f);
                }
            }));
        }
        join.finish();
    });
    Some(format_once(move |f| {
        write!(f, group(&format_args!("(", soft_block_indent(&parameters), ")")));
    }))
}

/// Writes `@let name = value;`, breaking after the `=` (as Prettier prints it),
/// its value formatted, or as written up to the `;` where it does not format (`embeddedLanguageFormatting: off`, a comment in it);
/// `end` is the declaration's, past its `;`.
/// `false` to print it all as written, where Angular reads no declaration: no name, an error in the value
/// (apps/oxfmt/DIVERGENCES.md#js-in-angular-unread-code-as-written).
/// The `;` ends it whatever `semi` says: Angular's lexer ends the value there.
pub fn write_let(
    declaration: &LetDeclaration<'_>,
    end: u32,
    f: &mut HtmlishFormatter<'_, '_>,
) -> bool {
    let source = f.context().tree().source();
    if declaration.name.is_empty() {
        return false;
    }
    let Some(angular) = declaration.angular.as_deref() else { return false };
    let name = declaration.name.slice(source);
    // A formatted comment would print after the `;`, in the text (apps/oxfmt/DIVERGENCES.md#js-in-angular-let-comment-as-written)
    let parts = if angular.comment.is_none() {
        Parts::dispatch(angular, JsInAngularTemplate::BlockPiece, f)
    } else {
        None
    };
    let value = format_once(|f| {
        if let Some(parts) = parts {
            write!(f, parts.format());
        } else {
            // Up to the `;` the declaration's `end` follows, the whitespace before it too: a line break that ends a line comment.
            // Its first line decides whether it breaks after the `=`, as Prettier measures it
            write!(
                f,
                text(&source[declaration.value.start as usize..end as usize - 1])
                    .without_expand_parent()
            );
        }
    });
    write!(
        f,
        group(&format_args!(
            "@let ",
            text(name),
            " =",
            group(&indent(&format_args!(soft_line_break_or_space(), value))),
            ";"
        ))
    );
    true
}

/// A comment the code ends in, which runs to its end: the host's next delimiter goes on a line of its own,
/// inside the quotes (apps/oxfmt/DIVERGENCES.md#js-in-angular-binding-comment-in-quotes).
fn write_comment<'a>(comment: Option<&'a str>, f: &mut HtmlishFormatter<'_, 'a>) {
    if let Some(comment) = comment {
        write!(f, [line_suffix(&format_args!(space(), text(comment))), expand_parent()]);
    }
}

/// An expression's pieces of code formatted, and its pipes' names and its comment as written.
struct Parts<'a> {
    code: Code<'a>,
    pipes: Vec<(&'a str, Vec<Code<'a>>)>,
    comment: Option<&'a str>,
}

/// A piece of code formatted; `conditional`: its root is a conditional ([`AngularCode::conditional`]).
struct Code<'a> {
    ir: Ir<'a>,
    conditional: bool,
    /// What its IR holds a placeholder of each for ([`Code::dispatch`]).
    replacements: Vec<Replacement<'a>>,
}

/// A nested pipe formatted ([`AngularCode::nested`]), which a placeholder in a piece of code's IR stands for,
/// in its parens where Angular needs them.
struct Replacement<'a> {
    parts: Parts<'a>,
    /// [`NestedPipe::parens`].
    parens: Option<NestedPipeParent>,
}

/// What a nested pipe is dispatched as: a name of its own, `oxfmt_ng_{index}_`.
/// Each `Code` checks its own IR for its own: a nested one's is dispatched on its own, and goes in resolved, not scanned again.
const PLACEHOLDER: Placeholder<'static> = Placeholder { prefix: "oxfmt_ng_", suffix: "_" };

impl<'a> Code<'a> {
    /// `code` formatted as `part`, each nested pipe dispatched as a name standing for it (TS reads `|` as a bitwise or);
    /// `before_pipe`: pipes follow it, a conditional's parens closing before them ([`Self::before_operator`]).
    /// `None` when a pipe does not format, or the formatted code does not hold each name once.
    /// Not complete: JS's layout takes each for a name, no break after a property's `:` or before a `.`
    /// (apps/oxfmt/DIVERGENCES.md#js-in-angular-nested-pipe-as-name).
    fn dispatch(
        code: &AngularCode<'_>,
        part: JsInAngularTemplate,
        before_pipe: bool,
        f: &HtmlishFormatter<'_, 'a>,
    ) -> Option<Self> {
        let closes_paren = before_pipe && code.conditional;
        let source = f.context().tree().source();
        let piece = code.span.slice(source);
        if code.nested.is_empty() {
            let ir = dispatch_piece(part, closes_paren, piece, f)?;
            return Some(Self { ir, conditional: code.conditional, replacements: vec![] });
        }
        if PLACEHOLDER.is_in(piece) {
            return None;
        }
        // A nested pipe is no binding's whole value, which hugs the quotes; nor what follows the code follows it
        let nested_part = match part {
            JsInAngularTemplate::Binding { tailwind_classes, .. } => {
                JsInAngularTemplate::AttributePiece { tailwind_classes }
            }
            part => part,
        };
        let mut replaced = String::with_capacity(piece.len());
        let mut replacements = Vec::with_capacity(code.nested.len());
        let mut at = code.span.start as usize;
        for NestedPipe { span, parens, last_value, spread, expression } in &code.nested {
            replaced.push_str(&source[at..span.start as usize]);
            PLACEHOLDER.write(&mut replaced, replacements.len());
            // The last value of an object: its `}` and the pipe's last may meet (`{k: a | p: {x: 1}}`), which ends a `{{ }}`.
            // In parens too, where they cannot, as Prettier parens it (`{k: (a | p: ({x: 1}))}`), but a spread's (`{...(a | p: {x: 1})}`)
            let before_brace = *last_value && !*spread;
            let parts = Parts::dispatch(expression, nested_part.before_brace(before_brace), f)?;
            // A spread's prints in parens, as Prettier parens it (`[...(a | p)]`)
            let parens = parens.or(spread.then_some(NestedPipeParent::Other));
            replacements.push(Replacement { parts, parens });
            at = span.end as usize;
        }
        replaced.push_str(&source[at..code.span.end as usize]);
        let ir = dispatch_piece(part, closes_paren, f.allocator().alloc_str(&replaced), f)?;
        // Each name once, in order: one dropped or printed twice would lose the pipe or repeat it
        PLACEHOLDER.are_sequential(&ir, replacements.len()).then_some(Self {
            ir,
            conditional: code.conditional,
            replacements,
        })
    }

    /// Its IR, each placeholder replaced with what it stands for, in every variant of a `BestFitting`.
    fn resolve(self, f: &mut HtmlishFormatter<'_, 'a>) -> Ir<'a> {
        if self.replacements.is_empty() {
            return self.ir;
        }
        let indent_width = f.options().indent_width;
        let replacements: Vec<_> = self
            .replacements
            .into_iter()
            .map(|Replacement { parts, parens }| match parens {
                Some(parent) => f.intern(&parts.format_in_parens(parent)),
                None => f.intern(&parts.format()),
            })
            .collect();
        map_text_in_ir(self.ir, f.allocator(), &mut |text, out| {
            if !PLACEHOLDER.is_in(text) {
                return false;
            }
            for piece in PLACEHOLDER.pieces(text) {
                match piece {
                    PlaceholderPiece::Text(text) => {
                        out.push(FormatElement::text(text, indent_width));
                    }
                    PlaceholderPiece::Index(index) => out.extend(replacements[index].clone()),
                }
            }
            true
        })
    }

    /// Before a pipe or an argument, a conditional prints in parens, whose branch would take it otherwise
    /// (`a ? b : c | p` is `a ? b : (c | p)`, prettier#5389);
    /// the `)` on a line of its own when it breaks before a pipe, JS's to break ([`JsInAngularPiece::closes_paren`]),
    /// after the last branch before a `:`, as Prettier prints them.
    fn before_operator(self) -> impl Format<'a, HtmlishFormatContext<'a>> {
        format_once(move |f| {
            let conditional = self.conditional;
            let ir = self.resolve(f);
            if conditional {
                write!(f, ["(", format_ir(ir), ")"]);
            } else {
                f.write_elements(ir);
            }
        })
    }
}

impl<'a> Parts<'a> {
    /// Every piece of code dispatched as `part`; `None` when one prints as written.
    fn dispatch(
        expression: &AngularExpression<'_>,
        part: JsInAngularTemplate,
        f: &HtmlishFormatter<'_, 'a>,
    ) -> Option<Self> {
        let source = f.context().tree().source();
        let comment = expression.comment.map(|comment| comment.slice(source));
        // A binding's value hugs the quotes only as a whole: none of its pieces does when pipes or a comment follow
        let part = match part {
            JsInAngularTemplate::Binding { tailwind_classes, alias }
                if !expression.pipes.is_empty() || comment.is_some() =>
            {
                // `None` for an alias: the caller's microsyntax path prints it after the pipes
                if alias.is_some() {
                    return None;
                }
                JsInAngularTemplate::AttributePiece { tailwind_classes }
            }
            part => part,
        };
        // What follows the expression follows its last piece only: the code, or the last pipe's last argument
        let inner = part.before_brace(false);
        let part_of = |last: bool| if last { part } else { inner };
        let code_part = part_of(expression.pipes.is_empty());
        // Pipes in parens, all of the code: the pipes after them go on, as Angular reads them (`(a | p) | q` is `a | p | q`)
        let (code, mut pipes) = match expression.code.nested.as_slice() {
            [nested] if nested.span == expression.code.span => {
                let Self { code, pipes, .. } = Self::dispatch(&nested.expression, code_part, f)?;
                (code, pipes)
            }
            _ => (
                Code::dispatch(&expression.code, code_part, !expression.pipes.is_empty(), f)?,
                Vec::with_capacity(expression.pipes.len()),
            ),
        };
        for (i, pipe) in expression.pipes.iter().enumerate() {
            let last_pipe = i + 1 == expression.pipes.len();
            let arguments = pipe
                .arguments
                .iter()
                .enumerate()
                .map(|(j, argument)| {
                    Code::dispatch(
                        argument,
                        part_of(last_pipe && j + 1 == pipe.arguments.len()),
                        false,
                        f,
                    )
                })
                .collect::<Option<Vec<_>>>()?;
            pipes.push((pipe.name.slice(source), arguments));
        }
        Some(Self { code, pipes, comment })
    }

    fn format(self) -> impl Format<'a, HtmlishFormatContext<'a>> {
        format_once(move |f| self.write(f))
    }

    /// In parens of its own, as JS prints a binary's in them, by what they are part of (as Prettier prints them):
    /// - a call's callee, a member's object or a unary's argument: on lines of their own, its pipes lined up with its code
    /// - a conditional that is no call's argument: hugging it, its pipes lined up with its code
    /// - anything else: hugging it, its pipes indented
    ///   (an object's value too, where Prettier lines them up: after a `k:` JS keeps on its line, they indent past it, as Prettier's do)
    fn format_in_parens(
        self,
        parent: NestedPipeParent,
    ) -> impl Format<'a, HtmlishFormatContext<'a>> {
        format_once(move |f| match parent {
            NestedPipeParent::Callee
            | NestedPipeParent::MemberObject { computed: false }
            | NestedPipeParent::UnaryArgument => {
                let parts = format_once(move |f| self.write_with(false, f));
                write!(f, ["(", group(&soft_block_indent(&parts)), ")"]);
            }
            NestedPipeParent::Conditional { call_argument: false } => {
                write!(f, ["(", format_once(move |f| self.write_with(false, f)), ")"]);
            }
            NestedPipeParent::Conditional { call_argument: true }
            | NestedPipeParent::MemberObject { computed: true }
            | NestedPipeParent::ObjectValue
            | NestedPipeParent::Other => write!(f, ["(", self.format(), ")"]),
        })
    }

    /// The code, then each pipe on a line of its own when they break, its arguments too (as Prettier prints them);
    /// a lone pipe breaks on its own (`f(⏎…⏎) | async`).
    /// The code's own lines do not indent: they read as one operand
    /// (apps/oxfmt/DIVERGENCES.md#js-in-angular-pipe-operand-as-root).
    fn write(self, f: &mut HtmlishFormatter<'_, 'a>) {
        self.write_with(true, f);
    }

    /// [`Self::write`], its pipes indented or not (`indent_pipes`).
    fn write_with(self, indent_pipes: bool, f: &mut HtmlishFormatter<'_, 'a>) {
        let Self { code, pipes, comment } = self;
        if pipes.is_empty() {
            // Nothing follows the code but the comment, if any
            let ir = code.resolve(f);
            f.write_elements(ir);
        } else {
            let lone = pipes.len() == 1;
            let chain = format_once(move |f| {
                write!(f, code.before_operator());
                let pipes = format_once(move |f| {
                    for (name, arguments) in pipes {
                        write_pipe(name, arguments, lone, f);
                    }
                });
                if indent_pipes {
                    write!(f, indent(&pipes));
                } else {
                    write!(f, pipes);
                }
            });
            write!(f, group(&chain));
        }
        write_comment(comment, f);
    }
}

/// `| name: argument : argument`, each `: argument` on a line of its own when they break;
/// `lone`: the only pipe, in a group of its own.
fn write_pipe<'a>(
    name: &'a str,
    arguments: Vec<Code<'a>>,
    lone: bool,
    f: &mut HtmlishFormatter<'_, 'a>,
) {
    let pipe = format_once(move |f| {
        write!(f, [soft_line_break_or_space(), "| ", text(name)]);
        if arguments.is_empty() {
            return;
        }
        let arguments = format_once(move |f| {
            write!(f, [soft_line_break(), ": "]);
            for (i, argument) in arguments.into_iter().enumerate() {
                if i > 0 {
                    write!(f, [soft_line_break_or_space(), ": "]);
                }
                // Every argument as Prettier prints it, the last one too
                write!(f, align(2, &group(&argument.before_operator())));
            }
        });
        write!(f, group(&indent(&arguments)));
    });
    if lone {
        write!(f, group(&pipe));
    } else {
        write!(f, pipe);
    }
}
