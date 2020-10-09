use std::{fmt::Display, iter::Peekable, str::Chars};

use crate::tr::IntoOwned;

/// A single token of command line arguments.
///
/// Tokens can be uptained by [parsing] iterator of strings.
///
/// [parsing]: Token::parse
///
/// ## Parsing notes
///
/// - `-xyz` is parsed as 3 short options `x`, `y` and `z`.
/// - `-xyz value` is parsed as 3 short options and a positional argument (If user wants to bind value to `z` it needs to write `-xy -z value`).
/// - `-x -y` parsed as 2 short options `x` and `y`.
/// - `-x -` is parsed as short option `x` with value `-`.
/// - Everything after `--` token parsed as a positional.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Token<'a> {
    /// Positional argument, i.e. just `something`.
    Positional(&'a str),
    // Short option, i.e. `-k`, `-k value`.
    Short {
        key: char,
        value: Option<&'a str>,
    },
    // Long option, i.e. `--key`, `--key value`.
    Long {
        key: &'a str,
        value: Option<&'a str>,
    },
    // Duble dash, i.e. `--`.
    DashDash,
}

impl<'a> Token<'a> {
    /// Created parsing iterator.
    ///
    /// For parse notes see [`Token`](Token#parsing-notes)
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

impl Display for Token<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Positional(s) => f.write_str(s),
            Token::Short { key, value } => {
                '-'.fmt(f).and_then(|()| key.fmt(f))?;
                if let Some(value) = value {
                    ' '.fmt(f).and_then(|()| value.fmt(f))?;
                }
                Ok(())
            }
            Token::Long { key, value } => {
                "--".fmt(f).and_then(|()| key.fmt(f))?;
                if let Some(value) = value {
                    ' '.fmt(f).and_then(|()| value.fmt(f))?;
                }
                Ok(())
            }
            Token::DashDash => "--".fmt(f),
        }
    }
}

#[derive(Debug)]
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
            "--" => {
                self.pos_only = true;
                Token::DashDash
            }
            key if key.starts_with("--") => Token::Long {
                key: &key[2..],
                value: next_value(&mut self.args),
            },
            keys if keys.starts_with('-') => {
                let mut chars = keys[1..].chars().peekable();
                match (chars.next(), chars.peek()) {
                    (None, _) => Token::Positional("-"),
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

impl OwnToken {
    /// Borrow owned token as borrowed token.
    pub fn borrow(&self) -> Token {
        match self {
            OwnToken::Positional(s) => Token::Positional(s.as_ref()),
            OwnToken::Short { key, value } => Token::Short {
                key: *key,
                value: value.as_ref().map(<_>::as_ref),
            },
            OwnToken::Long { key, value } => Token::Long {
                key: key.as_ref(),
                value: value.as_ref().map(<_>::as_ref),
            },
            OwnToken::DashDash => Token::DashDash,
        }
    }
}

impl Display for OwnToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.borrow().fmt(f)
    }
}

impl IntoOwned for Token<'_> {
    type Owned = OwnToken;

    fn into_owned(self) -> Self::Owned {
        match self {
            Token::Positional(s) => OwnToken::Positional(s.into()),
            Token::Short { key, value } => OwnToken::Short {
                key,
                value: value.map(Into::into),
            },
            Token::Long { key, value } => OwnToken::Long {
                key: key.into(),
                value: value.map(Into::into),
            },
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
        "-",
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
        Token::Positional("-"),
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
