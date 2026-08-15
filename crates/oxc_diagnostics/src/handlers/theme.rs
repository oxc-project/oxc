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
    pub(super) characters: ThemeCharacters,
    pub(super) styles: ThemeStyles,
}

fn force_color() -> bool {
    // Assume CI can always print colors.
    env::var_os("CI").is_some() || env::var_os("FORCE_COLOR").is_some_and(|value| value != "0")
}

impl Default for GraphicalTheme {
    fn default() -> Self {
        Self::new(io::stdout().is_terminal() && io::stderr().is_terminal())
    }
}

impl GraphicalTheme {
    /// Chooses a graphical theme based on terminal and environment support.
    #[must_use]
    pub(super) fn new(is_terminal: bool) -> Self {
        if force_color() {
            return Self::unicode();
        }
        match env::var_os("NO_COLOR") {
            _ if !is_terminal => Self::none(),
            Some(value) if value != "0" => Self::unicode_nocolor(),
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
pub(super) struct ThemeStyles {
    pub(super) error: Style,
    pub(super) warning: Style,
    pub(super) advice: Style,
    pub(super) help: Style,
    pub(super) note: Style,
    pub(super) link: Style,
    pub(super) linum: Style,
    pub(super) highlights: [Style; 3],
}

fn style() -> Style {
    Style::new()
}

impl ThemeStyles {
    fn rgb() -> Self {
        Self {
            error: style().fg_rgb::<225, 80, 80>().bold(),
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
pub(super) struct ThemeCharacters {
    pub(super) hbar: char,
    pub(super) vbar: char,
    pub(super) vbar_break: char,

    pub(super) uarrow: char,
    pub(super) rarrow: char,

    pub(super) ltop: char,
    pub(super) lbot: char,

    pub(super) lcross: char,
    pub(super) rcross: char,

    pub(super) underbar: char,
    pub(super) underline: char,

    pub(super) error: &'static str,
    pub(super) warning: &'static str,
    pub(super) advice: &'static str,
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
