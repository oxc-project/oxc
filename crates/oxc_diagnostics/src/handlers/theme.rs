use std::{
    env, fmt,
    io::{self, IsTerminal},
};

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

    /// Write text using the theme's warning style.
    ///
    /// # Errors
    ///
    /// Returns an error when writing the text fails.
    pub fn write_warning(&self, f: &mut impl fmt::Write, value: impl fmt::Display) -> fmt::Result {
        self.styles.warning.write(f, value)
    }
}

const ANSI_RESET: &str = "\x1b[0m";
const ANSI_PREFIXES: [&str; 9] = [
    "",
    "\x1b[38;2;225;80;80;1m",
    "\x1b[38;2;244;191;117;1m",
    "\x1b[38;2;106;159;181m",
    "\x1b[38;2;92;157;255;1m",
    "\x1b[2m",
    "\x1b[38;2;246;87;248m",
    "\x1b[38;2;30;201;212m",
    "\x1b[38;2;145;246;111m",
];

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[repr(u8)]
pub(super) enum Style {
    Plain,
    Error,
    Warning,
    Info,
    Link,
    LineNumber,
    Highlight1,
    Highlight2,
    Highlight3,
}

impl Style {
    const fn prefix(self) -> &'static str {
        ANSI_PREFIXES[self as usize]
    }

    pub(super) const fn is_plain(self) -> bool {
        matches!(self, Self::Plain)
    }

    fn write(self, f: &mut impl fmt::Write, value: impl fmt::Display) -> fmt::Result {
        let prefix = self.prefix();
        f.write_str(prefix)?;
        write!(f, "{value}")?;
        if self.is_plain() { Ok(()) } else { f.write_str(ANSI_RESET) }
    }
}

pub(super) struct Styled<T> {
    target: T,
    style: Style,
}

impl<T: fmt::Display> fmt::Display for Styled<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let prefix = self.style.prefix();
        f.write_str(prefix)?;
        self.target.fmt(f)?;
        if self.style.is_plain() { Ok(()) } else { f.write_str(ANSI_RESET) }
    }
}

pub(super) trait DiagnosticColorize: Sized {
    fn style(self, style: Style) -> Styled<Self> {
        Styled { target: self, style }
    }
}

impl<T> DiagnosticColorize for T {}

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

impl ThemeStyles {
    fn rgb() -> Self {
        Self {
            error: Style::Error,
            warning: Style::Warning,
            advice: Style::Info,
            help: Style::Info,
            note: Style::Info,
            link: Style::Link,
            linum: Style::LineNumber,
            highlights: [Style::Highlight1, Style::Highlight2, Style::Highlight3],
        }
    }

    fn none() -> Self {
        Self {
            error: Style::Plain,
            warning: Style::Plain,
            advice: Style::Plain,
            help: Style::Plain,
            note: Style::Plain,
            link: Style::Plain,
            linum: Style::Plain,
            highlights: [Style::Plain; 3],
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
