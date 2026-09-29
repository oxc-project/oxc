use oxc_allocator::Allocator;
use oxc_formatter::{Expand, JsFormatOptions, Semicolons, TrailingCommas};
use oxc_formatter_core::{IndentStyle, LineWidth};
use oxc_span::SourceType;

// The snapshot harness records idempotency failures instead of asserting them.
// Exercise the regression fixtures across width boundaries and assert the fixpoint directly.
#[test]
fn breaking_last_group_is_idempotent() {
    let fixtures = [
        include_str!("fixtures/js/member-chains/breaking-last-group/objects.js"),
        include_str!("fixtures/js/member-chains/breaking-last-group/arguments.js"),
        include_str!("fixtures/js/member-chains/breaking-last-group/typescript.tsx"),
    ];

    for (fixture_index, source) in fixtures.into_iter().enumerate() {
        for width in 40..=120 {
            for expand in [Expand::Auto, Expand::Never] {
                for tabs in [false, true] {
                    let options = JsFormatOptions {
                        line_width: LineWidth::try_from(width).unwrap(),
                        expand,
                        indent_style: if tabs { IndentStyle::Tab } else { IndentStyle::Space },
                        semicolons: if tabs { Semicolons::AsNeeded } else { Semicolons::Always },
                        trailing_commas: if tabs {
                            TrailingCommas::None
                        } else {
                            TrailingCommas::All
                        },
                        ..JsFormatOptions::default()
                    };
                    let format = |source: &str| {
                        let allocator = Allocator::default();
                        oxc_formatter::format(
                            &allocator,
                            source,
                            SourceType::tsx(),
                            options.clone(),
                        )
                        .unwrap()
                        .print()
                        .unwrap()
                        .into_code()
                    };

                    let first = format(source);
                    let second = format(&first);
                    assert_eq!(
                        first, second,
                        "fixture {fixture_index}, width {width}, expand {expand:?}, tabs {tabs}"
                    );
                }
            }
        }
    }
}
