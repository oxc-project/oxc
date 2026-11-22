//! Angular: the JS of a template (`{{ }}`, `[a]`, `[(a)]`, `(a)`, `*a`, `{{ }}` in another attribute's value, block parameters, `@let`),
//! formatted by the session's dispatcher as TS fragments, split as the parser reads it ([`AngularExpression`] and the rest):
//! each piece of code is dispatched, Angular's own syntax around them (pipes, comments, `;`, microsyntax, block parameters) prints here.
//! A piece the dispatcher does not format leaves what holds it as written: the value, a block's parameters, a `{{ }}`.

use oxc_allocator::ArenaVec;
use oxc_formatter_core::{
    Format, FormatElement,
    builders::{
        FormatWith, align, expand_parent, group, indent, line_suffix, soft_block_indent,
        soft_line_break, soft_line_break_or_space, soft_space_or_block_indent, space, text,
    },
    dispatch_fragment_ir, format_args, write,
};
use oxc_htmlish_parser::ast::{
    AngularAction, AngularCode, AngularExpression, Attribute, Block, BlockParameterKind,
    LetDeclaration, TemplateBinding, ValueInterpolation,
};

use super::{
    HtmlishFormatter,
    attribute::{quoted_piece, unescaped},
    embed, format_ir, format_once, format_with,
};
use crate::context::HtmlishFormatContext;

/// Parent→child context for a JS fragment of an Angular template, dispatched as TS.
///
/// The host translates it into JS's `JsRoot` and `JsEmbeddedIn`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JsInAngularTemplate {
    /// A piece of code in `{{ }}` (all of it, but its pipes and comment): an expression the interpolation lays out.
    Interpolation,
    /// A binding's value (`[a]`, `[(a)]`, `bind-a`, a lone expression of `*a`) without a pipe or a comment:
    /// an expression in a `"`-quoted attribute value, whose strings print in `'`;
    /// an object, an array, a string or a template literal hugs the quotes.
    /// `tailwind_classes`: its strings are Tailwind classes (`TailwindAttributes::binds`).
    Binding { tailwind_classes: bool },
    /// A `{{ }}` in another attribute's value (`title="Hi {{ name }}"`), or a piece of its code: an expression it lays out.
    ValueInterpolation,
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
        !matches!(self, Self::Interpolation | Self::BlockPiece)
    }
}

/// `;` and a line, between statements or a block's parameters.
pub(super) const SEPARATOR: FormatWith<fn(&mut HtmlishFormatter<'_, '_>)> =
    format_with(|f| write!(f, [";", soft_line_break_or_space()]));

/// A piece of code formatted as `part`; `None` when the dispatcher leaves it as written.
fn dispatch_piece<'a>(
    part: JsInAngularTemplate,
    piece: &'a str,
    f: &HtmlishFormatter<'_, 'a>,
) -> Option<Ir<'a>> {
    if part.in_html_attribute() {
        embed::dispatch_attribute_value("ts", &part, unescaped(piece, f), f)
    } else {
        dispatch_fragment_ir(f, "ts", piece, Some(&part))
    }
}

type Ir<'a> = ArenaVec<'a, FormatElement<'a>>;

/// `{{ }}`'s expression formatted; `None` to print it as written.
pub fn format_interpolation<'a>(
    expression: Option<&AngularExpression<'_>>,
    f: &HtmlishFormatter<'_, 'a>,
) -> Option<impl Format<'a, HtmlishFormatContext<'a>> + use<'a>> {
    Some(Parts::dispatch(expression?, JsInAngularTemplate::Interpolation, f)?.format())
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
    // The whole value, its quote entities read back by the caller, which an object or an array hugs
    if expression.pipes.is_empty() && expression.comment.is_none() {
        let part = JsInAngularTemplate::Binding { tailwind_classes };
        return embed::write_attribute_value("ts", &part, value, f);
    }
    let part = JsInAngularTemplate::AttributePiece { tailwind_classes };
    let Some(parts) = Parts::dispatch(expression, part, f) else {
        return false;
    };
    embed::write_quoted(&soft_block_indent(&parts.format()), f);
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
            dispatch_piece(part, statement.span.slice(source), f)
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
                Parts::dispatch(expression, JsInAngularTemplate::ValueInterpolation, f)?,
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
/// `;` between them (a space before `of`), a `:` after a key but `of`, `then`, `else` and `track`;
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
                        write!(f, [" as ", text(alias.slice(source))]);
                    }
                }
                TemplateBinding::Let { name, value } => {
                    write!(f, ["let ", text(name.slice(source))]);
                    if let Some(value) = value {
                        write!(f, [" = ", text(value.slice(source))]);
                    }
                }
                TemplateBinding::As { key, alias } => {
                    write!(f, [text(key.slice(source)), " as ", text(alias.slice(source))]);
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

/// Writes `@let name = value;` formatted, breaking after the `=` (as Prettier prints it);
/// `false` to print it as written.
/// The `;` ends it whatever `semi` says: Angular's lexer ends the value there.
pub fn write_let(declaration: &LetDeclaration<'_>, f: &mut HtmlishFormatter<'_, '_>) -> bool {
    let source = f.context().tree().source();
    // A comment would print after the `;`, in the text
    let Some(angular) = declaration.angular.as_deref().filter(|e| e.comment.is_none()) else {
        return false;
    };
    let Some(parts) = Parts::dispatch(angular, JsInAngularTemplate::BlockPiece, f) else {
        return false;
    };
    let name = declaration.name.slice(source);
    let value = parts.format();
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

/// A comment the code ends in, which runs to its end: the host's next delimiter goes on a line of its own.
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
}

impl<'a> Code<'a> {
    /// Before a pipe or an argument, a conditional prints in parens, whose branch would take it otherwise
    /// (`a ? b : c | p` is `a ? b : (c | p)`, prettier#5389);
    /// the `)` on a line of its own when it breaks before a pipe (`own_line`), after the last branch before a `:`, as Prettier prints them.
    fn before_operator(self, own_line: bool) -> impl Format<'a, HtmlishFormatContext<'a>> {
        format_once(move |f| {
            if self.conditional {
                let close = own_line.then_some(soft_line_break());
                write!(f, group(&format_args!("(", format_ir(self.ir), close, ")")));
            } else {
                f.write_elements(self.ir);
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
        let code = |code: &AngularCode| {
            let ir = dispatch_piece(part, code.span.slice(source), f)?;
            Some(Code { ir, conditional: code.conditional })
        };
        let mut pipes = Vec::with_capacity(expression.pipes.len());
        for pipe in &expression.pipes {
            let arguments = pipe.arguments.iter().map(code).collect::<Option<Vec<_>>>()?;
            pipes.push((pipe.name.slice(source), arguments));
        }
        Some(Self {
            code: code(&expression.code)?,
            pipes,
            comment: expression.comment.map(|comment| comment.slice(source)),
        })
    }

    fn format(self) -> impl Format<'a, HtmlishFormatContext<'a>> {
        format_once(move |f| self.write(f))
    }

    /// The code, then each pipe on a line of its own when they break, its arguments too (as Prettier prints them);
    /// a lone pipe breaks on its own (`f(⏎…⏎) | async`).
    /// The code's own lines do not indent: they read as one operand
    /// (apps/oxfmt/DIVERGENCES.md#angular-pipe-operand-as-root, written with the switch to native Angular).
    fn write(self, f: &mut HtmlishFormatter<'_, 'a>) {
        let Self { code, pipes, comment } = self;
        if pipes.is_empty() {
            // Nothing follows the code but the comment, if any
            f.write_elements(code.ir);
        } else {
            let lone = pipes.len() == 1;
            let chain = format_once(move |f| {
                write!(f, code.before_operator(true));
                let pipes = format_once(move |f| {
                    for (name, arguments) in pipes {
                        write_pipe(name, arguments, lone, f);
                    }
                });
                write!(f, indent(&pipes));
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
                write!(f, align(2, &group(&argument.before_operator(false))));
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
