#![allow(warnings)]

use never::Never;

use std::{borrow::Borrow, convert::Infallible, env, fmt::Debug, iter::Peekable, str::Chars};

pub fn from_args<'a, T: FromArgs<'a>>(
    args: impl Iterator<Item = &'a str>,
) -> Result<T, Vec<<T::Init as PollInit<'a>>::Err>> {
    let (init, mut errs) = Token::parse(args).fold(
        (T::Init::default(), Vec::new()),
        |(mut acc, mut vec), token| {
            if let Err(err) = acc.poll_init(token) {
                vec.push(err);
            }
            (acc, vec)
        },
    );

    match init.finish() {
        Ok(res) if errs.is_empty() => Ok(res),
        Ok(_) => Err(errs),
        Err(e) => {
            errs.push(e);
            Err(errs)
        }
    }
}

#[derive(Debug)]
pub enum Error<'a, VP = Never> {
    Parse(ParseError<'a>),
    UnknownOption(Token<'a>),
    UnexpectedMulti(Token<'a>),
    ExpectedValue(Token<'a>),
    UnexpectedValue(Token<'a>),
    ExpectedPositional(Token<'a>),
    UnexpectedPositional(Token<'a>),
    RequiredOption(&'static str), // TODO: may not be strign
    ValueParse(VP),
}

impl<'a, T> From<ParseError<'a>> for Error<'a, T> {
    fn from(pe: ParseError<'a>) -> Self {
        Error::Parse(pe)
    }
}

impl<T> From<Infallible> for Error<'_, T> {
    fn from(inf: Infallible) -> Self {
        match inf {}
    }
}

// /// ???
pub trait FromArgs<'a>: Sized {
    type Init: PollInit<'a, Output = Self>;
}

pub trait PollInit<'a>: Default {
    type Output;
    type Err: From<ParseError<'a>>;

    fn poll_init(&mut self, token: Token<'a>) -> Result<(), Self::Err>;

    fn finish(self) -> Result<Self::Output, Self::Err>;
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Token<'a> {
    Positional(&'a str),
    Short {
        key: char,
        value: Option<&'a str>,
    },
    Long {
        key: &'a str,
        value: Option<&'a str>,
    },
    DashDash,
    Error(ParseError<'a>),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ParseError<'a> {
    RepeatedKeyNotMatch {
        expected: char,
        found: char,
        full: &'a str,
    },
    EmptyShortKey,
}

impl<'a> Token<'a> {
    pub fn parse<I>(args: I) -> Parse<'a, I>
    where
        I: Iterator<Item = &'a str>,
    {
        Parse {
            args: args.peekable(),
            shorts: "".chars().peekable(),
            pos_only: false,
        }
    }
}

pub struct Parse<'a, I: Iterator<Item = &'a str>> {
    args: Peekable<I>,
    shorts: Peekable<Chars<'a>>,
    pos_only: bool,
}

impl<'a, I: Iterator<Item = &'a str>> Iterator for Parse<'a, I> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(short) = self.shorts.next() {
            return Some(Token::Short {
                key: short,
                // We do not parse values after many shorts, i.e.
                // "-vv x" => [Short('v'), Short('v'), Positional("x")]
                // while
                // "-v -v x" => [Short('v'), Short('v', value: "x")]
                value: None,
            });
        }

        let item = self.args.next()?;
        if self.pos_only {
            return Some(Token::Positional(item));
        }

        let token = match item {
            "-" => unimplemented!("???"),
            "--" => {
                self.pos_only = true;
                Token::DashDash
            }
            key if key.starts_with("--") => Token::Long {
                key: &key[2..],
                value: next_value(&mut self.args),
            },
            keys if keys.starts_with("-") => {
                let mut chars = keys[1..].chars().peekable();
                match (chars.next(), chars.peek()) {
                    (None, _) => unreachable!("'-' is checked before"),
                    (Some(key), None) => Token::Short {
                        key,
                        value: next_value(&mut self.args),
                    },
                    (Some(key), Some(_)) => {
                        self.shorts = chars;
                        Token::Short { key, value: None }
                    }
                }
            }
            word => Token::Positional(word),
        };

        Some(token)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (lower, upper) = self.args.size_hint();
        (lower / 2, upper)
    }
}

fn next_value<'a>(args: &mut Peekable<impl Iterator<Item = &'a str>>) -> Option<&'a str> {
    match args.peek() {
        Some(&x) if !x.starts_with('-') => {
            let _ = args.next();
            Some(x)
        }
        _ => None,
    }
}

// bool::then is unstable
fn then<T, F: FnOnce() -> T>(b: bool, f: F) -> Option<T> {
    if b {
        Some(f())
    } else {
        None
    }
}

#[doc(hidden)]
pub fn try_insert<T: Debug, E: Debug>(
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

#[doc(hidden)]
pub fn try_set<E>(b: &mut bool, f: impl FnOnce() -> E) -> Result<(), E> {
    if *b {
        Err(f())
    } else {
        *b = true;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{convert::TryInto, str::FromStr};

    use crate::{try_insert, try_set, Error, FromArgs, PollInit, Token};

    #[test]
    fn ast_parse() {
        let args = [
            "unparsed",
            "-vvvv",
            "-v",
            "val",
            "-xx",
            "1",
            "unp",
            "--python",
            "48",
            "--loooong",
            "-okk",
            "--",
            "--option",
            "-x",
        ];
        let expected = [
            Token::Positional("unparsed"),
            Token::Short {
                key: 'v',
                value: None,
            },
            Token::Short {
                key: 'v',
                value: None,
            },
            Token::Short {
                key: 'v',
                value: None,
            },
            Token::Short {
                key: 'v',
                value: None,
            },
            Token::Short {
                key: 'v',
                value: Some("val"),
            },
            Token::Short {
                key: 'x',
                value: None,
            },
            Token::Short {
                key: 'x',
                value: None,
            },
            Token::Positional("1"),
            Token::Positional("unp"),
            Token::Long {
                key: "python",
                value: Some("48"),
            },
            Token::Long {
                key: "loooong",
                value: None,
            },
            Token::Short {
                key: 'o',
                value: None,
            },
            Token::Short {
                key: 'k',
                value: None,
            },
            Token::Short {
                key: 'k',
                value: None,
            },
            Token::DashDash,
            Token::Positional("--option"),
            Token::Positional("-x"),
        ];

        assert_eq!(
            Token::parse(args.iter().copied()).collect::<Vec<_>>(),
            &expected
        );
    }

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
    }
    #[derive(Debug)]
    struct TestInit {
        // only used if there is no need in things after --
        term: bool,
        a: Option<String>, // raw
        b: Option<i32>,    // parse
        c: bool,           // switch
        d: u32,            // count
        x: Option<Option<String>>, // default+wrap(Some)
    }
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
                Token::Positional(_) => Err(crate::Error::UnexpectedPositional(todo!())),
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
                } => try_set(&mut self.c, || Error::UnexpectedMulti(t)),
                t
                @
                Token::Short {
                    key: 'c',
                    value: Some(_),
                } => Err(Error::UnexpectedValue(t)),

                Token::Short {
                    key: 'd',
                    value: None,
                } => {
                    self.d += 1;
                    Ok(())
                }
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
                } => Ok(Test { a, b, c, d, x: x.unwrap_or_default() }),
            }
        }
    }
    impl Default for TestInit {
        fn default() -> Self {
            Self {
                term: false,
                a: None,
                b: None,
                c: false,
                d: 0,
                x: Default::default(),
            }
        }
    }

    #[test]
    fn expanded() {
        let args = ["-a", "a_val", "-c", "-ddd", "-d", "-b", "42"];
        let res: Test = crate::from_args(args.iter().copied()).unwrap();
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
