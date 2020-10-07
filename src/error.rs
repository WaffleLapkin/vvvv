use std::convert::Infallible;

use crate::{OwnToken, Token, tr::IntoOwned};

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

impl<'a, E> IntoOwned for Error<'a, E> {
    type Owned = OwnError<E>;

    fn into_owned(self) -> Self::Owned {
        match self {
            Error::UnknownOption(t) => OwnError::UnknownOption(t.into_owned()),
            Error::UnexpectedMulti(t) => OwnError::UnexpectedMulti(t.into_owned()),
            Error::ExpectedValue(t) => OwnError::ExpectedValue(t.into_owned()),
            Error::UnexpectedValue(t) => OwnError::UnexpectedValue(t.into_owned()),
            Error::ExpectedPositional(t) => OwnError::ExpectedPositional(t.into_owned()),
            Error::UnexpectedPositional(t) => OwnError::UnexpectedPositional(t.into_owned()),
            Error::RequiredOption(t) => OwnError::RequiredOption(t),
            Error::TooManyOptions(t) => OwnError::TooManyOptions(t.into_owned()),
            Error::ValueParse(t) => OwnError::ValueParse(t),
        }
    }
}

impl<'a, E> From<Error<'a, E>> for OwnError<E> {
    fn from(x: Error<'a, E>) -> Self {
        x.into_owned()
    }
}

#[derive(Debug)]
pub enum OwnError<VP = Infallible> {
    UnknownOption(OwnToken),
    UnexpectedMulti(OwnToken),
    ExpectedValue(OwnToken),
    UnexpectedValue(OwnToken),
    ExpectedPositional(OwnToken),
    UnexpectedPositional(OwnToken),
    RequiredOption(&'static str), // TODO: may not be strign
    TooManyOptions(OwnToken),
    ValueParse(VP),
}

pub struct SwitchAlreadySetError;

pub struct TooManyOptionsError;
