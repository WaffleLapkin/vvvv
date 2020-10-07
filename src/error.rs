use std::convert::Infallible;

use crate::{tr::IntoOwned, OwnToken, Token};

/// Error occured when parsing command line arguments.
#[derive(Debug)]
pub enum Error<'a, C = Infallible> {
    /// Unknown option. i.e. `-x` wass provided, but not expected.
    UnknownOption(Token<'a>),
    /// Unexpected multiple options. I.e.: `--opt --opt` were provided, but `opt` is a flag.
    UnexpectedMulti(Token<'a>),
    /// Expected value was not provided. I.e.: `-o` was provided, but `-o` option requires value (`-o value`).
    ExpectedValue(Token<'a>),
    /// Value was provided but not expected. I.e.: `--opt value` was provided, but `--opt` is flag.
    UnexpectedValue(Token<'a>),
    /// Expected positional argument, but option was provided. I.e. expected `positional` but `--opt` was provided.
    ExpectedPositional(Token<'a>),
    /// Unexpected positional argument. I.e. all positional arguments are already .
    UnexpectedPositional(Token<'a>),
    /// Requires option was not present. I.e. `-x <val>` option was required but not provided.
    RequiredOption(&'static str), // TODO: may not be strign
    /// Too many options caused an overflow of the counter. I.e. counter type is `u8` and user provided `-vvvvvvvvvvvvvvvv -vvvvvvvvvvvvvvvv -vvvvvvvvvvvvvvvv -vvvvvvvvvvvvvvvv -vvvvvvvvvvvvvvvv -vvvvvvvvvvvvvvvv -vvvvvvvvvvvvvvvv -vvvvvvvvvvvvvvvv -vvvvvvvvvvvvvvvv -vvvvvvvvvvvvvvvv -vvvvvvvvvvvvvvvv -vvvvvvvvvvvvvvvv -vvvvvvvvvvvvvvvv -vvvvvvvvvvvvvvvv -vvvvvvvvvvvvvvvv -vvvvvvvvvvvvvvvv`.
    TooManyOptions(Token<'a>),
    /// Custom error. Usually this represents parsing errors (e.g. `<i32 as FromStr>::Err | <PathBuf as FromStr>::Err`).
    Custom(C),
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
            Error::Custom(t) => OwnError::Custom(t),
        }
    }
}

impl<'a, E> From<Error<'a, E>> for OwnError<E> {
    fn from(x: Error<'a, E>) -> Self {
        x.into_owned()
    }
}

#[derive(Debug)]
pub enum OwnError<C = Infallible> {
    /// Unknown option. i.e. `-x` wass provided, but not expected.
    UnknownOption(OwnToken),
    /// Unexpected multiple options. I.e.: `--opt --opt` were provided, but `opt` is a flag.
    UnexpectedMulti(OwnToken),
    /// Expected value was not provided. I.e.: `-o` was provided, but `-o` option requires value (`-o value`).
    ExpectedValue(OwnToken),
    /// Value was provided but not expected. I.e.: `--opt value` was provided, but `--opt` is flag.
    UnexpectedValue(OwnToken),
    /// Expected positional argument, but option was provided. I.e. expected `positional` but `--opt` was provided.
    ExpectedPositional(OwnToken),
    /// Unexpected positional argument. I.e. all positional arguments are already .
    UnexpectedPositional(OwnToken),
    /// Requires option was not present. I.e. `-x <val>` option was required but not provided.
    RequiredOption(&'static str), // TODO: may not be strign
    /// Too many options caused an overflow of the counter. I.e. counter type is `u8` and user provided `-vvvvvvvvvvvvvvvv -vvvvvvvvvvvvvvvv -vvvvvvvvvvvvvvvv -vvvvvvvvvvvvvvvv -vvvvvvvvvvvvvvvv -vvvvvvvvvvvvvvvv -vvvvvvvvvvvvvvvv -vvvvvvvvvvvvvvvv -vvvvvvvvvvvvvvvv -vvvvvvvvvvvvvvvv -vvvvvvvvvvvvvvvv -vvvvvvvvvvvvvvvv -vvvvvvvvvvvvvvvv -vvvvvvvvvvvvvvvv -vvvvvvvvvvvvvvvv -vvvvvvvvvvvvvvvv`.
    TooManyOptions(OwnToken),
    /// Custom error. Usually this represents parsing errors (e.g. `<i32 as FromStr>::Err | <PathBuf as FromStr>::Err`).
    Custom(C),
}

pub struct SwitchAlreadySetError;

pub struct TooManyOptionsError;
