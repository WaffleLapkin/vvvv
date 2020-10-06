use std::convert::Infallible;

use crate::Token;

#[derive(Debug)]
pub enum Error<'a, VP = Infallible> {
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

impl<T> From<Infallible> for Error<'_, T> {
    fn from(inf: Infallible) -> Self {
        match inf {}
    }
}

pub struct SwitchAlreadySetError;

pub struct TooManyOptionsError;
