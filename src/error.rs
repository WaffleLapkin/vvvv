use std::convert::Infallible;

use crate::Token;

#[derive(Debug)]
pub enum Error<'a, VP = Infallible> {
    Tokenizer(TokenizerError<'a>),
    UnknownOption(Token<'a>),
    UnexpectedMulti(Token<'a>),
    ExpectedValue(Token<'a>),
    UnexpectedValue(Token<'a>),
    ExpectedPositional(Token<'a>),
    UnexpectedPositional(Token<'a>),
    RequiredOption(&'static str), // TODO: may not be strign
    TooManyOptions(Token<'a>),
    ValueParse(VP),
}

impl<'a, T> From<TokenizerError<'a>> for Error<'a, T> {
    fn from(te: TokenizerError<'a>) -> Self {
        Error::Tokenizer(te)
    }
}

impl<T> From<Infallible> for Error<'_, T> {
    fn from(inf: Infallible) -> Self {
        match inf {}
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum TokenizerError<'a> {
    RepeatedKeyNotMatch {
        expected: char,
        found: char,
        full: &'a str,
    },
    EmptyShortKey,
}

pub struct SwitchAlreadySetError;

pub struct TooManyOptionsError;
