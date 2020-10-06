// private, but reexported
mod error;
mod from_args;
mod token;

pub use error::{Error, SwitchAlreadySetError, TokenizerError, TooManyOptionsError};
pub use from_args::FromArgsIter;
pub use token::{Parse, Token};

pub mod tr;

pub fn from_args<'a, T, A>(args: A) -> FromArgsIter<'a, T, A>
where
    T: FromArgs<'a>,
    A: Iterator<Item = &'a str>,
{
    FromArgsIter {
        parser: Token::parse(args),
        init: Some(T::initializer()),
    }
}

pub fn collect_from_args<'a, T, A>(args: A) -> Result<T, Vec<<T::Init as PollInit<'a>>::Err>>
where
    T: FromArgs<'a>,
    A: Iterator<Item = &'a str>,
{
    fn flip<L, R>(either: Result<L, R>) -> Result<R, L> {
        match either {
            Ok(l) => Err(l),
            Err(r) => Ok(r),
        }
    }

    flip(from_args(args).map(flip).collect())
}

pub fn from_env<T, E>() -> Result<T, Vec<E>>
where
    // FIXME: we probably need ToOwned here because out error borrows
    for<'a> T: FromArgs<'a>,
    for<'a> <T as FromArgs<'a>>::Init: PollInit<'a, Err = E>,
{
    use std::env;
    let args: Vec<_> = env::args().skip(1).collect();
    collect_from_args(args.iter().map(String::as_ref))
}

/// Type that can be created from command line arguments.
pub trait FromArgs<'a>: Sized {
    type Init: PollInit<'a, Output = Self>;

    fn initializer() -> Self::Init;
}

pub trait PollInit<'a> {
    type Output;
    type Err: From<TokenizerError<'a>>;

    fn poll_init(&mut self, token: Token<'a>) -> Result<(), Self::Err>;

    fn finish(self) -> Result<Self::Output, Self::Err>;
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

        type Err = crate::Error<'a, TestParseError>;

        fn poll_init(&mut self, token: Token<'a>) -> Result<(), Self::Err> {
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
                            .map_err(|err| Error::ValueParse(TestParseError::a(err)))
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
                            .map_err(|err| Error::ValueParse(TestParseError::b(err)))
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
                            .map_err(|err| Error::ValueParse(TestParseError::x(err)))
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

                Token::Error(_) => todo!(),
            }
        }

        fn finish(self) -> Result<Self::Output, Self::Err> {
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
}
