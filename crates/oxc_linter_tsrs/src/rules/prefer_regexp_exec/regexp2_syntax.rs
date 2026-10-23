//! Validation-only port of the github.com/dlclark/regexp2/v2 v2.8.0 parser (syntax/parser.go), which
//! tsgolint's prefer-regexp-exec uses through `regexp2.Compile(pattern, regexp2.ECMAScript)`.
//!
//! The parser is ported function by function, keeping the scanning positions and the state that decides
//! whether a parse error is reported (options stack, group stack, current unit, capture slots and names);
//! the regex tree and character sets themselves are not built.

use rustc_hash::{FxHashMap, FxHashSet};

use super::regexp2_tables as tables;

const IGNORE_CASE: u32 = 0x0001;
const MULTILINE: u32 = 0x0002;
const EXPLICIT_CAPTURE: u32 = 0x0004;
const SINGLELINE: u32 = 0x0010;
const IGNORE_PATTERN_WHITESPACE: u32 = 0x0020;
const RIGHT_TO_LEFT: u32 = 0x0040;
const ECMASCRIPT: u32 = 0x0100;
const RE2: u32 = 0x0200;
const UNICODE: u32 = 0x0400;

const MAX_INT32: i64 = i32::MAX as i64;
const MAX_VALUE_DIV10: i64 = MAX_INT32 / 10;
const MAX_VALUE_MOD10: i64 = MAX_INT32 % 10;
const MAX_RUNE: i64 = 0x10FFFF;

/// A parse error (the Go ErrorCode is not needed by callers).
struct Error;

type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Copy, PartialEq, Eq)]
enum NodeType {
    Capture,
    Group,
    PosLook,
    NegLook,
    Atomic,
    ExprCond,
    BackRefCond,
}

#[derive(Clone, Copy)]
struct Group {
    t: NodeType,
    /// Number of children (only tracked for conditional groups, where it is checked).
    children: usize,
}

fn in_ranges(table: &[(u32, u32)], ch: char) -> bool {
    let c = ch as u32;
    table
        .binary_search_by(|&(lo, hi)| {
            if hi < c {
                std::cmp::Ordering::Less
            } else if lo > c {
                std::cmp::Ordering::Greater
            } else {
                std::cmp::Ordering::Equal
            }
        })
        .is_ok()
}

/// charclass.go IsWordChar.
fn is_word_char(ch: char) -> bool {
    in_ranges(tables::WORD_CHAR, ch)
}

/// charclass.go IsECMAIdentifierStartChar.
fn is_ecma_identifier_start_char(ch: char) -> bool {
    in_ranges(tables::ECMA_IDENTIFIER_START_CHAR, ch)
}

/// charclass.go IsECMAIdentifierChar.
fn is_ecma_identifier_char(ch: char) -> bool {
    in_ranges(tables::ECMA_IDENTIFIER_CHAR, ch)
}

/// charclass.go normalizeUnicodeCategoryAlias.
fn normalize_unicode_category_alias(cat_name: &str) -> String {
    let mut b = String::with_capacity(cat_name.len());
    for ch in cat_name.chars() {
        match ch {
            '_' | '-' | ' ' => continue,
            _ => b.push(go_to_lower(ch)),
        }
    }
    b
}

/// unicode.ToLower (single rune mapping).
fn go_to_lower(ch: char) -> char {
    let mut lower = ch.to_lowercase();
    match (lower.next(), lower.next()) {
        (Some(l), None) => l,
        // Multi-rune lowercase mappings (only U+0130) map to their first rune in Go's simple mapping.
        (Some(l), Some(_)) => l,
        _ => ch,
    }
}

fn lookup_pair(table: &[(&'static str, &'static str)], key: &str) -> Option<&'static str> {
    table.binary_search_by(|&(k, _)| k.cmp(key)).ok().map(|i| table[i].1)
}

/// charclass.go canonicalUnicodeCatName (only whether the name is known).
fn is_known_unicode_cat_name(cat_name: &str) -> bool {
    let known = |name: &str| tables::UNICODE_CATEGORIES.binary_search(&name).is_ok();
    if known(cat_name) {
        return true;
    }
    let normalized = normalize_unicode_category_alias(cat_name);
    if lookup_pair(tables::UNICODE_SUPPORTED_PROPERTY_ALIASES, &normalized).is_some() {
        return true;
    }
    if lookup_pair(tables::UNICODE_BARE_PROPERTY_VALUE_ALIASES, &normalized).is_some() {
        return true;
    }
    if let Some(eq) = cat_name.find('=') {
        let prop_name = &cat_name[..eq];
        let value_name = &cat_name[eq + 1..];
        let Some(prop) = lookup_pair(
            tables::UNICODE_SUPPORTED_PROPERTY_ALIASES,
            &normalize_unicode_category_alias(prop_name),
        ) else {
            return false;
        };
        let normalized_value = normalize_unicode_category_alias(value_name);
        let value = tables::UNICODE_SUPPORTED_PROPERTY_VALUE_ALIASES
            .iter()
            .find(|&&(p, v, _)| p == prop && v == normalized_value)
            .map(|&(_, _, value)| value);
        let Some(value) = value else {
            return false;
        };
        return known(&format!("{prop}={value}"));
    }
    false
}

// For categorizing ascii characters.
const Q: u8 = 5; // quantifier
const S: u8 = 4; // ordinary stopper
const Z: u8 = 3; // ScanBlank stopper
const X: u8 = 2; // whitespace

#[rustfmt::skip]
const CATEGORY: [u8; 128] = [
    //01  2  3  4  5  6  7  8  9  A  B  C  D  E  F  0  1  2  3  4  5  6  7  8  9  A  B  C  D  E  F
    0, 0, 0, 0, 0, 0, 0, 0, 0, X, X, X, X, X, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    // !  "  #  $  %  &  '  (  )  *  +  ,  -  .  /  0  1  2  3  4  5  6  7  8  9  :  ;  <  =  >  ?
    X, 0, 0, Z, S, 0, 0, 0, S, S, Q, Q, 0, 0, S, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, Q,
    //@A  B  C  D  E  F  G  H  I  J  K  L  M  N  O  P  Q  R  S  T  U  V  W  X  Y  Z  [  \  ]  ^  _
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, S, S, 0, S, 0,
    //'a  b  c  d  e  f  g  h  i  j  k  l  m  n  o  p  q  r  s  t  u  v  w  x  y  z  {  |  }  ~
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, Q, S, 0, 0, 0,
];

fn category(ch: char) -> u8 {
    CATEGORY[ch as usize]
}

fn is_space(ch: char) -> bool {
    ch <= ' ' && category(ch) == X
}

// Returns true for those characters that terminate a string of ordinary chars.
fn is_special(ch: char) -> bool {
    ch <= '|' && category(ch) >= S
}

// Returns true for those characters that terminate a string of ordinary chars.
fn is_stopper_x(ch: char) -> bool {
    ch <= '|' && category(ch) >= X
}

// Returns true for those characters that begin a quantifier.
fn is_quantifier(ch: char) -> bool {
    ch <= '{' && category(ch) >= Q
}

fn option_from_code(ch: char) -> u32 {
    match ch {
        'i' | 'I' => IGNORE_CASE,
        'r' | 'R' => RIGHT_TO_LEFT,
        'm' | 'M' => MULTILINE,
        'n' | 'N' => EXPLICIT_CAPTURE,
        's' | 'S' => SINGLELINE,
        'x' | 'X' => IGNORE_PATTERN_WHITESPACE,
        'e' | 'E' => ECMASCRIPT,
        'u' | 'U' => UNICODE,
        _ => 0,
    }
}

// Returns true for options allowed only at the top level
fn is_only_top_option(option: u32) -> bool {
    option == RIGHT_TO_LEFT || option == ECMASCRIPT || option == RE2
}

// Returns n <= 0xF for a hex digit.
fn hex_digit(ch: char) -> i64 {
    match ch {
        '0'..='9' => ch as i64 - '0' as i64,
        'a'..='f' => ch as i64 - 'a' as i64 + 10,
        'A'..='F' => ch as i64 - 'A' as i64 + 10,
        _ => -1,
    }
}

struct Parser {
    pattern: Vec<char>,
    current_pos: isize,
    options: u32,
    options_stack: Vec<u32>,
    caps: FxHashMap<i64, isize>,
    capcount: i64,
    captop: i64,
    capnames: Option<FxHashMap<String, i64>>,
    autocap: i64,
    ignore_next_paren: bool,
    maintain_capture_order: bool,
    group: Group,
    stack: Vec<Group>,
    unit: bool,
}

impl Parser {
    fn note_capture_slot(&mut self, i: i64, pos: isize) {
        if let std::collections::hash_map::Entry::Vacant(e) = self.caps.entry(i) {
            // the rhs of the hashtable isn't used in the parser
            e.insert(pos);
            self.capcount += 1;
            if self.captop <= i {
                if i == MAX_INT32 {
                    self.captop = i;
                } else {
                    self.captop = i + 1;
                }
            }
        }
    }

    fn note_capture_name(&mut self, name: String, pos: isize) -> Result<()> {
        if self.capnames.is_none() {
            self.capnames = Some(FxHashMap::default());
        }
        if !self.capnames.as_ref().unwrap().contains_key(&name) {
            if self.maintain_capture_order {
                let slot = self.consume_autocap();
                self.capnames.as_mut().unwrap().insert(name, slot);
                self.note_capture_slot(slot, pos);
            } else {
                self.capnames.as_mut().unwrap().insert(name, pos as i64);
            }
        } else if self.use_option_e() {
            return Err(Error);
        }
        Ok(())
    }

    /// assignNameSlots/assignOrderedNameSlots: with maintainCaptureOrder (always on in ECMAScript mode) they
    /// only rename slots, except that capnames becomes non-nil.
    fn assign_name_slots(&mut self) {
        if self.maintain_capture_order {
            if !self.use_option_e() && self.capnames.is_none() && self.capcount == self.captop {
                return;
            }
            if self.capnames.is_none() {
                self.capnames = Some(FxHashMap::default());
            }
            if !self.use_option_e() {
                // Unnamed slots get their number as name.
                let mut slots: Vec<i64> = self.caps.keys().copied().collect();
                slots.sort_unstable();
                let named: FxHashSet<i64> =
                    self.capnames.as_ref().unwrap().values().copied().collect();
                for slot in slots {
                    if !named.contains(&slot) {
                        self.capnames.as_mut().unwrap().entry(slot.to_string()).or_insert(slot);
                    }
                }
            }
        }
    }

    fn consume_autocap(&mut self) -> i64 {
        let r = self.autocap;
        self.autocap += 1;
        r
    }

    fn consume_capture_slot(&mut self, capnum: i64) {
        if self.maintain_capture_order && capnum == self.autocap {
            self.consume_autocap();
        }
    }

    // CountCaptures is a prescanner for deducing the slots used for
    // captures by doing a partial tokenization of the pattern.
    fn count_captures(&mut self) -> Result<()> {
        self.note_capture_slot(0, 0);
        self.autocap = 1;
        while self.chars_right() > 0 {
            let pos = self.textpos();
            let ch = self.move_right_get_char();
            match ch {
                '\\' => {
                    if self.chars_right() > 0 {
                        let _ = self.scan_backslash(true);
                    }
                }
                '#' => {
                    if self.use_option_x() {
                        self.move_left();
                        let _ = self.scan_blank();
                    }
                }
                '[' => {
                    let _ = self.scan_char_set(false, true);
                }
                ')' => {
                    if !self.empty_options_stack() {
                        self.pop_options();
                    }
                }
                '(' => {
                    if self.chars_right() >= 2
                        && self.right_char(1) == '#'
                        && self.right_char(0) == '?'
                    {
                        self.move_left();
                        let _ = self.scan_blank();
                    } else {
                        self.push_options();
                        if self.chars_right() > 0 && self.right_char(0) == '?' {
                            // we have (?...
                            self.move_right(1);
                            if self.chars_right() > 1
                                && (self.right_char(0) == '<' || self.right_char(0) == '\'')
                            {
                                // named group: (?<... or (?'...
                                self.move_right(1);
                                let ch = self.right_char(0);
                                if ch != '0' && self.is_group_name_start_char(ch) {
                                    if ('1'..='9').contains(&ch) && !self.use_option_e() {
                                        let dec = self.scan_decimal()?;
                                        if self.maintain_capture_order {
                                            self.note_capture_name(dec.to_string(), pos)?;
                                        } else {
                                            self.note_capture_slot(dec, pos);
                                        }
                                    } else {
                                        let capname = self.scan_capname()?;
                                        self.note_capture_name(capname, pos)?;
                                    }
                                }
                            } else if self.use_re2()
                                && self.chars_right() > 2
                                && (self.right_char(0) == 'P' && self.right_char(1) == '<')
                            {
                                // RE2-compat (?P<)
                                self.move_right(2);
                                let ch = self.right_char(0);
                                if is_word_char(ch) {
                                    let capname = self.scan_capname()?;
                                    self.note_capture_name(capname, pos)?;
                                }
                            } else {
                                // (?...
                                // get the options if it's an option construct (?cimsx-cimsx...)
                                self.scan_options();
                                if self.chars_right() > 0 {
                                    if self.right_char(0) == ')' {
                                        // (?cimsx-cimsx)
                                        self.move_right(1);
                                        self.pop_keep_options();
                                    } else if self.right_char(0) == '(' {
                                        // alternation construct: (?(foo)yes|no)
                                        // ignore the next paren so we don't capture the condition
                                        self.ignore_next_paren = true;
                                        // break from here so we don't reset ignoreNextParen
                                        continue;
                                    }
                                }
                            }
                        } else if !self.use_option_n() && !self.ignore_next_paren {
                            let slot = self.consume_autocap();
                            self.note_capture_slot(slot, pos);
                        }
                    }
                    self.ignore_next_paren = false;
                }
                _ => {}
            }
        }
        self.assign_name_slots();
        Ok(())
    }

    fn reset(&mut self, topopts: u32) {
        self.current_pos = 0;
        self.autocap = 1;
        self.ignore_next_paren = false;
        self.options_stack.clear();
        self.options = topopts;
        self.stack.clear();
    }

    #[expect(
        clippy::if_same_then_else,
        reason = "parser.go sets lazy or possessive in the quantifier branches; the port builds no tree"
    )]
    fn scan_regex(&mut self) -> Result<()> {
        let mut ch: char;
        let mut is_quant = false;

        self.start_group(NodeType::Capture);

        'outer: while self.chars_right() > 0 {
            let mut was_prev_quantifier = is_quant;
            is_quant = false;

            self.scan_blank()?;

            let mut startpos = self.textpos();

            // move past all of the normal characters.  We'll stop when we hit some kind of control character,
            // or if IgnorePatternWhiteSpace is on, we'll stop when we see some whitespace.
            if self.use_option_x() {
                while self.chars_right() > 0 {
                    ch = self.right_char(0);
                    if is_stopper_x(ch) && (ch != '{' || self.is_true_quantifier()) {
                        break;
                    }
                    self.move_right(1);
                }
            } else {
                while self.chars_right() > 0 {
                    ch = self.right_char(0);
                    if is_special(ch) && (ch != '{' || self.is_true_quantifier()) {
                        break;
                    }
                    self.move_right(1);
                }
            }

            let endpos = self.textpos();

            self.scan_blank()?;

            if self.chars_right() == 0 {
                ch = '!'; // nonspecial, means at end
            } else {
                ch = self.right_char(0);
                if is_special(ch) {
                    is_quant = is_quantifier(ch);
                    self.move_right(1);
                } else {
                    ch = ' '; // nonspecial, means at ordinary char
                }
            }

            if startpos < endpos {
                let mut cch_unquantified = endpos - startpos;
                if is_quant {
                    cch_unquantified -= 1;
                }
                was_prev_quantifier = false;
                let _ = cch_unquantified; // addToConcatenate only builds the tree
                if is_quant {
                    self.unit = true; // addUnitOne
                }
            }

            match ch {
                '!' => break 'outer,
                ' ' => continue 'outer,
                '[' => {
                    let ci = self.use_option_i();
                    self.scan_char_set(ci, false)?;
                    self.unit = true;
                }
                '(' => {
                    self.push_options();
                    match self.scan_group_open()? {
                        None => self.pop_keep_options(),
                        Some(grouper) => {
                            self.push_group();
                            self.start_group(grouper);
                        }
                    }
                    continue 'outer;
                }
                '|' => {
                    self.add_alternate();
                    continue 'outer;
                }
                ')' => {
                    if self.empty_stack() {
                        return Err(Error);
                    }
                    self.add_group()?;
                    self.pop_group()?;
                    self.pop_options();
                    if !self.unit {
                        continue 'outer;
                    }
                }
                '\\' => {
                    self.scan_backslash(false)?;
                    self.unit = true;
                }
                '^' | '$' | '.' => self.unit = true,
                '{' | '*' | '+' | '?' => {
                    if !self.unit {
                        // ErrInvalidRepeatOp if wasPrevQuantifier, ErrMissingRepeatArgument otherwise
                        let _ = was_prev_quantifier;
                        return Err(Error);
                    }
                    self.move_left();
                }
                _ => return Err(Error),
            }

            self.scan_blank()?;

            if self.chars_right() > 0 {
                is_quant = self.is_true_quantifier();
            }
            if self.chars_right() == 0 || !is_quant {
                self.add_concatenate();
                continue 'outer;
            }

            ch = self.move_right_get_char();

            // Handle quantifiers
            while self.unit {
                let min: i64;
                let mut max: i64;
                match ch {
                    '*' => {
                        min = 0;
                        max = MAX_INT32;
                    }
                    '?' => {
                        min = 0;
                        max = 1;
                    }
                    '+' => {
                        min = 1;
                        max = MAX_INT32;
                    }
                    '{' => {
                        startpos = self.textpos();
                        min = self.scan_decimal()?;
                        max = min;
                        if startpos < self.textpos()
                            && self.chars_right() > 0
                            && self.right_char(0) == ','
                        {
                            self.move_right(1);
                            if self.chars_right() == 0 || self.right_char(0) == '}' {
                                max = MAX_INT32;
                            } else {
                                max = self.scan_decimal()?;
                            }
                        }
                        if startpos == self.textpos()
                            || self.chars_right() == 0
                            || self.move_right_get_char() != '}'
                        {
                            self.add_concatenate();
                            self.textto(startpos - 1);
                            continue 'outer;
                        }
                    }
                    _ => return Err(Error),
                }

                self.scan_blank()?;

                if self.chars_right() > 0 && self.right_char(0) == '?' {
                    self.move_right(1);
                } else if self.chars_right() > 0
                    && self.right_char(0) == '+'
                    && !self.use_option_e()
                {
                    self.move_right(1);
                }

                if min > max {
                    return Err(Error);
                }

                self.unit = false; // addConcatenate3
            }
        }

        if !self.empty_stack() {
            return Err(Error);
        }
        self.add_group()?;
        Ok(())
    }

    fn is_group_name_start_char(&self, ch: char) -> bool {
        if self.use_option_e() {
            return is_ecma_identifier_start_char(ch) || ch == '\\';
        }
        is_word_char(ch)
    }

    // scanGroupOpen scans chars following a '(' (not counting the '('), and returns
    // the type of group scanned, or None if the group simply changed options (?cimsx-cimsx).
    fn scan_group_open(&mut self) -> Result<Option<NodeType>> {
        let mut ch: char;
        let mut close = '>';

        // just return a RegexNode if we have:
        // 1. "(" followed by nothing
        // 2. "(x" where x != ?
        // 3. "(?)"
        if self.chars_right() == 0
            || self.right_char(0) != '?'
            || (self.right_char(0) == '?' && (self.chars_right() > 1 && self.right_char(1) == ')'))
        {
            if self.use_option_n() || self.ignore_next_paren {
                self.ignore_next_paren = false;
                return Ok(Some(NodeType::Group));
            }
            self.consume_autocap();
            return Ok(Some(NodeType::Capture));
        }

        self.move_right(1);

        if self.chars_right() > 0 {
            let nt: NodeType;
            ch = self.move_right_get_char();
            match ch {
                ':' => nt = NodeType::Group,
                '=' => {
                    self.options &= !RIGHT_TO_LEFT;
                    nt = NodeType::PosLook;
                }
                '!' => {
                    self.options &= !RIGHT_TO_LEFT;
                    nt = NodeType::NegLook;
                }
                '>' => nt = NodeType::Atomic,
                '\'' | '<' => {
                    if ch == '\'' {
                        close = '\'';
                    }
                    if self.chars_right() == 0 {
                        return Err(Error);
                    }
                    ch = self.move_right_get_char();
                    match ch {
                        '=' => {
                            if close == '\'' {
                                return Err(Error);
                            }
                            self.options |= RIGHT_TO_LEFT;
                            nt = NodeType::PosLook;
                        }
                        '!' => {
                            if close == '\'' {
                                return Err(Error);
                            }
                            self.options |= RIGHT_TO_LEFT;
                            nt = NodeType::NegLook;
                        }
                        _ => {
                            self.move_left();
                            let mut capnum: i64 = -1;
                            let mut uncapnum: i64 = -1;
                            let mut proceed = false;

                            // grab part before -
                            if ch.is_ascii_digit() && !self.use_option_e() {
                                capnum = self.scan_decimal()?;
                                if !self.is_capture_slot(capnum) {
                                    capnum = -1;
                                }
                                // check if we have bogus characters after the number
                                if self.chars_right() > 0
                                    && self.right_char(0) != close
                                    && self.right_char(0) != '-'
                                {
                                    return Err(Error);
                                }
                                if capnum == 0 {
                                    return Err(Error);
                                }
                            } else if self.is_group_name_start_char(ch) {
                                let capname = self.scan_capname()?;
                                if self.is_capture_name(&capname) {
                                    capnum = self.capture_slot_from_name(&capname);
                                }
                                // check if we have bogus character after the name
                                if self.chars_right() > 0
                                    && self.right_char(0) != close
                                    && self.right_char(0) != '-'
                                {
                                    return Err(Error);
                                }
                            } else if ch == '-' {
                                proceed = true;
                            } else {
                                // bad group name - starts with something other than a word character and isn't a number
                                return Err(Error);
                            }

                            // grab part after - if any
                            if !self.use_option_e()
                                && (capnum != -1 || proceed)
                                && self.chars_right() > 0
                                && self.right_char(0) == '-'
                            {
                                self.move_right(1);
                                // no more chars left, no closing char, etc
                                if self.chars_right() == 0 {
                                    return Err(Error);
                                }
                                ch = self.right_char(0);
                                if ch.is_ascii_digit() {
                                    uncapnum = self.scan_decimal()?;
                                    if !self.is_capture_slot(uncapnum) {
                                        return Err(Error);
                                    }
                                    // check if we have bogus characters after the number
                                    if self.chars_right() > 0 && self.right_char(0) != close {
                                        return Err(Error);
                                    }
                                } else if is_word_char(ch) {
                                    let uncapname = self.scan_capname()?;
                                    if !self.is_capture_name(&uncapname) {
                                        return Err(Error);
                                    }
                                    uncapnum = self.capture_slot_from_name(&uncapname);
                                    // check if we have bogus character after the name
                                    if self.chars_right() > 0 && self.right_char(0) != close {
                                        return Err(Error);
                                    }
                                } else {
                                    // bad group name - starts with something other than a word character and isn't a number
                                    return Err(Error);
                                }
                            }

                            // actually make the node
                            if (capnum != -1 || uncapnum != -1)
                                && self.chars_right() > 0
                                && self.move_right_get_char() == close
                            {
                                self.consume_capture_slot(capnum);
                                return Ok(Some(NodeType::Capture));
                            }
                            return Err(Error);
                        }
                    }
                }
                '(' => {
                    // alternation construct (?(...) | )
                    let paren_pos = self.textpos();
                    if self.chars_right() > 0 {
                        ch = self.right_char(0);
                        // check if the alternation condition is a backref
                        if ch.is_ascii_digit() {
                            let capnum = self.scan_decimal()?;
                            if self.chars_right() > 0 && self.move_right_get_char() == ')' {
                                if self.is_capture_slot(capnum) {
                                    return Ok(Some(NodeType::BackRefCond));
                                }
                                return Err(Error);
                            }
                            return Err(Error);
                        } else if is_word_char(ch) {
                            let capname = self.scan_capname()?;
                            if self.is_capture_name(&capname)
                                && self.chars_right() > 0
                                && self.move_right_get_char() == ')'
                            {
                                return Ok(Some(NodeType::BackRefCond));
                            }
                        }
                    }
                    // not a backref
                    nt = NodeType::ExprCond;
                    self.textto(paren_pos - 1); // jump to the start of the parentheses
                    self.ignore_next_paren = true; // but make sure we don't try to capture the insides

                    let chars_right = self.chars_right();
                    if chars_right >= 3 && self.right_char(1) == '?' {
                        let rightchar2 = self.right_char(2);
                        // disallow comments in the condition
                        if rightchar2 == '#' {
                            return Err(Error);
                        }
                        // disallow named capture group (?<..>..) in the condition
                        if rightchar2 == '\'' {
                            return Err(Error);
                        }
                        if chars_right >= 4
                            && (rightchar2 == '<'
                                && self.right_char(3) != '!'
                                && self.right_char(3) != '=')
                        {
                            return Err(Error);
                        }
                    }
                }
                _ => {
                    // 'P' without RE2 behaves like the default case.
                    self.move_left();
                    nt = NodeType::Group;
                    // disallow options in the children of a testgroup node
                    if self.group.t != NodeType::ExprCond {
                        self.scan_options();
                    }
                    if self.chars_right() == 0 {
                        return Err(Error);
                    }
                    ch = self.move_right_get_char();
                    if ch == ')' {
                        return Ok(None);
                    }
                    if ch != ':' {
                        return Err(Error);
                    }
                }
            }
            return Ok(Some(nt));
        }

        // BreakRecognize: ErrUnrecognizedGrouping
        Err(Error)
    }

    // scans backslash specials and basics
    fn scan_backslash(&mut self, scan_only: bool) -> Result<()> {
        if self.chars_right() == 0 {
            return Err(Error);
        }
        let ch = self.right_char(0);
        match ch {
            'Q' => {
                if self.use_option_e() {
                    return self.scan_basic_backslash(scan_only);
                }
                self.move_right(1);
                self.scan_quoted();
                Ok(())
            }
            'R' | 'X' => {
                if self.use_option_e() || self.use_re2() {
                    return self.scan_basic_backslash(scan_only);
                }
                self.move_right(1);
                Ok(())
            }
            'b' | 'B' | 'A' | 'G' | 'Z' | 'z' | 'w' | 'W' | 's' | 'S' | 'd' | 'D' => {
                self.move_right(1);
                Ok(())
            }
            'p' | 'P' => {
                // skip if we're in ECMAScript mode WITHOUT unicode flag
                if self.use_option_e() && !self.use_option_u() {
                    return self.scan_basic_backslash(scan_only);
                }
                self.move_right(1);
                self.parse_property()?;
                Ok(())
            }
            _ => self.scan_basic_backslash(scan_only),
        }
    }

    // scanQuoted scans the literal text after \Q through the next \E, or through
    // the end of the pattern when there is no terminator.
    fn scan_quoted(&mut self) -> usize {
        let start = self.textpos();
        while self.chars_right() > 0 {
            if self.right_char(0) == '\\' && self.chars_right() > 1 && self.right_char(1) == 'E' {
                let n = (self.textpos() - start) as usize;
                self.move_right(2);
                return n;
            }
            self.move_right(1);
        }
        (self.textpos() - start) as usize
    }

    // Scans \-style backreferences and character escapes
    fn scan_basic_backslash(&mut self, scan_only: bool) -> Result<()> {
        if self.chars_right() == 0 {
            return Err(Error);
        }
        let mut angled = false;
        let mut k = false;
        let mut close = '\0';

        let backpos = self.textpos();
        let mut ch = self.right_char(0);

        // Allow \k<foo> instead of \<foo>, which is now deprecated.
        let has_capnames = self.capnames.as_ref().is_some_and(|m| !m.is_empty());
        if ch == 'k' && (!self.use_option_e() || self.use_option_u() || has_capnames) {
            if self.chars_right() >= 2 {
                self.move_right(1);
                ch = self.move_right_get_char();
                if ch == '<' || (!self.use_option_e() && ch == '\'') {
                    // No support for \k'name' in ECMAScript
                    angled = true;
                    close = if ch == '\'' { '\'' } else { '>' };
                }
            }
            if !angled || self.chars_right() <= 0 {
                return Err(Error);
            }
            ch = self.right_char(0);
            k = true;
        } else if !self.use_option_e() && (ch == '<' || ch == '\'') && self.chars_right() > 1 {
            // Note angle without \g
            angled = true;
            close = if ch == '\'' { '\'' } else { '>' };
            self.move_right(1);
            ch = self.right_char(0);
        }

        // Try to parse backreference: \<1> or \<cap>
        if angled && ch.is_ascii_digit() {
            let capnum = self.scan_decimal()?;
            if self.chars_right() > 0 && self.move_right_get_char() == close {
                if self.is_capture_slot(capnum) {
                    return Ok(());
                }
                return Err(Error);
            }
        } else if !angled && ('1'..='9').contains(&ch) {
            // Try to parse backreference or octal: \1
            let capnum = self.scan_decimal()?;
            if scan_only {
                return Ok(());
            }
            if self.is_capture_slot(capnum) {
                return Ok(());
            }
            if capnum <= 9 && !self.use_option_e() {
                return Err(Error);
            }
        } else if angled {
            let capname = self.scan_capname()?;
            if !capname.is_empty() && self.chars_right() > 0 && self.move_right_get_char() == close
            {
                if scan_only {
                    return Ok(());
                }
                if self.is_capture_name(&capname) {
                    return Ok(());
                }
                return Err(Error);
            } else if k {
                return Err(Error);
            }
        }

        // Not backreference: must be char code
        self.textto(backpos);
        self.scan_char_escape()?;
        Ok(())
    }

    // Scans X for \p{X} or \P{X}
    fn parse_property(&mut self) -> Result<()> {
        // RE2 and PCRE supports \pX syntax (no {} and only 1 letter unicode cats supported)
        // since this is purely additive syntax it's not behind a flag
        if self.chars_right() >= 1
            && self.right_char(0) != '{'
            && (!self.use_option_e() || !self.use_option_u())
        {
            let ch = self.move_right_get_char().to_string();
            // check if it's a valid cat
            if !is_known_unicode_cat_name(&ch) {
                return Err(Error);
            }
            return Ok(());
        }

        if self.chars_right() < 3 {
            return Err(Error);
        }
        let mut ch = self.move_right_get_char();
        if ch != '{' {
            return Err(Error);
        }

        let startpos = self.textpos();
        while self.chars_right() > 0 {
            ch = self.move_right_get_char();
            if !is_word_char(ch) && ch != '-' && ch != '=' {
                self.move_left();
                break;
            }
        }
        let capname: String =
            self.pattern[startpos as usize..self.textpos() as usize].iter().collect();

        if self.chars_right() == 0 || self.move_right_get_char() != '}' {
            return Err(Error);
        }

        if !is_known_unicode_cat_name(&capname) {
            return Err(Error);
        }
        Ok(())
    }

    // Scans whitespace or x-mode comments.
    fn scan_blank(&mut self) -> Result<()> {
        if self.use_option_x() {
            loop {
                while self.chars_right() > 0 && is_space(self.right_char(0)) {
                    self.move_right(1);
                }
                if self.chars_right() == 0 {
                    break;
                }
                if self.right_char(0) == '#' {
                    while self.chars_right() > 0 && self.right_char(0) != '\n' {
                        self.move_right(1);
                    }
                } else if self.chars_right() >= 3
                    && self.right_char(2) == '#'
                    && self.right_char(1) == '?'
                    && self.right_char(0) == '('
                {
                    while self.chars_right() > 0 && self.right_char(0) != ')' {
                        self.move_right(1);
                    }
                    if self.chars_right() == 0 {
                        return Err(Error);
                    }
                    self.move_right(1);
                } else {
                    break;
                }
            }
        } else {
            loop {
                if self.chars_right() < 3
                    || self.right_char(2) != '#'
                    || self.right_char(1) != '?'
                    || self.right_char(0) != '('
                {
                    return Ok(());
                }
                while self.chars_right() > 0 && self.right_char(0) != ')' {
                    self.move_right(1);
                }
                if self.chars_right() == 0 {
                    return Err(Error);
                }
                self.move_right(1);
            }
        }
        Ok(())
    }

    fn scan_ecma_capname(&mut self) -> Result<String> {
        let startpos = self.textpos();
        let mut sb = String::new();
        let mut has_escape = false;
        let mut index = 0;

        while self.chars_right() > 0 {
            let savedpos = self.textpos();
            let mut ch = self.move_right_get_char();
            let mut escaped = false;

            if ch == '\\' {
                if self.chars_right() == 0 || self.right_char(0) != 'u' {
                    return Err(Error);
                }
                escaped = true;
                self.move_right(1);
                let r = if self.chars_right() > 0 && self.right_char(0) == '{' {
                    if !self.use_option_u() {
                        return Err(Error);
                    }
                    self.move_right(1);
                    self.scan_hex_until_brace()?
                } else {
                    self.scan_hex(4)?
                };
                // Go converts the rune; surrogates and the like become U+FFFD when written.
                ch = char::from_u32(r as u32).unwrap_or('\u{FFFD}');
                if !has_escape {
                    sb.extend(&self.pattern[startpos as usize..savedpos as usize]);
                    has_escape = true;
                }
            }

            let valid = if index == 0 {
                is_ecma_identifier_start_char(ch)
            } else {
                is_ecma_identifier_char(ch)
            };
            if !valid {
                if escaped {
                    return Err(Error);
                }
                self.textto(savedpos);
                break;
            }
            if has_escape {
                sb.push(ch);
            }
            index += 1;
        }

        if has_escape {
            return Ok(sb);
        }
        Ok(self.pattern[startpos as usize..self.textpos() as usize].iter().collect())
    }

    fn scan_word(&mut self) -> String {
        let startpos = self.textpos();
        while self.chars_right() > 0 {
            if !is_word_char(self.move_right_get_char()) {
                self.move_left();
                break;
            }
        }
        self.pattern[startpos as usize..self.textpos() as usize].iter().collect()
    }

    fn scan_capname(&mut self) -> Result<String> {
        if self.use_option_e() {
            return self.scan_ecma_capname();
        }
        Ok(self.scan_word())
    }

    // Scans contents of [] (not including []'s).
    #[expect(
        clippy::only_used_in_recursion,
        reason = "parser.go signature: case_insensitive only shapes the char set, which the port does not build"
    )]
    fn scan_char_set(&mut self, case_insensitive: bool, scan_only: bool) -> Result<()> {
        let mut ch: char;
        let mut ch_prev = '\0';
        let mut in_range = false;
        let mut first_char = true;
        let mut closed = false;
        let mut quoted: std::collections::VecDeque<char> = Default::default();

        if self.chars_right() > 0 && self.right_char(0) == '^' {
            self.move_right(1);
        }

        'chars: while self.chars_right() > 0 || !quoted.is_empty() {
            let mut f_translated_char = false;
            loop {
                if let Some(q) = quoted.pop_front() {
                    ch = q;
                    f_translated_char = true;
                    break;
                }
                ch = self.move_right_get_char();
                if ch == '\\'
                    && !self.use_option_e()
                    && self.chars_right() > 0
                    && self.right_char(0) == 'Q'
                {
                    self.move_right(1);
                    let start = self.textpos();
                    let n = self.scan_quoted();
                    quoted.extend(self.pattern[start as usize..start as usize + n].iter());
                    if quoted.is_empty() {
                        if self.chars_right() == 0 {
                            break;
                        }
                        continue;
                    }
                    continue;
                }
                break;
            }

            // A labeled block stands in for the Go loop body so `continue` can run the post statement.
            'body: {
                if ch == ']' {
                    if f_translated_char {
                        // A quoted closing bracket is an ordinary class member.
                    } else if !first_char || self.use_option_e() {
                        closed = true;
                        break 'chars;
                    }
                } else if ch == '\\' && !f_translated_char && self.chars_right() > 0 {
                    ch = self.move_right_get_char();
                    match ch {
                        'D' | 'd' | 'S' | 's' | 'W' | 'w' => {
                            if !scan_only && in_range {
                                if !self.use_option_e() {
                                    return Err(Error);
                                }
                                in_range = false;
                            }
                            break 'body;
                        }
                        'p' | 'P' => {
                            if self.use_option_e() && !self.use_option_u() && ch == 'P' && in_range
                            {
                                return Err(Error);
                            }
                            if self.use_option_e() && !self.use_option_u() && ch == 'p' {
                                if !scan_only {
                                    if in_range {
                                        if ch_prev > ch {
                                            return Err(Error);
                                        }
                                        in_range = false;
                                    } else if self.chars_right() >= 2
                                        && self.right_char(0) == '-'
                                        && self.right_char(1) != ']'
                                    {
                                        self.move_right(1);
                                        let ch_last = self.move_right_get_char();
                                        if ch > ch_last {
                                            return Err(Error);
                                        }
                                    }
                                }
                                break 'body;
                            }
                            self.parse_property()?;
                            if !scan_only && in_range {
                                return Err(Error);
                            }
                            break 'body;
                        }
                        '-' => break 'body,
                        _ => {
                            self.move_left();
                            ch = self.scan_char_escape()?; // non-literal character
                            f_translated_char = true;
                        }
                    }
                } else if ch == '[' && !f_translated_char {
                    // This is code for Posix style properties - [:Ll:] or [:IsTibetan:].
                    // It currently doesn't do anything other than skip the whole thing!
                    if self.chars_right() > 0 && self.right_char(0) == ':' && !in_range {
                        let save_pos = self.textpos();
                        self.move_right(1);
                        if self.chars_right() > 1 && self.right_char(0) == '^' {
                            self.move_right(1);
                        }
                        self.scan_word(); // snag the name
                        if self.chars_right() < 2
                            || self.move_right_get_char() != ':'
                            || self.move_right_get_char() != ']'
                        {
                            self.textto(save_pos);
                        }
                    }
                }

                if in_range {
                    in_range = false;
                    if !scan_only {
                        if ch == '[' && !f_translated_char && !first_char {
                            // We thought we were in a range, but we're actually starting a subtraction.
                            self.scan_char_set(case_insensitive, false)?;
                            if self.chars_right() > 0 && self.right_char(0) != ']' {
                                return Err(Error);
                            }
                        } else {
                            // a regular range, like a-z
                            if ch_prev > ch {
                                return Err(Error);
                            }
                        }
                    }
                } else if self.chars_right() >= 2
                    && self.right_char(0) == '-'
                    && self.right_char(1) != ']'
                {
                    // this could be the start of a range
                    ch_prev = ch;
                    in_range = true;
                    self.move_right(1);
                } else if self.chars_right() >= 1
                    && ch == '-'
                    && !f_translated_char
                    && self.right_char(0) == '['
                    && !first_char
                {
                    // we aren't in a range, and now there is a subtraction.
                    if !scan_only {
                        self.move_right(1);
                        self.scan_char_set(case_insensitive, false)?;
                        if self.chars_right() > 0 && self.right_char(0) != ']' {
                            return Err(Error);
                        }
                    } else {
                        self.move_right(1);
                        let _ = self.scan_char_set(case_insensitive, true);
                    }
                }
            }
            first_char = false;
        }

        if !closed {
            return Err(Error);
        }
        Ok(())
    }

    // Scans any number of decimal digits
    fn scan_decimal(&mut self) -> Result<i64> {
        let mut i: i64 = 0;
        while self.chars_right() > 0 {
            let d = self.right_char(0) as i64 - '0' as i64;
            if !(0..=9).contains(&d) {
                break;
            }
            self.move_right(1);
            if i > MAX_VALUE_DIV10 || (i == MAX_VALUE_DIV10 && d > MAX_VALUE_MOD10) {
                return Err(Error);
            }
            i = i * 10 + d;
        }
        Ok(i)
    }

    // Scans imnsxu-imnsxu option string, stops at the first unrecognized char.
    fn scan_options(&mut self) {
        let mut off = false;
        while self.chars_right() > 0 {
            let ch = self.right_char(0);
            match ch {
                '-' => off = true,
                '+' => off = false,
                _ => {
                    let option = option_from_code(ch);
                    if option == 0 || is_only_top_option(option) {
                        return;
                    }
                    if off {
                        self.options &= !option;
                    } else {
                        self.options |= option;
                    }
                }
            }
            self.move_right(1);
        }
    }

    // Scans \ code for escape codes that map to single unicode chars.
    fn scan_char_escape(&mut self) -> Result<char> {
        let ch = self.move_right_get_char();
        if ('0'..='7').contains(&ch) {
            self.move_left();
            return Ok(self.scan_octal());
        }
        let pos = self.textpos();
        let r: Result<i64> = match ch {
            'x' => {
                // support for \x{HEX} syntax from Perl and PCRE
                if self.chars_right() > 0 && self.right_char(0) == '{' {
                    if self.use_option_e() {
                        return Ok(ch);
                    }
                    self.move_right(1);
                    return self.scan_hex_until_brace().map(to_char);
                }
                self.scan_hex(2)
            }
            'u' => {
                // ECMAScript supports \u{HEX} only if `u` is also set
                if self.use_option_e()
                    && self.use_option_u()
                    && self.chars_right() > 0
                    && self.right_char(0) == '{'
                {
                    self.move_right(1);
                    return self.scan_hex_until_brace().map(to_char);
                }
                self.scan_hex(4)
            }
            'a' => return Ok('\u{7}'),
            'b' => return Ok('\u{8}'),
            'e' => return Ok('\u{1B}'),
            'f' => return Ok('\u{C}'),
            'n' => return Ok('\n'),
            'r' => return Ok('\r'),
            't' => return Ok('\t'),
            'v' => return Ok('\u{B}'),
            'c' => self.scan_control(),
            _ => {
                if !self.use_option_e() && !self.use_re2() && is_word_char(ch) {
                    return Err(Error);
                }
                return Ok(ch);
            }
        };
        match r {
            Err(_) if self.use_option_e() => {
                self.textto(pos);
                Ok(ch)
            }
            r => r.map(to_char),
        }
    }

    // Grabs and converts an ascii control character
    fn scan_control(&mut self) -> Result<i64> {
        if self.chars_right() <= 0 {
            return Err(Error);
        }
        let mut ch = self.move_right_get_char() as i64;
        // \ca interpreted as \cA
        if ch >= 'a' as i64 && ch <= 'z' as i64 {
            ch -= 'a' as i64 - 'A' as i64;
        }
        ch -= '@' as i64;
        if (0..' ' as i64).contains(&ch) {
            return Ok(ch);
        }
        Err(Error)
    }

    // Scan hex digits until we hit a closing brace.
    fn scan_hex_until_brace(&mut self) -> Result<i64> {
        let mut i: i64 = 0;
        let mut has_content = false;
        while self.chars_right() > 0 {
            let ch = self.move_right_get_char();
            if ch == '}' {
                // prevent \x{}
                if !has_content {
                    return Err(Error);
                }
                return Ok(i);
            }
            has_content = true;
            let d = hex_digit(ch);
            if d < 0 {
                return Err(Error);
            }
            i = i * 0x10 + d;
            if i > MAX_RUNE {
                return Err(Error);
            }
        }
        Err(Error)
    }

    // Scans exactly c hex digits (c=2 for \xFF, c=4 for \uFFFF)
    fn scan_hex(&mut self, mut c: isize) -> Result<i64> {
        let mut i: i64 = 0;
        if self.chars_right() >= c {
            while c > 0 {
                let d = hex_digit(self.move_right_get_char());
                if d < 0 {
                    break;
                }
                i = i * 0x10 + d;
                c -= 1;
            }
        }
        if c > 0 {
            return Err(Error);
        }
        Ok(i)
    }

    // Scans up to three octal digits (stops before exceeding 0377).
    fn scan_octal(&mut self) -> char {
        let mut c = 3;
        if c > self.chars_right() {
            c = self.chars_right();
        }
        let mut i: i64 = 0;
        let mut d = self.right_char(0) as i64 - '0' as i64;
        while c > 0 && (0..=7).contains(&d) {
            if i >= 0x20 && self.use_option_e() {
                break;
            }
            i = i * 8 + d;
            c -= 1;
            self.move_right(1);
            if !self.right_most() {
                d = self.right_char(0) as i64 - '0' as i64;
            }
        }
        to_char(i & 0xFF)
    }

    fn textpos(&self) -> isize {
        self.current_pos
    }
    fn textto(&mut self, pos: isize) {
        self.current_pos = pos;
    }
    fn move_right_get_char(&mut self) -> char {
        let ch = self.pattern[self.current_pos as usize];
        self.current_pos += 1;
        ch
    }
    fn move_right(&mut self, i: isize) {
        self.current_pos += i;
    }
    fn move_left(&mut self) {
        self.current_pos -= 1;
    }
    fn char_at(&self, i: isize) -> char {
        self.pattern[i as usize]
    }
    fn right_char(&self, i: isize) -> char {
        self.pattern[(self.current_pos + i) as usize]
    }
    fn chars_right(&self) -> isize {
        self.pattern.len() as isize - self.current_pos
    }
    fn right_most(&self) -> bool {
        self.current_pos == self.pattern.len() as isize
    }

    fn capture_slot_from_name(&self, capname: &str) -> i64 {
        self.capnames.as_ref().and_then(|m| m.get(capname).copied()).unwrap_or(0)
    }
    fn is_capture_slot(&self, i: i64) -> bool {
        self.caps.contains_key(&i)
    }
    fn is_capture_name(&self, capname: &str) -> bool {
        self.capnames.as_ref().is_some_and(|m| m.contains_key(capname))
    }

    fn use_option_n(&self) -> bool {
        self.options & EXPLICIT_CAPTURE != 0
    }
    fn use_option_i(&self) -> bool {
        self.options & IGNORE_CASE != 0
    }
    fn use_option_x(&self) -> bool {
        self.options & IGNORE_PATTERN_WHITESPACE != 0
    }
    fn use_option_e(&self) -> bool {
        self.options & ECMASCRIPT != 0
    }
    fn use_re2(&self) -> bool {
        self.options & RE2 != 0
    }
    fn use_option_u(&self) -> bool {
        self.options & UNICODE != 0
    }
    fn empty_options_stack(&self) -> bool {
        self.options_stack.is_empty()
    }

    // Finish the current quantifiable (when a quantifier is not found or is not possible)
    fn add_concatenate(&mut self) {
        self.unit = false;
    }

    // Finish the current group (in response to a ')' or end)
    fn add_group(&mut self) -> Result<()> {
        if self.group.t == NodeType::ExprCond || self.group.t == NodeType::BackRefCond {
            self.group.children += 1;
            if (self.group.t == NodeType::BackRefCond && self.group.children > 2)
                || self.group.children > 3
            {
                return Err(Error);
            }
        }
        self.unit = true; // p.unit = p.group
        Ok(())
    }

    // Pops the option stack, but keeps the current options unchanged.
    fn pop_keep_options(&mut self) {
        self.options_stack.pop();
    }

    // Recalls options from the stack.
    fn pop_options(&mut self) {
        self.options = self.options_stack.pop().unwrap();
    }

    // Saves options on a stack.
    fn push_options(&mut self) {
        self.options_stack.push(self.options);
    }

    // Push the parser state (in response to an open paren)
    fn push_group(&mut self) {
        self.stack.push(self.group);
    }

    // Remember the pushed state (in response to a ')')
    fn pop_group(&mut self) -> Result<()> {
        self.group = self.stack.pop().unwrap();
        // The first () inside a Testgroup group goes directly to the group
        if self.group.t == NodeType::ExprCond && self.group.children == 0 {
            if !self.unit {
                return Err(Error);
            }
            self.group.children += 1;
            self.unit = false;
        }
        Ok(())
    }

    // True if the group stack is empty.
    fn empty_stack(&self) -> bool {
        self.stack.is_empty()
    }

    // Start a new round for the parser state (in response to an open paren or string start)
    fn start_group(&mut self, open_group: NodeType) {
        self.group = Group { t: open_group, children: 0 };
    }

    // Finish the current concatenation (in response to a |)
    fn add_alternate(&mut self) {
        // The | parts inside a Testgroup group go directly to the group
        if self.group.t == NodeType::ExprCond || self.group.t == NodeType::BackRefCond {
            self.group.children += 1;
        }
    }

    fn is_true_quantifier(&self) -> bool {
        let mut n_chars = self.chars_right();
        if n_chars == 0 {
            return false;
        }
        let startpos = self.textpos();
        let mut ch = self.char_at(startpos);
        if ch != '{' {
            return ch <= '{' && category(ch) >= Q;
        }
        let mut pos = startpos;
        loop {
            n_chars -= 1;
            if n_chars <= 0 {
                break;
            }
            pos += 1;
            ch = self.char_at(pos);
            if !ch.is_ascii_digit() {
                break;
            }
        }
        if n_chars == 0 || pos - startpos == 1 {
            return false;
        }
        if ch == '}' {
            return true;
        }
        if ch != ',' {
            return false;
        }
        loop {
            n_chars -= 1;
            if n_chars <= 0 {
                break;
            }
            pos += 1;
            ch = self.char_at(pos);
            if !ch.is_ascii_digit() {
                break;
            }
        }
        n_chars > 0 && ch == '}'
    }
}

fn to_char(r: i64) -> char {
    char::from_u32(r as u32).unwrap_or('\u{FFFD}')
}

/// Whether `regexp2.Compile(pattern, regexp2.ECMAScript)` succeeds (syntax.Parse reports no error).
pub(crate) fn compiles_ecmascript(pattern: &str) -> bool {
    let options = ECMASCRIPT;
    let mut p = Parser {
        pattern: pattern.chars().collect(),
        current_pos: 0,
        options,
        options_stack: Vec::new(),
        caps: FxHashMap::default(),
        capcount: 0,
        captop: 0,
        capnames: None,
        autocap: 0,
        ignore_next_paren: false,
        maintain_capture_order: true,
        group: Group { t: NodeType::Capture, children: 0 },
        stack: Vec::new(),
        unit: false,
    };
    if p.count_captures().is_err() {
        return false;
    }
    p.reset(options);
    p.scan_regex().is_ok()
}
