const MAX_GLOB_GROUPS: usize = 128;
const MAX_GLOB_MATCH_STEPS: usize = 100_000;

#[derive(Clone, Copy)]
enum Repetition {
    One,
    Optional,
    ZeroOrMore,
    OneOrMore,
    Not,
}

enum Glob<'a> {
    Basic(&'a str),
    Group { prefix: &'a str, alternatives: Vec<Self>, repetition: Repetition, suffix: Box<Self> },
}

impl<'a> Glob<'a> {
    fn parse(pattern: &'a str) -> Self {
        let bytes = pattern.as_bytes();
        let mut index = 0;
        while index < bytes.len() {
            match bytes[index] {
                b'\\' => index += 2,
                b'[' => {
                    index += 1;
                    while index < bytes.len() && bytes[index] != b']' {
                        index += if bytes[index] == b'\\' { 2 } else { 1 };
                    }
                    index += 1;
                }
                b'{' => {
                    return Self::parse_group(
                        pattern,
                        index,
                        index + 1,
                        b'}',
                        b',',
                        Repetition::One,
                    );
                }
                operator @ (b'@' | b'?' | b'*' | b'+' | b'!')
                    if bytes.get(index + 1) == Some(&b'(') =>
                {
                    let repetition = match operator {
                        b'?' => Repetition::Optional,
                        b'*' => Repetition::ZeroOrMore,
                        b'+' => Repetition::OneOrMore,
                        b'!' => Repetition::Not,
                        _ => Repetition::One,
                    };
                    return Self::parse_group(pattern, index, index + 2, b')', b'|', repetition);
                }
                _ => index += 1,
            }
        }
        Self::Basic(pattern)
    }

    fn parse_group(
        pattern: &'a str,
        prefix_end: usize,
        body_start: usize,
        closing: u8,
        separator: u8,
        repetition: Repetition,
    ) -> Self {
        let bytes = pattern.as_bytes();
        let mut stack = vec![closing];
        let mut alternatives = Vec::new();
        let mut start = body_start;
        let mut index = start;
        while index < bytes.len() {
            let byte = bytes[index];
            if byte == b'\\' {
                index += 2;
                continue;
            }
            if byte == b'[' {
                index += 1;
                while index < bytes.len() && bytes[index] != b']' {
                    index += if bytes[index] == b'\\' { 2 } else { 1 };
                }
            } else if stack.last() == Some(&byte) {
                stack.pop();
                if stack.is_empty() {
                    alternatives.push(Self::parse(&pattern[start..index]));
                    return Self::Group {
                        prefix: &pattern[..prefix_end],
                        alternatives,
                        repetition,
                        suffix: Box::new(Self::parse(&pattern[index + 1..])),
                    };
                }
            } else if byte == b'{' {
                stack.push(b'}');
            } else if byte == b'(' && matches!(bytes[index - 1], b'@' | b'?' | b'*' | b'+' | b'!') {
                stack.push(b')');
            } else if byte == separator && stack.len() == 1 {
                alternatives.push(Self::parse(&pattern[start..index]));
                start = index + 1;
            }
            index += 1;
        }
        Self::Basic(pattern)
    }

    fn matches(&self, path: &str, nested: bool, remaining_steps: &mut usize) -> bool {
        if *remaining_steps == 0 {
            return false;
        }
        *remaining_steps -= 1;
        let (prefix, alternatives, repetition, suffix) = match self {
            Self::Basic(pattern) => return fast_glob::glob_match(pattern, path),
            Self::Group { prefix, alternatives, repetition, suffix } => {
                (prefix, alternatives, repetition, suffix)
            }
        };
        let boundaries: Vec<_> =
            path.char_indices().map(|(index, _)| index).chain([path.len()]).collect();
        for (start_index, &start) in boundaries.iter().enumerate() {
            if !fast_glob::glob_match(prefix, &path[..start]) {
                continue;
            }
            match repetition {
                Repetition::One | Repetition::Optional | Repetition::Not => {
                    if matches!(repetition, Repetition::Not)
                        && boundaries[start_index..].iter().any(|&mid| {
                            alternatives
                                .iter()
                                .any(|glob| glob.matches(&path[start..mid], true, remaining_steps))
                                && if nested {
                                    // Nested negations exclude prefixes of the containing group.
                                    path[mid..]
                                        .char_indices()
                                        .map(|(end, _)| end)
                                        .chain([path.len() - mid])
                                        .any(|end| {
                                            suffix.matches(
                                                &path[mid..mid + end],
                                                nested,
                                                remaining_steps,
                                            )
                                        })
                                } else {
                                    suffix.matches(&path[mid..], nested, remaining_steps)
                                }
                        })
                    {
                        continue;
                    }
                    for &end in &boundaries[start_index..] {
                        let matched = match repetition {
                            Repetition::Not => !path[start..end].contains('/'),
                            Repetition::Optional if start == end => true,
                            _ => alternatives
                                .iter()
                                .any(|glob| glob.matches(&path[start..end], true, remaining_steps)),
                        };
                        if matched && suffix.matches(&path[end..], nested, remaining_steps) {
                            return true;
                        }
                    }
                }
                Repetition::ZeroOrMore | Repetition::OneOrMore => {
                    let mut reachable = vec![false; boundaries.len()];
                    reachable[start_index] = true;
                    if (matches!(repetition, Repetition::ZeroOrMore)
                        || alternatives.iter().any(|glob| glob.matches("", true, remaining_steps)))
                        && suffix.matches(&path[start..], nested, remaining_steps)
                    {
                        return true;
                    }
                    for index in start_index..boundaries.len() {
                        if !reachable[index] {
                            continue;
                        }
                        for next in index + 1..boundaries.len() {
                            if !reachable[next]
                                && alternatives.iter().any(|glob| {
                                    glob.matches(
                                        &path[boundaries[index]..boundaries[next]],
                                        true,
                                        remaining_steps,
                                    )
                                })
                            {
                                reachable[next] = true;
                                if suffix.matches(
                                    &path[boundaries[next]..],
                                    nested,
                                    remaining_steps,
                                ) {
                                    return true;
                                }
                            }
                        }
                    }
                }
            }
        }
        false
    }
}

fn matches_brace_expansions(pattern: &str, path: &str, remaining_steps: &mut usize) -> bool {
    if *remaining_steps == 0 {
        return false;
    }
    *remaining_steps -= 1;
    let bytes = pattern.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'\\' {
            index += 2;
            continue;
        }
        if bytes[index] != b'{' {
            index += 1;
            continue;
        }
        let opening = index;
        let mut depth = 1;
        let mut start = index + 1;
        let mut alternatives = Vec::new();
        index += 1;
        while index < bytes.len() {
            match bytes[index] {
                b'\\' => {
                    index += 2;
                    continue;
                }
                b'{' => depth += 1,
                b'}' => {
                    depth -= 1;
                    if depth == 0 {
                        if !alternatives.is_empty() {
                            alternatives.push(&pattern[start..index]);
                            return alternatives.iter().any(|alternative| {
                                let expanded = format!(
                                    "{}{alternative}{}",
                                    &pattern[..opening],
                                    &pattern[index + 1..]
                                );
                                matches_brace_expansions(&expanded, path, remaining_steps)
                            });
                        }
                        break;
                    }
                }
                b',' if depth == 1 => {
                    alternatives.push(&pattern[start..index]);
                    start = index + 1;
                }
                _ => {}
            }
            index += 1;
        }
        index += 1;
    }
    Glob::parse(pattern).matches(path, false, remaining_steps)
}

/// Match filename globs including the extended groups used by ESLint rule options.
/// Ordinary patterns retain the allocation-free `fast_glob` path.
/// Excessively complex patterns fail to match instead of exhausting the stack or match budget.
pub fn glob_match_with_extglobs(pattern: &str, path: &str) -> bool {
    if !pattern
        .as_bytes()
        .windows(2)
        .any(|pair| matches!(pair[0], b'@' | b'?' | b'*' | b'+' | b'!') && pair[1] == b'(')
    {
        return fast_glob::glob_match(pattern, path);
    }
    if pattern.bytes().filter(|byte| matches!(byte, b'(' | b'{')).count() > MAX_GLOB_GROUPS {
        return false;
    }
    let positive = pattern.trim_start_matches('!');
    let negated = !(pattern.len() - positive.len()).is_multiple_of(2);
    let mut remaining_steps = MAX_GLOB_MATCH_STEPS;
    let matched = matches_brace_expansions(positive, path, &mut remaining_steps);
    remaining_steps > 0 && matched != negated
}

#[cfg(test)]
mod tests {
    use super::glob_match_with_extglobs;

    #[test]
    fn excessive_nesting() {
        let pattern = format!("{}a{}", "@(".repeat(256), ")".repeat(256));
        assert!(!glob_match_with_extglobs(&pattern, "a"));
        assert!(!glob_match_with_extglobs(&format!("!{pattern}"), "b"));
    }

    #[test]
    fn extended_groups() {
        for (pattern, path, expected) in [
            ("**/*.+(test|spec).ts", "src/foo.test.ts", true),
            ("**/*.+(test|spec).ts", "foo.testspec.ts", true),
            ("**/*.+(test|spec).ts", "foo..ts", false),
            ("**/*.*(test|spec).ts", "foo..ts", true),
            ("**/*.?(test).ts", "foo.testtest.ts", false),
            ("**/*.@(test|spec).ts", "foo.testspec.ts", false),
            ("**/*.!(test|spec).ts", "foo.ts", false),
            ("**/*.!(test|spec).ts", "foo.test.ts", false),
            ("**/!(prod)*.ts", "src/prod.ts", false),
            ("**/!(prod)*.ts", "src/product.ts", false),
            ("**/!(prod)*.ts", "src/test.ts", true),
            ("**/!(prod).ts", "src/product.ts", true),
            ("**/!(prod)*.ts/rest", "src/prod.ts/rest", false),
            ("**/+(a|@(bc|dé)).ts", "src/abcadé.ts", true),
            ("**/+(a|@(bc|dé)).ts", "src/ab.ts", false),
            ("**/{+(test|spec),other}.ts", "specspec.ts", true),
            ("**/{+(test|spec),other}.ts", "other.ts", true),
            ("**/+(test|spec).{js,ts}", "testspec.ts", true),
            ("**/+({a,b}|c).ts", "acac.ts", true),
            ("**/+({a,b}|c).ts", "abc.ts", false),
            ("**/!({a,b}|c).ts", "a.ts", true),
            ("**/@(!(a)|b).ts", "abc.ts", false),
            ("**/@(!(a)|b).ts", "bc.ts", true),
            ("**/+().ts", ".ts", true),
            ("**/*().ts", "test.ts", false),
            (r"**/\+(test|spec).ts", "+(test|spec).ts", true),
            ("!**/*.@(test|spec).ts", "foo.test.ts", false),
            ("!**/*.@(test|spec).ts", "foo.js", true),
        ] {
            assert_eq!(glob_match_with_extglobs(pattern, path), expected, "{pattern}: {path}");
        }
    }
}
