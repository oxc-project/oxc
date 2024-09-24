#![allow(clippy::must_use_candidate)]
#![allow(clippy::pedantic)]
#![allow(clippy::nursery)]
#![allow(dead_code)]

/// origin file: https://github.com/zkat/miette/blob/75fea0935e495d0215518c80d32dd820910982e3/src/handlers/theme.rs
use miette::ThemeCharacters;
use owo_colors::Style;

/**
Theme used by [`GraphicalReportHandler`](crate::GraphicalReportHandler) to
render fancy [`Diagnostic`](crate::Diagnostic) reports.

A theme consists of two things: the set of characters to be used for drawing,
and the
[`owo_colors::Style`](https://docs.rs/owo-colors/latest/owo_colors/struct.Style.html)s to be used to paint various items.

You can create your own custom graphical theme using this type, or you can use
one of the predefined ones using the methods below.

When created by [`Default::default`], themes are automatically selected based on the `NO_COLOR`
environment variable and whether the process is running in a terminal.
*/
#[derive(Debug, Clone)]
pub struct GraphicalTheme {
    /// Characters to be used for drawing.
    pub characters: ThemeCharacters,
    /// Styles to be used for painting.
    pub styles: ThemeStyles,
}

impl GraphicalTheme {
    pub fn new(is_terminal: bool) -> Self {
        match std::env::var("NO_COLOR") {
            _ if !is_terminal => Self::none(),
            Ok(string) if string != "0" => Self::unicode_nocolor(),
            _ => Self::unicode(),
        }
    }

    /// ASCII-art-based graphical drawing, with ANSI styling.
    pub fn ascii() -> Self {
        Self { characters: ThemeCharacters::ascii(), styles: ThemeStyles::ansi() }
    }

    /// Graphical theme that draws using both ansi colors and unicode
    /// characters.
    ///
    /// Note that full rgb colors aren't enabled by default because they're
    /// an accessibility hazard, especially in the context of terminal themes
    /// that can change the background color and make hardcoded colors illegible.
    /// Such themes typically remap ansi codes properly, treating them more
    /// like CSS classes than specific colors.
    pub fn unicode() -> Self {
        Self { characters: ThemeCharacters::unicode(), styles: ThemeStyles::rgb() }
    }

    /// Graphical theme that draws in monochrome, while still using unicode
    /// characters.
    pub fn unicode_nocolor() -> Self {
        Self { characters: ThemeCharacters::unicode(), styles: ThemeStyles::none() }
    }

    /// A "basic" graphical theme that skips colors and unicode characters and
    /// just does monochrome ascii art. If you want a completely non-graphical
    /// rendering of your [`Diagnostic`](crate::Diagnostic)s
    pub fn none() -> Self {
        Self { characters: ThemeCharacters::ascii(), styles: ThemeStyles::none() }
    }
}

/**
Styles for various parts of graphical rendering for the
[`GraphicalReportHandler`](crate::GraphicalReportHandler).
*/
#[derive(Debug, Clone)]
pub struct ThemeStyles {
    /// Style to apply to things highlighted as "error".
    pub error: Style,
    /// Style to apply to things highlighted as "warning".
    pub warning: Style,
    /// Style to apply to things highlighted as "advice".
    pub advice: Style,
    /// Style to apply to the help text.
    pub help: Style,
    /// Style to apply to filenames/links/URLs.
    pub link: Style,
    /// Style to apply to line numbers.
    pub linum: Style,
    /// Styles to cycle through (using `.iter().cycle()`), to render the lines
    /// and text for diagnostic highlights.
    pub highlights: Vec<Style>,
}

fn style() -> Style {
    Style::new()
}

impl ThemeStyles {
    /// Nice RGB colors.
    /// [Credit](http://terminal.sexy/#FRUV0NDQFRUVrEFCkKlZ9L91ap-1qnWfdbWq0NDQUFBQrEFCkKlZ9L91ap-1qnWfdbWq9fX1).
    pub fn rgb() -> Self {
        Self {
            error: style().fg_rgb::<225, 80, 80>().bold(), // CHANGED: <255, 30, 30>
            warning: style().fg_rgb::<244, 191, 117>().bold(),
            advice: style().fg_rgb::<106, 159, 181>(),
            help: style().fg_rgb::<106, 159, 181>(),
            link: style().fg_rgb::<92, 157, 255>().bold(),
            linum: style().dimmed(),
            highlights: vec![
                style().fg_rgb::<246, 87, 248>(),
                style().fg_rgb::<30, 201, 212>(),
                style().fg_rgb::<145, 246, 111>(),
            ],
        }
    }

    /// ANSI color-based styles.
    pub fn ansi() -> Self {
        Self {
            error: style().red(),
            warning: style().yellow(),
            advice: style().cyan(),
            help: style().cyan(),
            link: style().cyan().underline().bold(),
            linum: style().dimmed(),
            highlights: vec![
                style().magenta().bold(),
                style().yellow().bold(),
                style().green().bold(),
            ],
        }
    }

    /// No styling. Just regular ol' monochrome.
    pub fn none() -> Self {
        Self {
            error: style(),
            warning: style(),
            advice: style(),
            help: style(),
            link: style(),
            linum: style(),
            highlights: vec![style()],
        }
    }
}
