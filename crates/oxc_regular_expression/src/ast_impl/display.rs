use std::{
    fmt::{self, Display},
    iter::Peekable,
};

#[allow(clippy::wildcard_imports)]
use crate::ast::*;
use crate::surrogate_pair::{combine_surrogate_pair, is_lead_surrogate, is_trail_surrogate};

impl<'a> Display for Pattern<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.body)
    }
}

impl<'a> Display for Disjunction<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_join(f, "|", &self.body)
    }
}

impl<'a> Display for Alternative<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn as_character<'a>(term: &'a Term) -> Option<&'a Character> {
            if let Term::Character(ch) = term {
                Some(ch)
            } else {
                None
            }
        }

        write_join_with(f, "", &self.body, |iter| {
            let next = iter.next()?;
            let Some(next) = as_character(next) else { return Some(next.to_string()) };

            let peek = iter.peek().and_then(|it| as_character(it));
            let (result, eat) = character_to_string(next, peek);
            if eat {
                iter.next();
            }

            Some(result)
        })
    }
}

impl<'a> Display for Term<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BoundaryAssertion(it) => write!(f, "{}", it.as_ref()),
            Self::LookAroundAssertion(it) => write!(f, "{}", it.as_ref()),
            Self::Quantifier(it) => write!(f, "{}", it.as_ref()),
            Self::Character(it) => write!(f, "{}", it.as_ref()),
            Self::Dot(it) => write!(f, "{it}"),
            Self::CharacterClassEscape(it) => write!(f, "{}", it.as_ref()),
            Self::UnicodePropertyEscape(it) => write!(f, "{}", it.as_ref()),
            Self::CharacterClass(it) => write!(f, "{}", it.as_ref()),
            Self::CapturingGroup(it) => write!(f, "{}", it.as_ref()),
            Self::IgnoreGroup(it) => write!(f, "{}", it.as_ref()),
            Self::IndexedReference(it) => write!(f, "{}", it.as_ref()),
            Self::NamedReference(it) => write!(f, "{}", it.as_ref()),
        }
    }
}

impl Display for BoundaryAssertion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.kind)
    }
}

impl Display for BoundaryAssertionKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Start => write!(f, "^"),
            Self::End => write!(f, "$"),
            Self::Boundary => write!(f, r"\b"),
            Self::NegativeBoundary => write!(f, r"\B"),
        }
    }
}

impl<'a> Display for LookAroundAssertion<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}{})", self.kind, self.body)
    }
}

impl Display for LookAroundAssertionKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Lookahead => write!(f, "?="),
            Self::NegativeLookahead => write!(f, "?!"),
            Self::Lookbehind => write!(f, "?<="),
            Self::NegativeLookbehind => write!(f, "?<!"),
        }
    }
}

impl<'a> Display for Quantifier<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.body)?;

        match (self.min, self.max) {
            (0, None) => write!(f, "*")?,
            (1, None) => write!(f, "+")?,
            (0, Some(1)) => write!(f, "?")?,
            (min, Some(max)) if min == max => write!(f, "{{{min}}}",)?,
            (min, max) => {
                let max = max.map_or_else(String::default, |it| it.to_string());
                write!(f, "{{{min},{max}}}",)?;
            }
        }

        if !self.greedy {
            write!(f, "?")?;
        }

        Ok(())
    }
}

impl Display for Character {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (string, _) = character_to_string(self, None);
        write!(f, "{string}")
    }
}

impl Display for Dot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, ".")
    }
}

impl Display for CharacterClassEscape {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.kind)
    }
}

impl Display for CharacterClassEscapeKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::D => write!(f, r"\d"),
            Self::NegativeD => write!(f, r"\D"),
            Self::S => write!(f, r"\s"),
            Self::NegativeS => write!(f, r"\S"),
            Self::W => write!(f, r"\w"),
            Self::NegativeW => write!(f, r"\W"),
        }
    }
}

impl<'a> Display for UnicodePropertyEscape<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.negative {
            write!(f, r"\P")?;
        } else {
            write!(f, r"\p")?;
        }

        write!(f, r"{{")?;
        match (&self.name, &self.value) {
            (name, Some(value)) if name == "General_Category" => write!(f, r"{value}")?,
            (name, Some(value)) => write!(f, r"{name}={value}")?,
            _ => write!(f, r"{}", self.name)?,
        }
        write!(f, r"}}")
    }
}

impl<'a> Display for CharacterClass<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn as_character<'a>(content: &'a CharacterClassContents) -> Option<&'a Character> {
            if let CharacterClassContents::Character(ch) = content {
                Some(ch)
            } else {
                None
            }
        }

        write!(f, "[")?;

        if !self.body.is_empty() {
            if self.negative {
                write!(f, "^")?;
            }

            let sep = match self.kind {
                CharacterClassContentsKind::Union => "",
                CharacterClassContentsKind::Subtraction => "--",
                CharacterClassContentsKind::Intersection => "&&",
            };

            write_join_with(f, sep, &self.body, |iter| {
                let next = iter.next()?;
                let Some(next) = as_character(next) else { return Some(next.to_string()) };

                let peek = iter.peek().and_then(|it| as_character(it));
                let (result, eat) = character_to_string(next, peek);
                if eat {
                    iter.next();
                }

                Some(result)
            })?;
        }

        write!(f, "]")
    }
}

impl<'a> Display for CharacterClassContents<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CharacterClassRange(it) => write!(f, "{}", it.as_ref()),
            Self::CharacterClassEscape(it) => write!(f, "{}", it.as_ref()),
            Self::UnicodePropertyEscape(it) => write!(f, "{}", it.as_ref()),
            Self::Character(it) => write!(f, "{}", it.as_ref()),
            Self::NestedCharacterClass(it) => write!(f, "{}", it.as_ref()),
            Self::ClassStringDisjunction(it) => write!(f, "{}", it.as_ref()),
        }
    }
}

impl Display for CharacterClassRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}-{}", self.min, self.max)
    }
}

impl<'a> Display for ClassStringDisjunction<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, r"\q{{")?;
        write_join(f, "|", &self.body)?;
        write!(f, "}}")
    }
}

impl<'a> Display for ClassString<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_join(f, "", &self.body)
    }
}

impl<'a> Display for CapturingGroup<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(")?;

        if let Some(name) = &self.name {
            write!(f, "?<{name}>")?;
        }
        write!(f, "{}", &self.body)?;

        write!(f, ")")
    }
}

impl<'a> Display for IgnoreGroup<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn write_flags(
            f: &mut fmt::Formatter<'_>,
            prefix: char,
            flags: &ModifierFlags,
        ) -> fmt::Result {
            if flags.ignore_case {
                write!(f, "{prefix}i")?;
            }
            if flags.sticky {
                write!(f, "{prefix}y")?;
            }
            if flags.multiline {
                write!(f, "{prefix}m")?;
            }
            Ok(())
        }

        write!(f, "(?")?;

        if let Some(enabling) = &self.enabling_modifiers {
            write_flags(f, '\0', enabling)?;
        }
        if let Some(disabling) = &self.disabling_modifiers {
            write_flags(f, '-', disabling)?;
        }

        write!(f, ":{})", self.body)
    }
}

impl Display for IndexedReference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, r"\{}", self.index)
    }
}

impl<'a> Display for NamedReference<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, r"\k<{}>", self.name)
    }
}

// ---

fn character_to_string(
    this: &Character,
    peek: Option<&Character>,
) -> (/* result */ String, /* true of peek should be consumed */ bool) {
    let cp = this.value;

    if matches!(this.kind, CharacterKind::Symbol | CharacterKind::UnicodeEscape) {
        // Trail only
        if is_trail_surrogate(cp) {
            return (format!(r"\u{cp:X}"), false);
        }

        if is_lead_surrogate(cp) {
            if let Some(peek) = peek.filter(|peek| is_trail_surrogate(peek.value)) {
                // Lead+Trail
                let cp = combine_surrogate_pair(cp, peek.value);
                let ch = char::from_u32(cp).expect("Invalid surrogate pair `Character`!");
                return (format!("{ch}"), true);
            }

            // Lead only
            return (format!(r"\u{cp:X}"), false);
        }
    }

    let ch = char::from_u32(cp).expect("Invalid `Character`!");
    let result = match this.kind {
        // Not a surrogate, like BMP, or all units in unicode mode
        CharacterKind::Symbol => format!("{ch}"),
        CharacterKind::ControlLetter => match ch {
            '\n' => r"\cJ".to_string(),
            '\r' => r"\cM".to_string(),
            '\t' => r"\cI".to_string(),
            _ => format!(r"\c{ch}"),
        },
        CharacterKind::Identifier => {
            format!(r"\{ch}")
        }
        CharacterKind::SingleEscape => match ch {
            '\n' => String::from(r"\n"),
            '\r' => String::from(r"\r"),
            '\t' => String::from(r"\t"),
            '\u{b}' => String::from(r"\v"),
            '\u{c}' => String::from(r"\f"),
            '\u{8}' => String::from(r"\b"),
            '\u{2D}' => String::from(r"\-"),
            _ => format!(r"\{ch}"),
        },
        CharacterKind::Null => String::from(r"\0"),
        CharacterKind::UnicodeEscape => {
            let hex = &format!("{cp:04X}");
            if hex.len() <= 4 {
                format!(r"\u{hex}")
            } else {
                format!(r"\u{{{hex}}}")
            }
        }
        CharacterKind::HexadecimalEscape => {
            let hex = &format!("{cp:02X}");
            format!(r"\x{hex}")
        }
        CharacterKind::Octal1 => {
            let octal = format!("{cp:o}");
            format!(r"\{octal}")
        }
        CharacterKind::Octal2 => {
            let octal = format!("{cp:02o}");
            format!(r"\{octal}")
        }
        CharacterKind::Octal3 => {
            let octal = format!("{cp:03o}");
            format!(r"\{octal}")
        }
    };

    (result, false)
}

// ---

fn write_join<S, I, E>(f: &mut fmt::Formatter<'_>, sep: S, items: I) -> fmt::Result
where
    S: AsRef<str>,
    E: Display,
    I: IntoIterator<Item = E>,
{
    write_join_with(f, sep, items, |iter| iter.next().map(|it| it.to_string()))
}

fn write_join_with<S, I, E, F>(f: &mut fmt::Formatter<'_>, sep: S, items: I, next: F) -> fmt::Result
where
    S: AsRef<str>,
    E: Display,
    I: IntoIterator<Item = E>,
    F: Fn(&mut Peekable<I::IntoIter>) -> Option<String>,
{
    let sep = sep.as_ref();
    let iter = &mut items.into_iter().peekable();

    if let Some(first) = next(iter) {
        write!(f, "{first}")?;
    }

    while let Some(it) = next(iter) {
        write!(f, "{sep}{it}")?;
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use crate::{Parser, ParserOptions};
    use oxc_allocator::Allocator;

    type Case<'a> = (
        &'a str,
        /* expected display, None means expect the same as original */ Option<&'a str>,
    );

    static CASES: &[Case] = &[
        ("/ab/", None),
        ("/ab/u", None),
        ("/abc/i", None),
        ("/abc/iu", None),
        ("/a*?/i", None),
        ("/a*?/iu", None),
        ("/emo👈🏻ji/", None),
        ("/emo👈🏻ji/u", None),
        ("/ab|c/i", None),
        ("/ab|c/iu", None),
        ("/a|b+|c/i", None),
        ("/a|b+|c/iu", None),
        ("/(?=a)|(?<=b)|(?!c)|(?<!d)/i", None),
        ("/(?=a)|(?<=b)|(?!c)|(?<!d)/iu", None),
        (r"/(cg)(?<n>cg)(?:g)/", None),
        (r"/(cg)(?<n>cg)(?:g)/u", None),
        (r"/^(?=ab)\b(?!cd)(?<=ef)\B(?<!gh)$/", None),
        (r"/^(?=ab)\b(?!cd)(?<=ef)\B(?<!gh)$/u", None),
        (r"/^(?<!ab)$/", None),
        (r"/^(?<!ab)$/u", None),
        (r"/[abc]/", None),
        (r"/[abc]/u", None),
        (r"/[a&&b]/v", None),
        (r"/[a--b]/v", None),
        (r"/[^a--b--c]/v", None),
        (r"/[a[b[c[d[e[f[g[h[i[j[k[l]]]]]]]]]]]]/v", None),
        (r"/[\q{abc|d|e|}]/v", None),
        (r"/\p{Basic_Emoji}/v", None),
        (r"/[|\]]/", None),
        (r"/[|\]]/u", None),
        (r"/c\]/", None),
        (r"/c\]/u", None),
        ("/a{0}|b{1,2}|c{3,}/i", None),
        ("/a{0}|b{1,2}|c{3,}/iu", None),
        (r"/Em🥹j/", None),
        (r"/Em🥹j/u", None),
        (r"/\n\cM\0\x41\./", None),
        (r"/\n\cM\0\x41\./u", None),
        (r"/\n\cM\0\x41\u1234\./", None),
        (r"/\n\cM\0\x41\u1234\./u", None),
        (r"/[\bb]/", None),
        (r"/[\bb]/u", None),
        (r"/\d+/g", None),
        (r"/\d+/gu", None),
        (r"/\D/g", None),
        (r"/\D/gu", None),
        (r"/\w/g", None),
        (r"/\w/gu", None),
        (r"/\w+/g", None),
        (r"/\w+/gu", None),
        (r"/\s/g", None),
        (r"/\s/gu", None),
        (r"/\s+/g", None),
        (r"/\s+/gu", None),
        (r"/\t\n\v\f\r/", None),
        (r"/\t\n\v\f\r/u", None),
        (r"/\p{L}/u", None),
        (r"/\d/g", None),
        ("/abcd/igv", Some("/abcd/igv")),
        (r"/\d/ug", Some(r"/\d/ug")),
        // we capitalize hex unicodes.
        (r"/\n\cM\0\x41\u{1f600}\./u", Some(r"/\n\cM\0\x41\u{1F600}\./u")),
        (r"/\u02c1/u", Some(r"/\u02C1/u")),
        (r"/c]/", None),
        // Octal tests from: <https://github.com/tc39/test262/blob/d62fa93c8f9ce5e687c0bbaa5d2b59670ab2ff60/test/annexB/language/literals/regexp/legacy-octal-escape.js>
        (r"/\1/", None),
        (r"/\2/", None),
        (r"/\3/", None),
        (r"/\4/", None),
        (r"/\5/", None),
        (r"/\6/", None),
        (r"/\7/", None),
        (r"/\00/", None),
        (r"/\07/", None),
        (r"/\30/", None),
        (r"/\37/", None),
        (r"/\40/", None),
        (r"/\47/", None),
        (r"/\70/", None),
        (r"/\77/", None),
        (r"/\000/", None),
        (r"/\007/", None),
        (r"/\070/", None),
        (r"/\300/", None),
        (r"/\307/", None),
        (r"/\370/", None),
        (r"/\377/", None),
        (r"/\0111/", None),
        (r"/\0022/", None),
        (r"/\0003/", None),
        (r"/(.)\1/", None),
        // Identity escape from: <https://github.com/tc39/test262/blob/d62fa93c8f9ce5e687c0bbaa5d2b59670ab2ff60/test/annexB/language/literals/regexp/identity-escape.js>
        (r"/\C/", None),
        (r"/O\PQ/", None),
        (r"/\8/", None),
        (r"/7\89/", None),
        (r"/\9/", None),
        (r"/8\90/", None),
        (r"/(.)(.)(.)(.)(.)(.)(.)(.)\8\8/", None),
        // Class escape from: <https://github.com/tc39/test262/blob/d62fa93c8f9ce5e687c0bbaa5d2b59670ab2ff60/test/annexB/language/literals/regexp/class-escape.js>
        (r"/\c0/", None),
        (r"/[\c0]/", None),
        (r"/\c1/", None),
        (r"/[\c10]+/", None),
        (r"/\c8/", None),
        (r"/[\c8]/", None),
        (r"/[\c80]+/", None),
        (r"/\c_/", None),
        // Capitalize hex unicodes --
        (r"/^|\udf06/gu", Some(r"/^|\uDF06/gu")),
        (r"/\udf06/", Some(r"/\uDF06/")),
        (r"/\udf06/u", Some(r"/\uDF06/u")),
        (r"/^|\udf06/g", Some(r"/^|\uDF06/g")),
        // --
        (r"/[\-]/", None),
        (r"/[\-]/u", None),
        (r"/[\-]/v", None),
        (r"/([\-a-z]{0,31})/iu", None),
    ];

    #[test]
    fn test_display() {
        let allocator = &Allocator::default();

        for (input, output) in CASES {
            let (left_slash, right_slash) = (input.find('/').unwrap(), input.rfind('/').unwrap());

            let pattern = &input[left_slash + 1..right_slash];
            let flags = &input[right_slash + 1..];

            let actual = Parser::new(
                allocator,
                pattern,
                ParserOptions::default().with_span_offset(1).with_flags(flags),
            )
            .parse()
            .unwrap();

            let expect = output.unwrap_or(input);
            assert_eq!(expect, format!("/{actual}/{flags}")); // This uses `Display` impls
        }
    }
}
