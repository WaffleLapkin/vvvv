// private, but reexported
mod error;
mod from_args;
mod token;

pub use error::{Error, OwnError, SwitchAlreadySetError, TooManyOptionsError};
pub use from_args::FromArgsIter;
pub use token::{OwnToken, Parse, Token};

pub mod own;
pub mod tr;

use std::env;

use own::{FromArgsOwned, PollInitOwned};
use tr::IntoOwned;

/// Creates `T` from `Iterator<Item = &str>`.
///
/// Returns first error, if any has accured.
///
/// Note: First element of the iteratpr is **not** ignored. If you are using [`env::args`] use `.skip(1)`.
pub fn from_args<'a, T, A>(args: A) -> Result<T, Error<'a, <T::Init as PollInit<'a>>::Err>>
where
    T: FromArgs<'a>,
    A: Iterator<Item = &'a str>,
{
    from_args_iter(args)
        .next()
        // `FromArgsIter` always has at least 1 evlement
        .unwrap()
}

/// Creates [`FromArgsIter`] from `Iterator<Item = &str>`.
///
/// The iterator will return all (if any) accured errors followed by result of `T::finish`.
///
/// Note: First element of the `args` iterator is **not** ignored.
/// If you are using [`env::args`] use `.skip(1)`.
///
/// For more easy-to-use function see [`from_args`](from_args()), [`collect_from_args`] and [`from_env`]
pub fn from_args_iter<'a, T, A>(args: A) -> FromArgsIter<'a, T, A>
where
    T: FromArgs<'a>,
    A: Iterator<Item = &'a str>,
{
    FromArgsIter {
        parser: Token::parse(args),
        init: Some(T::initializer()),
    }
}

/// Creates `T` from `Iterator<Item = &str>`.
///
/// Returns all errors, if any has accured.
///
/// Note: First element of the iteratpr is **not** ignored. If you are using [`env::args`] use `.skip(1)`.
pub fn collect_from_args<'a, T, A>(
    args: A,
) -> Result<T, Vec<crate::Error<'a, <T::Init as PollInit<'a>>::Err>>>
where
    T: FromArgs<'a>,
    A: Iterator<Item = &'a str>,
{
    let mut iter = from_args_iter(args);
    let mut errors = match iter.next() {
        None => return Err(Vec::new()),
        Some(Ok(ok)) => return Ok(ok),
        Some(Err(err)) => vec![err],
    };

    for res in iter {
        if let Err(err) = res {
            errors.push(err);
        }
    }

    Err(errors)
}

/// Creates `T` from [`env::args`].
///
/// ## Notes
///
/// 1. Executable path is ignored.
/// 2. `T` can't borrow anything from args.
pub fn from_env<T>() -> Result<T, Vec<OwnError<<T::OwnInit as PollInitOwned>::OwnErr>>>
where
    T: FromArgsOwned,
{
    let args: Vec<_> = env::args().skip(1).collect();
    let mut iter = from_args_iter(args.iter().map(String::as_str));
    let mut errors = match iter.next() {
        None => return Err(Vec::new()),
        Some(Ok(ok)) => return Ok(ok),
        Some(Err(err)) => vec![err.into_owned()],
    };

    for res in iter {
        if let Err(err) = res {
            errors.push(err.into_owned());
        }
    }

    Err(errors)
}

/// Type that can be created from command line arguments.
///
/// To create implementator of this trait, use [`from_args`](from_args()), [`from_args_iter`], [`collect_from_args`] or [`from_env`].
pub trait FromArgs<'a>: Sized {
    /// Initializer of this type which holds possibly uninitialized data.
    type Init: PollInit<'a, Output = Self>;

    /// Returns initializer of this type.
    fn initializer() -> Self::Init;
}

/// Polling initializer of type `Self::Output`.
pub trait PollInit<'a> {
    /// Type beeing initialized.
    type Output;

    /// Type of errors which may happen additionally to [`vvvv::Error`](crate::Error)
    type Err;

    /// Sink `token` to progress in initializing `Self::Output`.
    fn poll_init(&mut self, token: Token<'a>) -> Result<(), crate::Error<'a, Self::Err>>;

    /// Finish the initialization.
    ///
    /// Returns error if output type can't be created from given tokens.
    fn finish(self) -> Result<Self::Output, crate::Error<'a, Self::Err>>;
}

#[doc(hidden)]
pub fn try_insert<T, E>(
    opt: &mut Option<T>,
    val: impl FnOnce() -> Result<T, E>,
    f: impl FnOnce() -> E,
) -> Result<(), E> {
    match opt {
        Some(_) => Err(f()),
        None => {
            *opt = Some(val()?);
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::{tr::Counter, tr::Switch, try_insert, Error, FromArgs, PollInit, Token};

    #[derive(Debug, Eq, PartialEq)]
    struct Test {
        a: String,         // raw
        b: i32,            // parse
        c: bool,           // switch
        d: u32,            // count
        x: Option<String>, // default+wrap(Some)
    }

    // Imagine it's generated via derive
    impl<'a> FromArgs<'a> for Test {
        type Init = TestInit;

        fn initializer() -> Self::Init {
            TestInit {
                term: false,
                a: None,
                b: None,
                c: false,
                d: 0,
                x: Default::default(),
            }
        }
    }
    #[derive(Debug)]
    struct TestInit {
        // only used if there is no need in things after --
        term: bool,
        a: Option<String>,         // raw
        b: Option<i32>,            // parse
        c: bool,                   // switch
        d: u32,                    // count
        x: Option<Option<String>>, // default+wrap(Some)
    }

    #[allow(non_camel_case_types)]
    #[derive(Debug)]
    enum TestParseError {
        a(<String as FromStr>::Err),
        b(<i32 as FromStr>::Err),
        x(<String as FromStr>::Err),
    }

    impl<'a> PollInit<'a> for TestInit {
        type Output = Test;

        type Err = TestParseError;

        fn poll_init(&mut self, token: Token<'a>) -> Result<(), crate::Error<'a, Self::Err>> {
            if self.term {
                return Ok(());
            }

            match token {
                t @ Token::Positional(_) => Err(crate::Error::UnexpectedPositional(t)),
                Token::Short {
                    key: 'a',
                    value: Some(v),
                } => try_insert(
                    &mut self.a,
                    || {
                        v.parse()
                            .map_err(|err| Error::Custom(TestParseError::a(err)))
                    },
                    || {
                        crate::Error::UnexpectedMulti(Token::Short {
                            key: 'a',
                            value: Some(v),
                        })
                    },
                ),
                t @ Token::Short { key: 'a', value: _ } => Err(Error::ExpectedValue(t)),
                Token::Short {
                    key: 'b',
                    value: Some(v),
                } => try_insert(
                    &mut self.b,
                    || {
                        v.parse()
                            .map_err(|err| Error::Custom(TestParseError::b(err)))
                    },
                    || {
                        crate::Error::UnexpectedMulti(Token::Short {
                            key: 'b',
                            value: Some(v),
                        })
                    },
                ),
                t @ Token::Short { key: 'b', value: _ } => Err(Error::ExpectedValue(t)),

                t
                @
                Token::Short {
                    key: 'c',
                    value: None,
                } => Switch::set(&mut self.c).map_err(|_| Error::UnexpectedMulti(t)),
                t
                @
                Token::Short {
                    key: 'c',
                    value: Some(_),
                } => Err(Error::UnexpectedValue(t)),

                t
                @
                Token::Short {
                    key: 'd',
                    value: None,
                } => self.d.inc().map_err(|_| Error::TooManyOptions(t)),
                t
                @
                Token::Short {
                    key: 'd',
                    value: Some(_),
                } => Err(Error::UnexpectedValue(t)),

                t
                @
                Token::Short {
                    key: 'x',
                    value: None,
                } => Err(Error::ExpectedValue(t)),
                // FIXME: we can't use try_insert here, Option<> was original
                Token::Short {
                    key: 'x',
                    value: Some(v),
                } => try_insert(
                    &mut self.x,
                    || {
                        v.parse()
                            .map(Some)
                            .map_err(|err| Error::Custom(TestParseError::x(err)))
                    },
                    || {
                        crate::Error::UnexpectedMulti(Token::Short {
                            key: 'x',
                            value: Some(v),
                        })
                    },
                ),

                t @ Token::Short { .. } => Err(Error::UnknownOption(t)),
                t @ Token::Long { .. } => Err(Error::UnknownOption(t)),

                Token::DashDash => {
                    self.term = true;
                    Ok(())
                }
            }
        }

        fn finish(self) -> Result<Self::Output, crate::Error<'a, Self::Err>> {
            match self {
                Self { a: None, .. } => Err(Error::RequiredOption("a")),
                Self { b: None, .. } => Err(Error::RequiredOption("b")),
                Self {
                    a: Some(a),
                    b: Some(b),
                    c,
                    d,
                    x,
                    term: _,
                } => Ok(Test {
                    a,
                    b,
                    c,
                    d,
                    x: x.unwrap_or_default(),
                }),
            }
        }
    }

    #[test]
    fn expanded() {
        let args = ["-a", "a_val", "-c", "-ddd", "-d", "-b", "42"];
        let res: Test = crate::collect_from_args(args.iter().copied()).unwrap();
        assert_eq!(
            res,
            Test {
                a: String::from("a_val"),
                b: 42,
                c: true,
                d: 4,
                x: None,
            }
        )
    }

    #[allow(dead_code)]
    fn from_env_is_callable() {
        let _: Test = crate::from_env().unwrap();
    }
}
