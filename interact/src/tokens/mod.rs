/// Token parser for Interact input.
///
/// Behind the scenes, we would like to perform parsing of certain types of expressions, mainly for
/// the purpose of data access. The most common type of data access is dereferences of a struct
/// field, i.e., `.field`. Others, include map indexing `["value"]`, so we only need a certain
/// subset of Rust tokens.
///
/// However, to make implementation easier, we only do basic splitting of tokens here using `pest`,
/// and for the basic types, allow `ron` to do a more in depth resolve of the basic Rust types
/// into the actual values.

#[derive(Parser)]
#[grammar = "tokens/parse.pest"]
pub struct ExprParser;
use std::borrow::Cow;

use pest::Parser;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum TokenKind {
    Ident,
    NonNegativeDecimal(u64),
    Decimal(i64),
    SubscriptOpen,
    SubscriptClose,
    TupleOpen,
    TupleClose,
    CurlyOpen,
    CurlyClose,
    FieldAccess,
    Assign,
    Colon,
    Asterix,
    Char(char),
    String(String),
    Range(bool),
    Comma,
    InvalidToken,
}

/// Represents a single meaningful substring part in an Interact string expression.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Token<'a> {
    /// Token kind
    pub kind: TokenKind,

    /// Token text
    pub text: Cow<'a, str>,

    /// Amount of whitespace from the previous token
    pub space_diff: usize,
}

impl<'a> Token<'a> {
    pub fn new_owned(s: String) -> Self {
        Token {
            kind: TokenKind::Ident,
            text: Cow::Owned(s),
            space_diff: 0,
        }
    }

    pub fn new_borrowed(kind: TokenKind, s: &'static str) -> Self {
        Token {
            kind,
            text: Cow::from(s),
            space_diff: 0,
        }
    }

    pub fn clone_owned(&self) -> Token<'static> {
        Token {
            kind: self.kind.clone(),
            text: Cow::Owned(String::from(self.text.as_ref())),
            space_diff: self.space_diff,
        }
    }

    /// Returns whether two tokens are idential with whitespace removed.
    pub fn similar(&self, token: &Token) -> bool {
        if self.kind != token.kind {
            return false;
        }

        let mut a = self.text.split_whitespace();
        let mut b = token.text.split_whitespace();
        a.next() == b.next()
    }

    /// Returns whether one token is a prefix or another.
    pub fn is_prefix_of(&self, token: &Token) -> bool {
        if self.kind != token.kind {
            return false;
        }

        token.text.starts_with(self.text.as_ref())
    }

    /// Return the amount of space following the text of the token.
    pub fn space_suffix(&self) -> usize {
        let mut a = self.text.split_whitespace();
        self.text.len() - a.next().unwrap().len()
    }
}

/// Wrapper for the traversal of a borrowed list of tokens.
#[derive(Debug, Clone)]
pub struct TokenVec<'a> {
    tokens: Cow<'a, [Token<'a>]>,
    pos: usize,
}

impl<'a> TokenVec<'a> {
    pub fn new(tokens: &'a [Token<'a>]) -> Self {
        Self {
            tokens: Cow::Borrowed(tokens),
            pos: 0,
        }
    }

    pub fn clone_owned(&self) -> TokenVec<'static> {
        let mut v = vec![];

        for token in self.tokens.iter() {
            v.push(token.clone_owned());
        }

        TokenVec {
            tokens: Cow::Owned(v),
            pos: self.pos,
        }
    }

    pub fn take_pos(&mut self, other: usize) {
        self.pos = other;
    }

    pub fn pos(&self) -> usize {
        self.pos
    }

    pub fn new_empty() -> Self {
        Self {
            tokens: Cow::Borrowed(&[]),
            pos: 0,
        }
    }

    pub fn remaining(&self) -> usize {
        self.tokens.len() - self.pos
    }

    pub fn has_remaining(&self) -> bool {
        self.remaining() > 0
    }

    pub fn is_empty(&self) -> bool {
        self.remaining() == 0
    }

    pub fn top(&self) -> &Token<'a> {
        &self.tokens[self.pos]
    }

    pub fn top_kind(&self) -> &TokenKind {
        &self.tokens[self.pos].kind
    }

    pub fn len(&self) -> usize {
        self.tokens.len()
    }

    pub fn advance(&mut self, count: usize) {
        self.pos += count;
        if self.pos > self.tokens.len() {
            panic!("invalid token advance {} > {}", self.pos, self.tokens.len());
        }
    }

    pub fn step(&mut self) {
        self.advance(1)
    }
}

#[derive(Debug)]
pub enum Error {
    Pest(pest::error::Error<Rule>),
    IntError(std::num::ParseIntError),
    RonError(ron::de::Error),
}

/// Parse a string into a vector of tokens.
pub fn parse_to_tokens<'a>(s: &'a str) -> Result<Vec<Token<'a>>, Error> {
    let mut vec = vec![];

    let pairs = ExprParser::parse(Rule::token_list, s).map_err(Error::Pest)?;
    let mut last_end = 0;

    for pair in pairs {
        let span = pair.clone().as_span();
        let mut stop = false;

        let token_inner = match pair.as_rule() {
            Rule::identifier => TokenKind::Ident,
            Rule::nonnegative_decimal => {
                TokenKind::NonNegativeDecimal({ span.as_str().parse().map_err(Error::IntError)? })
            }
            Rule::decimal => {
                TokenKind::Decimal({ ron::de::from_str(span.as_str()).map_err(Error::RonError)? })
            }
            Rule::invalid => {
                stop = true;
                TokenKind::InvalidToken
            }
            Rule::field_access => TokenKind::FieldAccess,
            Rule::subscript_open => TokenKind::SubscriptOpen,
            Rule::subscript_close => TokenKind::SubscriptClose,
            Rule::tuple_open => TokenKind::TupleOpen,
            Rule::tuple_close => TokenKind::TupleClose,
            Rule::curly_open => TokenKind::CurlyOpen,
            Rule::curly_close => TokenKind::CurlyClose,
            Rule::comma => TokenKind::Comma,
            Rule::colon => TokenKind::Colon,
            Rule::asterix => TokenKind::Asterix,
            Rule::char_literal => {
                TokenKind::Char({ ron::de::from_str(span.as_str()).map_err(Error::RonError)? })
            }
            Rule::assign => TokenKind::Assign,
            Rule::range_access => TokenKind::Range(false),
            Rule::range_access_inclusive => TokenKind::Range(true),
            Rule::string_literal => {
                TokenKind::String({ ron::de::from_str(span.as_str()).map_err(Error::RonError)? })
            }
            Rule::underscore
            | Rule::alpha
            | Rule::alphanumeric
            | Rule::digit
            | Rule::nonzero
            | Rule::token
            | Rule::negative_decimal
            | Rule::escape_sequence
            | Rule::whitespace_char
            | Rule::literal_char
            | Rule::single_literal_char
            | Rule::token_list
            | Rule::WHITESPACE => {
                continue;
            }
        };

        vec.push(Token {
            kind: token_inner,
            text: Cow::Borrowed(span.as_str()),
            space_diff: span.start() - last_end,
        });

        if stop {
            break;
        }

        last_end = span.end();
    }

    Ok(vec)
}
