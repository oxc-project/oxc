use oxc_allocator::Allocator;
use oxc_diagnostics::{OxcDiagnostic, Result};

use crate::{ast::RegularExpressionFlags, diagnostics, options::ParserOptions, span::SpanFactory};

pub struct FlagsParser<'a> {
    source_text: &'a str,
    span_factory: SpanFactory,
}

impl<'a> FlagsParser<'a> {
    pub fn new(_allocator: &'a Allocator, source_text: &'a str, options: ParserOptions) -> Self {
        Self {
            source_text,
            // options,
            span_factory: SpanFactory::new(options.span_offset),
        }
    }

    pub fn parse(&mut self) -> Result<RegularExpressionFlags> {
        let mut flags = RegularExpressionFlags::empty();
        let mut idx = 0;
        for c in self.source_text.chars() {
            let flag = RegularExpressionFlags::try_from(c)
                .map_err(|e| e.with_label(self.span_factory.create(idx, idx)))?;
            if flags.contains(flag) {
                return Err(diagnostics::duplicated_flag(self.span_factory.create(idx, idx)));
            }
            flags |= flag;
            idx += 1;
        }

        if flags.contains(RegularExpressionFlags::U | RegularExpressionFlags::V) {
            return Err(diagnostics::invalid_unicode_flags(self.span_factory.create(0, idx)));
        }

        Ok(flags)
    }
}

impl TryFrom<char> for RegularExpressionFlags {
    type Error = OxcDiagnostic;

    fn try_from(value: char) -> Result<Self> {
        match value {
            'g' => Ok(Self::G),
            'i' => Ok(Self::I),
            'm' => Ok(Self::M),
            's' => Ok(Self::S),
            'u' => Ok(Self::U),
            'y' => Ok(Self::Y),
            'd' => Ok(Self::D),
            'v' => Ok(Self::V),
            _ => Err(diagnostics::unknown_flag()),
        }
    }
}

impl TryFrom<u8> for RegularExpressionFlags {
    type Error = u8;

    fn try_from(value: u8) -> std::result::Result<Self, Self::Error> {
        match value {
            b'g' => Ok(Self::G),
            b'i' => Ok(Self::I),
            b'm' => Ok(Self::M),
            b's' => Ok(Self::S),
            b'u' => Ok(Self::U),
            b'y' => Ok(Self::Y),
            b'd' => Ok(Self::D),
            b'v' => Ok(Self::V),
            _ => Err(value),
        }
    }
}
