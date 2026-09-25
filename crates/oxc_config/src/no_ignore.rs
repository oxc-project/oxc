use std::str::FromStr;

#[cfg(feature = "bpaf")]
use bpaf::{Parser, doc::Doc};

/// Which ignore sources `--no-ignore` disables, grouped by where they come from.
///
/// Shared between Oxlint and Oxfmt so the kind names and their parsing stay in sync;
/// what each kind concretely disables is tool-specific and documented in each tool's help text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NoIgnoreKinds {
    /// `.gitignore` files and `$GIT_DIR/info/exclude`
    pub vcs: bool,
    /// The tool's own ignore files and CLI ignore flags
    /// (e.g. Oxlint's `.eslintignore` / `--ignore-path` / `--ignore-pattern`)
    pub cli: bool,
    /// `ignorePatterns` from config files
    pub config: bool,
}

impl NoIgnoreKinds {
    pub const NONE: Self = Self { vcs: false, cli: false, config: false };
    /// Only the `cli` sources: the historical behavior of bare `oxlint --no-ignore`.
    pub const CLI: Self = Self { vcs: false, cli: true, config: false };
    pub const ALL: Self = Self { vcs: true, cli: true, config: true };

    pub const VCS_NAME: &str = "vcs";
    pub const CLI_NAME: &str = "cli";
    pub const CONFIG_NAME: &str = "config";
    pub const ALL_NAME: &str = "all";
}

impl FromStr for NoIgnoreKinds {
    type Err = String;

    fn from_str(kinds: &str) -> Result<Self, Self::Err> {
        let mut result = Self::NONE;
        for kind in kinds.split(',').filter(|kind| !kind.is_empty()) {
            match kind {
                Self::VCS_NAME => result.vcs = true,
                Self::CLI_NAME => result.cli = true,
                Self::CONFIG_NAME => result.config = true,
                Self::ALL_NAME => result = Self::ALL,
                _ => return Err(format!("'{kind}' is not a known ignore kind")),
            }
        }
        if result == Self::NONE {
            return Err(format!(
                "expected at least one ignore kind: `{}`, `{}`, `{}`, `{}`",
                Self::VCS_NAME,
                Self::CLI_NAME,
                Self::CONFIG_NAME,
                Self::ALL_NAME
            ));
        }
        Ok(result)
    }
}

/// Build the `--no-ignore[=KINDS]` parser shared between Oxlint and Oxfmt.
///
/// The flag accepts an optional attached value:
/// bare `--no-ignore` behaves as `--no-ignore=<bare_kind_name>`, `--no-ignore=<KINDS>` disables the listed kinds.
/// The value must be attached with `=`,
/// so that a following positional path (`--no-ignore src/`) is never mistaken for a kind list.
///
/// The help text is assembled here so both tools document the flag identically;
/// only the `cli` kind covers tool-specific sources, so its bullet body comes from `cli_kind_help`.
///
/// The two forms are parsed as alternatives and converted to [`NoIgnoreKinds`] only after they converge to the same type.
/// Validating the argument before combining the parsers makes `bpaf` swallow the kind-validation error
/// (`--no-ignore=bogus` then reports a generic "not expected in this context").
/// The value-taking alternative is hidden from help to avoid displaying the option twice;
/// the visible bare alternative documents both forms.
///
/// # Panics
/// Panics when `bare_kind_name` is not a valid kind list.
#[cfg(feature = "bpaf")]
pub fn no_ignore_kinds(
    bare_kind_name: &'static str,
    cli_kind_help: &str,
) -> impl Parser<NoIgnoreKinds> {
    let bare_kinds: NoIgnoreKinds =
        bare_kind_name.parse().expect("`bare_kind_name` must be a valid kind list");

    // Pushed as flat `text()` chunks, one per line plus one per bullet lead-in,
    // mirroring how `&[(&str, Style)]` help consts render;
    // `Doc::doc()` splicing would render nested and break bullet indentation.
    // Everything is plain text with backticks denoting code, matching the other flags' description style.
    let mut help = Doc::default();
    help.text("Disable ignore sources; without a value, disables `");
    help.text(bare_kind_name);
    help.text(
        "`. Takes an optional comma-separated list of kinds attached with `=`, e.g. `--no-ignore=vcs,config`:\n",
    );
    help.text("  * `");
    help.text(NoIgnoreKinds::VCS_NAME);
    help.text("` - `.gitignore` files and `.git/info/exclude`\n");
    help.text("  * `");
    help.text(NoIgnoreKinds::CLI_NAME);
    help.text("` - ");
    help.text(cli_kind_help);
    help.text("\n");
    help.text("  * `");
    help.text(NoIgnoreKinds::CONFIG_NAME);
    help.text("` - `ignorePatterns` in config files\n");
    help.text("  * `");
    help.text(NoIgnoreKinds::ALL_NAME);
    help.text("` - all of the above");

    // `adjacent` requires `=KINDS`, so a following positional path is never consumed as the value.
    // Keep this alternative hidden to render a single help entry for the option.
    let with_kinds =
        bpaf::long("no-ignore").argument::<String>("KINDS").adjacent().map(Some).hide();
    let bare = bpaf::long("no-ignore").help(help).req_flag(None);

    bpaf::construct!([with_kinds, bare])
        .parse(move |kinds| match kinds {
            None => Ok(bare_kinds),
            Some(kinds) => kinds.parse(),
        })
        .fallback(NoIgnoreKinds::NONE)
}

#[cfg(test)]
mod test {
    use super::NoIgnoreKinds;

    #[test]
    fn from_str_single_kind() {
        assert_eq!(
            "vcs".parse::<NoIgnoreKinds>().unwrap(),
            NoIgnoreKinds { vcs: true, cli: false, config: false }
        );
    }

    #[test]
    fn from_str_multiple_kinds() {
        assert_eq!(
            "vcs,config".parse::<NoIgnoreKinds>().unwrap(),
            NoIgnoreKinds { vcs: true, cli: false, config: true }
        );
    }

    #[test]
    fn from_str_all() {
        assert_eq!("all".parse::<NoIgnoreKinds>().unwrap(), NoIgnoreKinds::ALL);
    }

    #[test]
    fn from_str_skips_interior_empties() {
        assert_eq!("cli,,".parse::<NoIgnoreKinds>().unwrap(), NoIgnoreKinds::CLI);
    }

    #[test]
    fn from_str_unknown_kind() {
        let err = "bogus".parse::<NoIgnoreKinds>().unwrap_err();
        assert!(err.contains("'bogus' is not a known ignore kind"), "{err}");
    }

    #[test]
    fn from_str_empty() {
        let err = "".parse::<NoIgnoreKinds>().unwrap_err();
        assert!(err.contains("expected at least one ignore kind"), "{err}");
    }
}
