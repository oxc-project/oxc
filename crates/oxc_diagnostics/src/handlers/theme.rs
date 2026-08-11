use std::{
    env,
    io::{self, IsTerminal},
};

use owo_colors::Style;

/// Theme used by [`GraphicalReportHandler`](crate::GraphicalReportHandler).
///
/// Use one of the predefined constructors below.
#[derive(Debug, Clone)]
pub struct GraphicalTheme {
    pub(crate) characters: ThemeCharacters,
    pub(crate) styles: ThemeStyles,
}

fn force_color() -> bool {
    // Assume CI can always print colors.
    env::var("CI").is_ok() || env::var("FORCE_COLOR").is_ok_and(|env| env != "0")
}

impl Default for GraphicalTheme {
    fn default() -> Self {
        Self::new(io::stdout().is_terminal() && io::stderr().is_terminal())
    }
}

impl GraphicalTheme {
    /// Chooses a graphical theme based on terminal and environment support.
    #[must_use]
    pub(crate) fn new(is_terminal: bool) -> Self {
        if force_color() {
            return Self::unicode();
        }
        match env::var("NO_COLOR") {
            _ if !is_terminal => Self::none(),
            Ok(string) if string != "0" => Self::unicode_nocolor(),
            _ => Self::unicode(),
        }
    }

    /// Graphical theme that draws using both ansi colors and unicode
    /// characters.
    ///
    /// Note that full rgb colors aren't enabled by default because they're
    /// an accessibility hazard, especially in the context of terminal themes
    /// that can change the background color and make hardcoded colors illegible.
    /// Such themes typically remap ansi codes properly, treating them more
    /// like CSS classes than specific colors.
    #[must_use]
    pub fn unicode() -> Self {
        Self { characters: ThemeCharacters::unicode(), styles: ThemeStyles::rgb() }
    }

    /// Graphical theme that draws in monochrome, while still using unicode
    /// characters.
    #[must_use]
    pub fn unicode_nocolor() -> Self {
        Self { characters: ThemeCharacters::unicode(), styles: ThemeStyles::none() }
    }

    /// A "basic" graphical theme that skips colors and unicode characters and
    /// just does monochrome ASCII art.
    #[must_use]
    pub fn none() -> Self {
        Self { characters: ThemeCharacters::ascii(), styles: ThemeStyles::none() }
    }

    /// Style used for warning text.
    #[must_use]
    pub const fn warning_style(&self) -> Style {
        self.styles.warning
    }
}

#[derive(Debug, Clone)]
#[expect(clippy::redundant_pub_crate, reason = "prevents public glob re-export")]
pub(crate) struct ThemeStyles {
    pub(crate) error: Style,
    pub(crate) warning: Style,
    pub(crate) advice: Style,
    pub(crate) help: Style,
    pub(crate) note: Style,
    pub(crate) link: Style,
    pub(crate) linum: Style,
    pub(crate) highlights: [Style; 3],
}

fn style() -> Style {
    Style::new()
}

impl ThemeStyles {
    fn rgb() -> Self {
        Self {
            error: style().fg_rgb::<225, 80, 80>().bold(), // CHANGED: <255, 30, 30>
            warning: style().fg_rgb::<244, 191, 117>().bold(),
            advice: style().fg_rgb::<106, 159, 181>(),
            help: style().fg_rgb::<106, 159, 181>(),
            note: style().fg_rgb::<106, 159, 181>(),
            link: style().fg_rgb::<92, 157, 255>().bold(),
            linum: style().dimmed(),
            highlights: [
                style().fg_rgb::<246, 87, 248>(),
                style().fg_rgb::<30, 201, 212>(),
                style().fg_rgb::<145, 246, 111>(),
            ],
        }
    }

    fn none() -> Self {
        Self {
            error: style(),
            warning: style(),
            advice: style(),
            help: style(),
            note: style(),
            link: style(),
            linum: style(),
            highlights: [style(); 3],
        }
    }
}

// ----------------------------------------
// Most of these characters were taken from
// https://github.com/zesterer/ariadne/blob/e3cb394cb56ecda116a0a1caecd385a49e7f6662/src/draw.rs

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[expect(clippy::redundant_pub_crate, reason = "prevents public glob re-export")]
pub(crate) struct ThemeCharacters {
    pub(crate) hbar: char,
    pub(crate) vbar: char,
    pub(crate) vbar_break: char,

    pub(crate) uarrow: char,
    pub(crate) rarrow: char,

    pub(crate) ltop: char,
    pub(crate) lbot: char,

    pub(crate) lcross: char,
    pub(crate) rcross: char,

    pub(crate) underbar: char,
    pub(crate) underline: char,

    pub(crate) error: &'static str,
    pub(crate) warning: &'static str,
    pub(crate) advice: &'static str,
}

impl ThemeCharacters {
    const fn unicode() -> Self {
        Self {
            hbar: '─',
            vbar: '│',
            vbar_break: '·',
            uarrow: '▲',
            rarrow: '▶',
            ltop: '╭',
            lbot: '╰',
            lcross: '├',
            rcross: '┤',
            underbar: '┬',
            underline: '─',
            error: "×",
            warning: "⚠",
            advice: "☞",
        }
    }

    const fn ascii() -> Self {
        Self {
            hbar: '-',
            vbar: '|',
            vbar_break: ':',
            uarrow: '^',
            rarrow: '>',
            ltop: ',',
            lbot: '`',
            lcross: '|',
            rcross: '|',
            underbar: '|',
            underline: '^',
            error: "x",
            warning: "!",
            advice: ">",
        }
    }
}
