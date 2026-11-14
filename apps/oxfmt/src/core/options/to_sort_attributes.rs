use std::sync::Arc;

use lazy_regex::Regex;

use super::super::oxfmtrc::{FormatConfig, SortAttributesUserConfig, SortOrderConfig};

type CompiledOptions = (
    Option<Arc<oxc_formatter::SortAttributesOptions>>,
    Option<Arc<oxc_formatter_htmlish::SortAttributesOptions>>,
);

// Same preset order as prettier-plugin-organize-attributes:
// https://github.com/NiklasPor/prettier-plugin-organize-attributes#code-guide-by-mdo
const CODE_GUIDE: &[&str] = &[
    "^class$", "^id$", "^name$", "^data-", "^src$", "^for$", "^type$", "^href$", "^value$",
    "^title$", "^alt$", "^role$", "^aria-",
];

/// Compile user patterns once at the config-resolution gate, shared by all files and embeds.
pub(super) fn to_sort_attributes(config: &FormatConfig) -> Result<CompiledOptions, String> {
    let (patterns, descending) = match &config.sort_attributes {
        None | Some(SortAttributesUserConfig::Bool(false)) => return Ok((None, None)),
        Some(SortAttributesUserConfig::Bool(true)) => (&[][..], false),
        Some(SortAttributesUserConfig::Object(config)) => (
            config.groups.as_deref().unwrap_or_default(),
            matches!(config.order, Some(SortOrderConfig::Desc)),
        ),
    };
    let mut groups = Vec::new();
    let mut default_group = None;
    let mut index = 0;
    for pattern in patterns {
        if pattern == "$DEFAULT" {
            if default_group.replace(index).is_some() {
                return Err(
                    "Invalid `sortAttributes` configuration: duplicate `$DEFAULT` group".into()
                );
            }
            index += 1;
            continue;
        }
        if pattern.starts_with('$') && pattern.len() > 1 && pattern != "$CODE_GUIDE" {
            return Err(format!("Invalid `sortAttributes` group: unknown preset `{pattern}`"));
        }
        let single = [pattern.as_str()];
        let expanded = if pattern == "$CODE_GUIDE" { CODE_GUIDE } else { &single };
        for pattern in expanded {
            let regex = Regex::new(pattern)
                .map_err(|error| format!("Invalid `sortAttributes` group `{pattern}`: {error}"))?;
            groups.push((index, regex));
            index += 1;
        }
    }
    let default_group = default_group.unwrap_or(index);
    Ok((
        Some(Arc::new(oxc_formatter::SortAttributesOptions {
            groups: groups.clone(),
            default_group,
            descending,
        })),
        Some(Arc::new(oxc_formatter_htmlish::SortAttributesOptions {
            groups,
            default_group,
            descending,
        })),
    ))
}
