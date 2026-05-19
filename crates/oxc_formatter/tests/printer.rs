use oxc_allocator::Allocator;

use oxc_formatter::format_args;
use oxc_formatter::formatter::prelude::document::Document;
use oxc_formatter::formatter::prelude::*;
use oxc_formatter::formatter::printer::{PrintWidth, Printer, PrinterOptions};
use oxc_formatter::formatter::{
    FormatState, Formatter, JsFormatContext, JsFormatter, Printed, VecBuffer,
};
use oxc_formatter::write;
use oxc_formatter::{IndentStyle, LineEnding};

fn format<'a>(allocator: &'a Allocator, root: &dyn Format<'a, JsFormatContext<'a>>) -> Printed {
    format_with_options(
        allocator,
        root,
        PrinterOptions {
            indent_style: IndentStyle::Space,
            indent_width: 2.try_into().unwrap(),
            line_ending: LineEnding::Lf,
            ..PrinterOptions::default()
        },
    )
}

fn format_with_options<'a>(
    allocator: &'a Allocator,
    root: &dyn Format<'a, JsFormatContext<'a>>,
    options: PrinterOptions,
) -> Printed {
    let formatted = oxc_formatter::format!(JsFormatContext::dummy(allocator), [root]);

    Printer::new(options, &[]).print(formatted.document()).expect("Document to be valid")
}

#[test]
fn it_prints_a_group_on_a_single_line_if_it_fits() {
    let allocator = Allocator::default();
    let result = format(
        &allocator,
        &FormatArrayElements {
            items: vec![&token("\"a\""), &token("\"b\""), &token("\"c\""), &token("\"d\"")],
        },
    );

    assert_eq!(r#"["a", "b", "c", "d"]"#, result.as_code());
}

#[test]
fn it_tracks_the_indent_for_each_token() {
    let allocator = Allocator::default();
    let formatted = format(
        &allocator,
        &format_args!(
            token("a"),
            soft_block_indent(&format_args!(
                token("b"),
                soft_block_indent(&format_args!(
                    token("c"),
                    soft_block_indent(&format_args!(token("d"), soft_line_break(), token("d"),)),
                    token("c"),
                )),
                token("b"),
            )),
            token("a")
        ),
    );

    assert_eq!(
        r"a
  b
    c
      d
      d
    c
  b
a",
        formatted.as_code()
    );
}

#[test]
fn it_converts_line_endings() {
    let allocator = Allocator::default();
    let options = PrinterOptions {
        indent_style: IndentStyle::Tab,
        line_ending: LineEnding::Crlf,
        ..PrinterOptions::default()
    };

    let result = format_with_options(
        &allocator,
        &format_args!(
            token("function main() {"),
            block_indent(&text("let x = `This is a multiline\nstring`;")),
            token("}"),
            hard_line_break()
        ),
        options,
    );

    assert_eq!(
        "function main() {\r\n\tlet x = `This is a multiline\r\nstring`;\r\n}\r\n",
        result.as_code()
    );
}

#[test]
fn it_converts_line_endings_to_cr() {
    let allocator = Allocator::default();
    let options = PrinterOptions {
        indent_style: IndentStyle::Tab,
        line_ending: LineEnding::Cr,
        ..PrinterOptions::default()
    };

    let result = format_with_options(
        &allocator,
        &format_args!(
            token("function main() {"),
            block_indent(&text("let x = `This is a multiline\nstring`;")),
            token("}"),
            hard_line_break()
        ),
        options,
    );

    assert_eq!(
        "function main() {\r\tlet x = `This is a multiline\rstring`;\r}\r",
        result.as_code()
    );
}

#[test]
fn it_breaks_a_group_if_a_string_contains_a_newline() {
    let allocator = Allocator::default();
    let result = format(
        &allocator,
        &FormatArrayElements {
            items: vec![&text("`This is a string spanning\ntwo lines`"), &token("\"b\"")],
        },
    );

    assert_eq!(
        r#"[
  `This is a string spanning
two lines`,
  "b",
]"#,
        result.as_code()
    );
}
#[test]
fn it_breaks_a_group_if_it_contains_a_hard_line_break() {
    let allocator = Allocator::default();
    let result = format(&allocator, &group(&format_args!(token("a"), block_indent(&token("b")))));

    assert_eq!("a\n  b\n", result.as_code());
}

#[test]
fn it_breaks_parent_groups_if_they_dont_fit_on_a_single_line() {
    let allocator = Allocator::default();
    let result = format(
        &allocator,
        &FormatArrayElements {
            items: vec![
                &token("\"a\""),
                &token("\"b\""),
                &token("\"c\""),
                &token("\"d\""),
                &FormatArrayElements {
                    items: vec![
                        &token("\"0123456789\""),
                        &token("\"0123456789\""),
                        &token("\"0123456789\""),
                        &token("\"0123456789\""),
                        &token("\"0123456789\""),
                        &token("\"0123456789\""),
                    ],
                },
            ],
        },
    );

    assert_eq!(
        r#"[
  "a",
  "b",
  "c",
  "d",
  ["0123456789", "0123456789", "0123456789", "0123456789", "0123456789", "0123456789"],
]"#,
        result.as_code()
    );
}

#[test]
fn it_use_the_indent_character_specified_in_the_options() {
    let allocator = Allocator::default();
    let options = PrinterOptions {
        indent_style: IndentStyle::Tab,
        indent_width: 2.try_into().unwrap(),
        print_width: PrintWidth::new(19),
        ..PrinterOptions::default()
    };

    let result = format_with_options(
        &allocator,
        &FormatArrayElements {
            items: vec![&token("'a'"), &token("'b'"), &token("'c'"), &token("'d'")],
        },
        options,
    );

    assert_eq!("[\n\t'a',\n\t\'b',\n\t\'c',\n\t'd',\n]", result.as_code());
}

#[test]
fn it_prints_consecutive_hard_lines_as_one() {
    let allocator = Allocator::default();
    let result = format(
        &allocator,
        &format_args!(
            token("a"),
            hard_line_break(),
            hard_line_break(),
            hard_line_break(),
            token("b"),
        ),
    );

    assert_eq!("a\nb", result.as_code());
}

#[test]
fn it_prints_consecutive_empty_lines_as_one() {
    let allocator = Allocator::default();
    let result = format(
        &allocator,
        &format_args!(token("a"), empty_line(), empty_line(), empty_line(), token("b"),),
    );

    assert_eq!("a\n\nb", result.as_code());
}

#[test]
fn it_prints_consecutive_mixed_lines_as_one() {
    let allocator = Allocator::default();
    let result = format(
        &allocator,
        &format_args!(
            token("a"),
            empty_line(),
            hard_line_break(),
            empty_line(),
            hard_line_break(),
            token("b"),
        ),
    );

    assert_eq!("a\n\nb", result.as_code());
}

#[test]
fn test_fill_breaks() {
    let allocator = Allocator::default();
    let mut state = FormatState::new(JsFormatContext::dummy(&allocator), &allocator);
    let mut buffer = VecBuffer::new(&mut state);
    let mut formatter = Formatter::new(&mut buffer);

    formatter
        .fill()
        // These all fit on the same line together
        .entry(&soft_line_break_or_space(), &format_args!(token("1"), token(",")))
        .entry(&soft_line_break_or_space(), &format_args!(token("2"), token(",")))
        .entry(&soft_line_break_or_space(), &format_args!(token("3"), token(",")))
        // This one fits on a line by itself,
        .entry(&soft_line_break_or_space(), &format_args!(token("723493294"), token(",")))
        // fits without breaking
        .entry(
            &soft_line_break_or_space(),
            &group(&format_args!(token("["), soft_block_indent(&token("5")), token("],"))),
        )
        // this one must be printed in expanded mode to fit
        .entry(
            &soft_line_break_or_space(),
            &group(&format_args!(token("["), soft_block_indent(&token("123456789")), token("]"),)),
        )
        .finish();

    let document = Document::new(buffer.into_vec(), Vec::default());

    let printed = Printer::new(
        PrinterOptions::default()
            .with_indent_style(IndentStyle::Tab)
            .with_print_width(PrintWidth::new(10)),
        &[],
    )
    .print(&document)
    .unwrap();

    assert_eq!(printed.as_code(), "1, 2, 3,\n723493294,\n[5],\n[\n\t123456789\n]");
}

#[test]
fn line_suffix_printed_at_end() {
    let allocator = Allocator::default();
    let printed = format(
        &allocator,
        &format_args!(
            group(&format_args!(
                token("["),
                soft_block_indent(&format_with(|f| {
                    f.fill()
                        .entry(&soft_line_break_or_space(), &format_args!(token("1"), token(",")))
                        .entry(&soft_line_break_or_space(), &format_args!(token("2"), token(",")))
                        .entry(
                            &soft_line_break_or_space(),
                            &format_args!(token("3"), if_group_breaks(&token(","))),
                        )
                        .finish();
                })),
                token("]")
            )),
            token(";"),
            &line_suffix(&format_args!(space(), token("// trailing"), space()))
        ),
    );

    assert_eq!(printed.as_code(), "[1, 2, 3]; // trailing");
}
#[test]
fn conditional_with_group_id_in_fits() {
    let allocator = Allocator::default();
    let content = format_with(|f| {
        let group_id = f.group_id("test");
        write!(
            f,
            [
                group(&format_args!(
                    token("The referenced group breaks."),
                    hard_line_break()
                ))
                .with_group_id(Some(group_id)),
                group(&format_args!(
                    token("This group breaks because:"),
                    soft_line_break_or_space(),
                    if_group_fits_on_line(&token("This content fits but should not be printed.")).with_group_id(Some(group_id)),
                    if_group_breaks(&token("It measures with the 'if_group_breaks' variant because the referenced group breaks and that's just way too much text.")).with_group_id(Some(group_id)),
                ))
            ]
        );
    });

    let printed = format(&allocator, &content);

    assert_eq!(
        printed.as_code(),
        "The referenced group breaks.\nThis group breaks because:\nIt measures with the 'if_group_breaks' variant because the referenced group breaks and that's just way too much text."
    );
}

#[test]
fn out_of_order_group_ids() {
    let allocator = Allocator::default();
    let content = format_with(|f| {
        let id_1 = f.group_id("id-1");
        let id_2 = f.group_id("id-2");

        write!(f, [group(&token("Group with id-2")).with_group_id(Some(id_2)), hard_line_break()]);

        write!(f,
        [
            group(&token("Group with id-1 does not fit on the line because it exceeds the line width of 100 characters by..........")).with_group_id(Some(id_1)),
            hard_line_break()
        ]);

        write!(
            f,
            [
                if_group_fits_on_line(&token("Group 2 fits")).with_group_id(Some(id_2)),
                hard_line_break(),
                if_group_breaks(&token("Group 1 breaks")).with_group_id(Some(id_1))
            ]
        );
    });

    let printed = format(&allocator, &content);

    assert_eq!(
        printed.as_code(),
        r"Group with id-2
Group with id-1 does not fit on the line because it exceeds the line width of 100 characters by..........
Group 2 fits
Group 1 breaks"
    );
}

#[test]
fn break_group_if_partial_string_exceeds_print_width() {
    let allocator = Allocator::default();
    let options = PrinterOptions { print_width: PrintWidth::new(10), ..PrinterOptions::default() };

    let result = format_with_options(
        &allocator,
        &format_args!(group(&format_args!(
            token("("),
            soft_line_break(),
            text("This is a string\n containing a newline"),
            soft_line_break(),
            token(")")
        ))),
        options,
    );

    assert_eq!("(\nThis is a string\n containing a newline\n)", result.as_code());
}

struct FormatArrayElements<'a> {
    items: Vec<&'a dyn Format<'a, JsFormatContext<'a>>>,
}

impl<'a> Format<'a, JsFormatContext<'a>> for FormatArrayElements<'a> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        write!(
            f,
            [group(&format_args!(
                token("["),
                soft_block_indent(&format_args!(
                    format_with(|f| {
                        f.join_with(format_args!(token(","), soft_line_break_or_space()))
                            .entries(&self.items);
                    }),
                    if_group_breaks(&token(",")),
                )),
                token("]")
            ))]
        );
    }
}
