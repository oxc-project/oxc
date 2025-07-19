#![expect(dead_code)]

use std::{
    ops::{Index, RangeInclusive},
    slice::Iter,
};

// TODO: Return value should be `Token` not `State`
pub fn next_token(iter: &mut Iter<u8>) -> State {
    let mut state = State::Eof;
    loop {
        let Some(&byte) = iter.next() else {
            return State::Eof;
        };

        let category = BYTE_TO_CATEGORY.0[byte as usize];
        state = NEXT_STATE_MAP.0[state as usize].0[category as usize];

        if state.is_final() {
            break;
        }
    }

    // TODO: Handle special `State`s which need further processing e.g. `Ident`

    state
}

// TODO: Try to get this down to 32 variants. Would need to lose 4.
// Maybe `Error`, `IrregularWhitespace`, `BackSlash`, `Unicode` are the ones to lose.
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum Category {
    Error = 0,               // Invalid character (rare)
    Whitespace = 1,          // <space>, \t
    LineBreak = 2,           // \n, \r
    IrregularWhitespace = 3, // 0xB, 0xC (rare)
    Exclamation = 4,         // !
    Quote = 5,               // ', ", `
    Hash = 6,                // #
    Percent = 7,             // %
    Ampersand = 8,           // &
    ParenOpen = 9,           // (
    ParenClose = 10,         // )
    Star = 11,               // *
    Plus = 12,               // +
    Comma = 13,              // ,
    Minus = 14,              // -
    Dot = 15,                // .
    Slash = 16,              // /
    Zero = 17,               // 0
    Digit = 18,              // 1-9
    Colon = 19,              // :
    Semicolon = 20,          // ;
    AngleOpen = 21,          // <
    Equal = 22,              // =
    AngleClose = 23,         // >
    Question = 24,           // ?
    At = 25,                 // @
    SquareOpen = 26,         // [
    BackSlash = 27,          // \ (rare - start of escaped identifier)
    SquareClose = 28,        // ]
    Caret = 29,              // ^ (rare)
    CurlyOpen = 30,          // {
    Pipe = 31,               // |
    CurlyClose = 32,         // }
    Tilde = 33,              // ~
    Unicode = 34,            // Unicode character (rare)
    Ident = 35,
}

const CATEGORY_LEN: usize = 36;

#[repr(C, align(128))]
struct ByteToCategory([Category; 256]);

impl ByteToCategory {
    const fn set<const N: usize>(&mut self, bytes: [u8; N], category: Category) {
        let mut i = 0;
        while i < N {
            self.set_one(bytes[i], category);
            i += 1;
        }
    }

    const fn set_range(&mut self, range: RangeInclusive<u8>, category: Category) {
        let mut byte = *range.start();
        loop {
            self.set_one(byte, category);
            if byte == *range.end() {
                break;
            }
            byte += 1;
        }
    }

    const fn set_one(&mut self, byte: u8, category: Category) {
        self.0[byte as usize] = category;
    }
}

impl Index<u8> for ByteToCategory {
    type Output = Category;
    fn index(&self, byte: u8) -> &Category {
        &self.0[byte as usize]
    }
}

static BYTE_TO_CATEGORY: ByteToCategory = {
    let mut table = ByteToCategory([Category::Error; 256]);

    table.set([b' ', b'\t'], Category::Whitespace);
    table.set([0xB, 0xC], Category::IrregularWhitespace);
    table.set([b'\r', b'\n'], Category::LineBreak);
    table.set([b'!'], Category::Exclamation);
    table.set([b'\'', b'"', b'`'], Category::Quote);
    table.set([b'#'], Category::Hash);
    table.set([b'_', b'$'], Category::Ident);
    table.set_range(b'A'..=b'Z', Category::Ident);
    table.set_range(b'a'..=b'z', Category::Ident);
    table.set([b'%'], Category::Percent);
    table.set([b'&'], Category::Ampersand);
    table.set([b'('], Category::ParenOpen);
    table.set([b')'], Category::ParenClose);
    table.set([b'*'], Category::Star);
    table.set([b'+'], Category::Plus);
    table.set([b','], Category::Comma);
    table.set([b'-'], Category::Minus);
    table.set([b'.'], Category::Dot);
    table.set([b'/'], Category::Slash);
    table.set([b'0'], Category::Zero);
    table.set_range(b'1'..=b'9', Category::Digit);
    table.set([b':'], Category::Colon);
    table.set([b';'], Category::Semicolon);
    table.set([b'<'], Category::AngleOpen);
    table.set([b'='], Category::Equal);
    table.set([b'>'], Category::AngleClose);
    table.set([b'?'], Category::Question);
    table.set([b'['], Category::SquareOpen);
    table.set([b'\\'], Category::BackSlash);
    table.set([b']'], Category::SquareClose);
    table.set([b'^'], Category::Caret);
    table.set([b'{'], Category::CurlyOpen);
    table.set([b'|'], Category::Pipe);
    table.set([b'}'], Category::CurlyClose);
    table.set([b'~'], Category::Tilde);
    table.set_range(0x80..=0xFF, Category::Unicode);

    table
};

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum State {
    // Exit state, but we treat is as not exit - used as default
    Eof = 0,

    // Partial states - 1 char
    ExclamationPartial = 1, // !
    PercentPartial = 2,     // %
    AmpersandPartial = 3,   // &
    StarPartial = 4,        // *
    PlusPartial = 5,        // +
    MinusPartial = 6,       // -
    DotPartial = 7,         // .
    SlashPartial = 8,       // /
    ZeroPartial = 9,        // 0
    DigitsPartial = 10,     // 1-9
    AngleOpenPartial = 11,  // <
    EqualPartial = 12,      // =
    QuestionPartial = 13,   // ?
    CaretPartial = 14,      // ^
    PipePartial = 15,       // |

    // Partial states - 2 chars
    ExclamationEqualPartial = 16, // !=
    Ampersand2Partial = 17,       // &&
    Star2Partial = 18,            // **
    Dot2Partial = 19,             // ..
    ZeroDotPartial = 20,          // 0.
    AngleOpen2Partial = 21,       // <<
    Equal2Partial = 22,           // ==
    Question2Partial = 23,        // ??
    QuestionDotPartial = 24,      // ?.
    Pipe2Partial = 25,            // ||

    // Partial states - any length
    DotDigitsPartial,  // .<digits>
    ZeroDigitsPartial, // 0<digits>
    DigitsDotPartial,  // <digits>.

    // Complete tokens - 1 char
    Exclamation, // !
    Percent,     // %
    Ampersand,   // &
    ParenOpen,   // (
    ParenClose,  // )
    Star,        // *
    Plus,        // +
    Comma,       // ,
    Minus,       // -
    Dot,         // .
    Slash,       // /
    // Zero?     // 0 TODO
    Colon,       // :
    Semicolon,   // ;
    AngleOpen,   // <
    Equal,       // =
    AngleClose,  // >
    Question,    // ?
    At,          // @
    SquareOpen,  // [
    SquareClose, // ]
    Caret,       // ^
    CurlyOpen,   // {
    Pipe,        // |
    CurlyClose,  // }
    Tilde,       // ~

    // Complete tokens - 2 chars
    ExclamationEqual, // !=
    PercentEqual,     // %=
    Ampersand2,       // &&
    AmpersandEqual,   // &=
    Star2,            // **
    StarEqual,        // *=
    Plus2,            // ++
    PlusEqual,        // +=
    Minus2,           // --
    MinusEqual,       // -=
    SlashEqual,       // /=
    AngleOpen2,       // <<
    AngleOpenEqual,   // <=
    Arrow,            // =>
    CaretEqual,       // ^=
    Question2,        // ??
    Pipe2,            // ||
    PipeEqual,        // |=

    // Complete tokens - 3 chars

    // Complete tokens - any length
    Digits,
    DotDigits,

    // Exit states - require further handling
    IrregularWhitespace,  // Irregular whitespace
    Quote,                // ', ", `
    Hash,                 // #
    BackSlash,            // \
    Unicode,              // Unicode characters
    Ident,                // Identifier
    Error,                // Error
    Slash2,               // //
    SlashStar,            // /*
    ZeroIdent,            // 0b, 0x, 0n etc, 0_, or illegal pattern e.g. 0A, 0$
    DigitsIdent,          // <digits>n, <digits>e, <digits>_, or illegal pattern e.g. 123A, 123$
    AngleOpenExclamation, // <!
}

const STATE_LEN: usize = 35;
const STATE_PARTIAL_LEN: usize = State::Exclamation as usize;

impl State {
    fn is_final(self) -> bool {
        self as u8 >= State::Exclamation as u8
    }
}

struct StateToCategoryToNextState([CategoryToNextState; STATE_PARTIAL_LEN]);

impl StateToCategoryToNextState {
    const fn set(&mut self, state: State, map: CategoryToNextState) {
        self.0[state as usize] = map;
    }
}

impl Index<State> for StateToCategoryToNextState {
    type Output = CategoryToNextState;
    fn index(&self, state: State) -> &CategoryToNextState {
        &self.0[state as usize]
    }
}

#[derive(Clone, Copy)]
struct CategoryToNextState([State; CATEGORY_LEN]);

impl CategoryToNextState {
    const fn default() -> Self {
        Self::splat(State::Eof)
    }

    const fn splat(state: State) -> Self {
        Self([state; CATEGORY_LEN])
    }

    const fn set(&mut self, category: Category, state: State) {
        self.0[category as usize] = state;
    }
}

impl Index<Category> for CategoryToNextState {
    type Output = State;
    fn index(&self, category: Category) -> &State {
        &self.0[category as usize]
    }
}

macro_rules! define_transition {
    (
        $map:ident[$initial:ident] = {
            $($category:ident => $new_state:ident,)+
        }
    ) => {
        define_transition!($map[$initial] = {
            @default => Eof,
            $($category => $new_state,)+
        })
    };

    (
        $map:ident[$initial:ident] = {
            @default => $default:ident,
            $($category:ident => $new_state:ident,)+
        }
    ) => {
        $map.set(State::$initial, {
            let mut map = CategoryToNextState::splat(State::$default);
            $(
                map.set(Category::$category, State::$new_state);
            )+
            map
        });
    };
}

static NEXT_STATE_MAP: StateToCategoryToNextState = {
    let mut map = StateToCategoryToNextState([CategoryToNextState::default(); STATE_PARTIAL_LEN]);

    // First character
    define_transition!(map[Eof] = {
        Error => Error,
        Whitespace => Eof,
        LineBreak => Eof, // TODO: Need to set line break flag
        IrregularWhitespace => IrregularWhitespace,
        Exclamation => ExclamationPartial, // May be start of `!=` or `!==`
        Quote => Quote, // Exit
        Hash => Hash, // Exit
        Percent => PercentPartial, // May be start of `%=`
        Ampersand => AmpersandPartial, // May be start of `&&`, `&=`, or `&&=`
        ParenOpen => ParenOpen, // Complete token
        ParenClose => ParenClose, // Complete token
        Star => StarPartial, // May be followed by `=`
        Plus => PlusPartial, // May be start of `++` or `+=`
        Comma => Comma, // Complete token
        Minus => MinusPartial, // May be start of `--`, `-=`, or `-->`
        Dot => DotPartial, // May be start of `...` or `.<digits>`
        Slash => SlashPartial, // May be start of `//`, `/*`, `/=`, or regexp
        Zero => ZeroPartial, // May be start of `0.<digits>`, `0<digits>`, `0n<digits>`, `0x<digits>` etc
        Digit => DigitsPartial, // May be start of `<digits>`, `<digits>.<digits>` etc
        Colon => Colon, // Complete token
        Semicolon => Semicolon, // Complete token
        AngleOpen => AngleOpenPartial, // May be start of `<<`, `<=`, `<<=`, or `<!--`
        Equal => EqualPartial, // May be start of `==`, `=>`, or `===`
        AngleClose => AngleClose, // Complete token
        Question => QuestionPartial, // May be start of `??`, `?.`, `??=`, or `?.<digit>`
        At => At, // Complete token
        SquareOpen => SquareOpen, // Complete token
        BackSlash => BackSlash, // Exit
        SquareClose => SquareClose, // Complete token
        Caret => CaretPartial, // May be start of `^=`
        CurlyOpen => CurlyOpen, // Complete token
        Pipe => PipePartial,
        CurlyClose => CurlyClose, // Complete token
        Pipe => PipePartial, // May be start of `||`, `|=`, or `||=`
        Tilde => Tilde, // Complete token
        Unicode => Unicode, // Exit
        Ident => Ident, // Exit (maybe should continue consuming)
    });

    // 2nd byte continuations
    define_transition!(map[ExclamationPartial] = {
        @default => Exclamation,          // !  - Complete token
        Equal => ExclamationEqualPartial, // != - May be start of `!==`
    });

    define_transition!(map[PercentPartial] = {
        @default => Percent,              // %  - Complete token
        Equal => PercentEqual,            // %= - Complete token
    });

    define_transition!(map[AmpersandPartial] = {
        @default => Ampersand,            // &  - Complete token
        Ampersand => Ampersand2Partial,   // && - May be start of `&&=`
        Equal => AmpersandEqual,          // &= - Complete token
    });

    define_transition!(map[StarPartial] = {
        @default => Star,                 // *  - Complete token
        Star => Star2Partial,             // ** - May be start of `**=`
        Equal => StarEqual,               // *= - Complete token
    });

    define_transition!(map[PlusPartial] = {
        @default => Plus,                 // +  - Complete token
        Plus => Plus2,                    // ++ - Complete token
        Equal => PlusEqual,               // += Complete token
    });

    define_transition!(map[MinusPartial] = {
        @default => Minus,                 // -  - Complete token
        Plus => Minus2,                    // -- - Complete token
        Equal => MinusEqual,               // -= - Complete token
    });

    define_transition!(map[DotPartial] = {
        @default => Dot,                   // .  - Complete token
        Plus => Dot2Partial,               // .. - May be start of `...`
        Zero => DotDigitsPartial,          // .0 - May be followed by more digits
        Digit => DotDigitsPartial,         // .1 - May be followed by more digits
    });

    define_transition!(map[SlashPartial] = {
        @default => Slash,                 // /  - Complete token
        Slash => Slash2,                   // // - Exit
        Zero => SlashStar,                 // /* - Exit
        Equal => SlashEqual,               // /= - Complete token
    });

    define_transition!(map[ZeroPartial] = {
        @default => Digits,                // 0  - Complete token - TODO: Should it be `Zero`?
        Ident => ZeroIdent,                // 0b, 0x, 0n etc, 0_, or illegal pattern e.g. 0A, 0$ - Exit
        Dot => DigitsDotPartial,           // 0. - May be followed by more digits
        Zero => ZeroDigitsPartial,         // 00 - May be followed by more digits
        Digit => ZeroDigitsPartial,        // 01 - May be followed by more digits
    });

    define_transition!(map[DigitsPartial] = {
        @default => Digits,                // <digits>  - Complete token - TODO: Should it be `Zero`?
        Dot => DigitsDotPartial,           // <digits>. - May be followed by more digits
        Zero => DigitsPartial,             // <digits>  - May be followed by more digits
        Digit => DigitsPartial,            // <digits>  - May be followed by more digits
        Ident => DigitsIdent,              // <digits>n, <digits>e, <digits>_, or illegal pattern e.g. 123A, 123$ - Exit
    });

    define_transition!(map[AngleOpenPartial] = {
        @default => AngleOpen,             // <  - Complete token
        AngleOpen => AngleOpen2Partial,    // << - May be start of `<<=`
        Equal => AngleOpenEqual,           // <= - Complete token
        Exclamation => AngleOpenExclamation, // <! - Exit
    });

    define_transition!(map[EqualPartial] = {
        @default => Equal,                 // =  - Complete token
        Equal => Equal2Partial,            // == - May be start of `===`
        AngleClose => Arrow,               // => - Complete token
    });

    define_transition!(map[QuestionPartial] = {
        @default => Question,              // ?  - Complete token
        Question => Question2Partial,      // ?? - May be start of `??=`
        Dot => QuestionDotPartial,         // ?. - May be followed by a digit
    });

    define_transition!(map[PipePartial] = {
        @default => Pipe,                  // |  - Complete token
        Pipe => Pipe2Partial,              // || - May be start of `||=`
        Equal => PipeEqual,                // |= - Complete token
    });

    // TODO: ExclamationEqualPartial onwards

    map
};
