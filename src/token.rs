use std::{iter::Peekable, str::Chars};

use crate::tr::IntoOwned;

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
        let (lower, _) = self.args.size_hint();
        (lower / 2, None) // Upper is unknown because of -vv => [Short('v'), Short('v')]
    }
}


#[derive(Debug, Clone, Eq, PartialEq)]
pub enum OwnToken {
    Positional(Box<str>),
    Short {
        key: char,
        value: Option<Box<str>>,
    },
    Long {
        key: Box<str>,
        value: Option<Box<str>>,
    },
    DashDash,
}

impl IntoOwned for Token<'_> {
    type Owned = OwnToken;

    fn into_owned(self) -> Self::Owned {
        match self {
            Token::Positional(s) => OwnToken::Positional(s.into()),
            Token::Short { key, value } => OwnToken::Short { key, value: value.map(Into::into) },
            Token::Long { key, value } => OwnToken::Long { key: key.into(), value: value.map(Into::into) },
            Token::DashDash => OwnToken::DashDash,
        }
    }
}

// If next value in iterator doesn't start with ('-' + any char) returns Some(next), otherwise returns None
fn next_value<'a>(args: &mut Peekable<impl Iterator<Item = &'a str>>) -> Option<&'a str> {
    match args.peek() {
        Some(&x) if x == "-" || !x.starts_with('-') => args.next(),
        _ => None,
    }
}

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
        "-",
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
            value: Some("-"),
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
