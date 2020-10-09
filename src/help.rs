use std::{
    io::{self, Write},
    iter::once,
};

use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::dumb_wrap::Wrap;

pub enum Description<'a> {
    None,
    Raw(&'a str),
    Typed {
        descr: &'a str,
        // TODO: usage probably should be created from `positionals` and `options`
        usage: &'a str,
        positionals: &'a [Pos<'a>],
        options: &'a [Opt<'a>],
    },
}

impl Description<'_> {
    pub fn print(&self, width_limit: Option<usize>) -> io::Result<()> {
        self.write_i(&mut io::stdout().lock(), width_limit, 4)
    }

    fn write_i(
        &self,
        writer: &mut dyn Write,
        width_limit: Option<usize>,
        indent: usize,
    ) -> io::Result<()> {
        match self {
            Description::None => Ok(()),
            Description::Raw(s) => writer.write_all(s.as_bytes()),
            Description::Typed {
                descr,
                usage,
                positionals,
                options,
            } => {
                let limit = width_limit.unwrap_or(usize::MAX);

                // here starts the 'ugh' part

                for it in Wrap::new(limit, once(*descr)) {
                    match it {
                        crate::dumb_wrap::Item::Part(p) => writer.write_all(p.as_bytes())?,
                        crate::dumb_wrap::Item::Break(l) => {
                            writer.write_all(l.as_bytes())?;
                            writer.write_all(b"\n")?;
                        }
                    }
                }

                writer.write_all(b"\n\nUsage:\n  ")?;

                for it in Wrap::new(limit - 2, once(*usage)) {
                    match it {
                        crate::dumb_wrap::Item::Part(p) => writer.write_all(p.as_bytes())?,
                        crate::dumb_wrap::Item::Break(l) => {
                            writer.write_all(l.as_bytes())?;
                            writer.write_all(b"\n  ")?;
                        }
                    }
                }

                // 64 spaces (I don't belive in indentension that takes more than 80% of default term)
                const SPACES: &[u8] =
                    b"                                                                ";

                if !positionals.is_empty() {
                    writer.write_all(b"\n\nPositional arguments:\n")?;
                    let max = positionals
                        .iter()
                        .map(|pos| pos.name.width())
                        .max()
                        .unwrap_or(0);
                    let descr_ind = indent + max + indent;

                    for pos in *positionals {
                        writer.write_all(&SPACES[..indent])?;
                        writer.write_all(pos.name.as_bytes())?;
                        writer.write_all(&SPACES[..(descr_ind - pos.name.width() - indent)])?;

                        for x in Wrap::new(limit - descr_ind, once(pos.descr).chain(once("\n"))) {
                            match x {
                                crate::dumb_wrap::Item::Part(p) => {
                                    writer.write_all(p.as_bytes())?
                                }
                                crate::dumb_wrap::Item::Break(l) => {
                                    writer.write_all(l.as_bytes())?;
                                    writer.write_all(b"\n")?;
                                    writer.write_all(&SPACES[..(descr_ind)])?;
                                }
                            }
                        }
                    }
                }

                if !options.is_empty() {
                    writer.write_all(b"\n\nOptions:\n")?;
                    let max = options.iter().map(|opt| opt.width()).max().unwrap_or(0);
                    let descr_ind = indent + max + indent;

                    for opt in *options {
                        writer.write_all(&SPACES[..indent])?;
                        opt.write(writer)?;
                        writer.write_all(&SPACES[..(descr_ind - opt.width() - indent)])?;

                        let arr;
                        let def = if let Kind::Value {
                            default: Some(def), ..
                        } = opt.kind
                        {
                            arr = [" [default: ", def, "]\n"];
                            arr.iter().copied()
                        } else {
                            ["\n"].iter().copied()
                        };

                        for x in Wrap::new(limit - descr_ind, once(opt.descr).chain(def)) {
                            match x {
                                crate::dumb_wrap::Item::Part(p) => {
                                    writer.write_all(p.as_bytes())?
                                }
                                crate::dumb_wrap::Item::Break(l) => {
                                    writer.write_all(l.as_bytes())?;
                                    writer.write_all(b"\n")?;
                                    writer.write_all(&SPACES[..(descr_ind)])?;
                                }
                            }
                        }
                    }
                }

                Ok(())
            }
        }
    }
}

pub struct Pos<'a> {
    pub name: &'a str,
    pub descr: &'a str,
}

pub struct Opt<'a> {
    pub short: Option<char>,
    pub long: Option<&'a str>,
    pub kind: Kind<'a>,
    pub descr: &'a str,
    pub required: Required<'a>,
}

impl Opt<'_> {
    fn width(&self) -> usize {
        matches!(self.kind, Kind::Value { .. }) as usize * 6
            + match (self.short, self.long) {
                (None, None) => 0,
                (None, Some(l)) => 2 + l.width(),
                (Some(s), None) => 1 + s.width().unwrap_or(0),
                (Some(s), Some(l)) => 5 + s.width().unwrap_or(0) + l.width(),
            }
    }

    fn write(&self, writer: &mut dyn Write) -> io::Result<()> {
        match (self.short, self.long) {
            (None, None) => Ok(()),
            (None, Some(l)) => writer
                .write_all(b"--")
                .and_then(|()| writer.write_all(l.as_bytes())),
            (Some(s), None) => {
                let mut buf = [0; 4];
                let s = s.encode_utf8(&mut buf);
                writer
                    .write_all(b"-")
                    .and_then(|()| writer.write_all(s.as_bytes()))
            }
            (Some(s), Some(l)) => {
                let mut buf = [0; 4];
                let s = s.encode_utf8(&mut buf);
                writer
                    .write_all(b"-")
                    .and_then(|()| writer.write_all(s.as_bytes()))
                    .and_then(|()| writer.write_all(b", --"))
                    .and_then(|()| writer.write_all(l.as_bytes()))
            }
        }?;

        if let Kind::Value { name, .. } = &self.kind {
            match name {
                Some(name) => {
                    writer.write_all(b" <")?;
                    writer.write_all(name.as_bytes())?;
                    writer.write_all(b">")?;
                }
                None => writer.write_all(b" <val>")?,
            }
        }

        Ok(())
    }
}

pub enum Required<'a> {
    Required,
    Optional,
    If(&'a str),
}

pub enum Kind<'a> {
    Value {
        name: Option<&'a str>,
        default: Option<&'a str>,
    },
    Flag,
    Count,
}
