//! This module provides **very** dumb implementation of text word wraspping.
//!
//! I wanted to use `textwrap` crate, but it requires the whole string up-front which I don't have.
//! So, I've written _this_. The code seems to work, but it doesn't do anything with lots of egde cases,
//! this may result in weird results.
//!
//! One day I'll refactor this (and maybe even make a crate with text wrapping).

// FIXME(waffle): refactor _this_

use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

#[derive(Debug, PartialEq)]
pub(crate) enum Item<'a> {
    Part(&'a str),
    Break(&'a str),
}

pub(crate) struct Wrap<'a, I> {
    max: usize,
    spent: usize,
    iter: I,
    curr: &'a str,
}

impl<'a, I> Wrap<'a, I>
where
    I: Iterator<Item = &'a str>,
{
    pub(crate) fn new(max: usize, mut iter: I) -> Self {
        Self {
            max,
            spent: 0,
            curr: iter.next().expect(""),
            iter,
        }
    }
}

impl<'a, I> Iterator for Wrap<'a, I>
where
    I: Iterator<Item = &'a str>,
{
    type Item = Item<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        // FIXME: this whole method is poorly written

        if self.curr == "" {
            return None;
        }
        let (l, r) = split_w(self.curr, self.max - self.spent);

        // FIXME(waffle): return width from `split_w`
        self.spent += l.width();
        if self.max > self.spent {
            if r == "" {
                self.curr = self.iter.next().unwrap_or("");
                Some(Item::Part(l))
            } else {
                self.curr = r;
                self.spent = 0;
                Some(Item::Break(l))
            }
        } else {
            if r == "" {
                self.curr = self.iter.next().unwrap_or("");
            } else {
                self.curr = r;
            }

            self.spent = 0;
            Some(Item::Break(l))
        }
    }
}

fn split_w(s: &str, at_w: usize) -> (&str, &str) {
    let (soft, hard) = s
        .char_indices()
        .scan((0, None), |(width, last_space), (i, ch)| {
            *width += ch.width().unwrap_or(0);

            let w = if ch == ' ' {
                *last_space = Some(i);

                // HACK(waffle): this is needed so `split_w("A B C", 3)` will result in ("A B", "C")
                *width - 1
            } else {
                *width
            };

            if w <= at_w {
                Some((*last_space, i + ch.len_utf8()))
            } else {
                None
            }
        })
        .last()
        .unwrap_or((None, 0)); // inpur string is empty anyway

    match soft {
        None => s.split_at(hard),
        Some(_) if hard == s.len() => (s, ""),
        Some(soft) => {
            let (l, r) = s.split_at(soft);
            // remove space
            (l, &r[1..])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{split_w, Item, Wrap};

    fn test_split_w(s: &str, cases: &[(usize, (&str, &str))]) {
        for &(at, res) in cases {
            assert_eq!(split_w(s, at), res);
        }
    }

    // kate.
    #[test]
    fn normal_split() {
        let s: &str = "DO NOT BECOME ADDICTED TO OXYGEN";
        //             ^\  ^\ ^\   ^\    ^\    ^\
        //              0   4  7    12    18    24
        test_split_w(
            s,
            &[
                (0, ("", "DO NOT BECOME ADDICTED TO OXYGEN")),
                (1, ("D", "O NOT BECOME ADDICTED TO OXYGEN")),
                (4, ("DO", "NOT BECOME ADDICTED TO OXYGEN")),
                (6, ("DO NOT", "BECOME ADDICTED TO OXYGEN")),
                (7, ("DO NOT", "BECOME ADDICTED TO OXYGEN")),
                (12, ("DO NOT", "BECOME ADDICTED TO OXYGEN")),
                (13, ("DO NOT BECOME", "ADDICTED TO OXYGEN")),
                (18, ("DO NOT BECOME", "ADDICTED TO OXYGEN")),
                (24, ("DO NOT BECOME ADDICTED", "TO OXYGEN")),
                (19038949, ("DO NOT BECOME ADDICTED TO OXYGEN", "")),
            ],
        );
    }

    // multibyte chars
    #[test]
    fn aaaaaa() {
        let s: &str = "áá ááá áááááá áááááááá áá áááááá";
        //             ^\  ^\ ^\   ^\    ^\    ^\
        //              0   4  7    12    18    24
        test_split_w(
            s,
            &[
                (0, ("", "áá ááá áááááá áááááááá áá áááááá")),
                (1, ("á", "á ááá áááááá áááááááá áá áááááá")),
                (4, ("áá", "ááá áááááá áááááááá áá áááááá")),
                (6, ("áá ááá", "áááááá áááááááá áá áááááá")),
                (7, ("áá ááá", "áááááá áááááááá áá áááááá")),
                (12, ("áá ááá", "áááááá áááááááá áá áááááá")),
                (13, ("áá ááá áááááá", "áááááááá áá áááááá")),
                (18, ("áá ááá áááááá", "áááááááá áá áááááá")),
                (24, ("áá ááá áááááá áááááááá", "áá áááááá")),
                (19038949, ("áá ááá áááááá áááááááá áá áááááá", "")),
            ],
        );
    }

    #[test]
    fn basic_wrap() {
        let vec = vec!["DO NOT BECOME", " ", "ADDICTED TO OXYGEN"];
        let wrap = Wrap::new(12, vec.iter().copied());
        assert!(wrap.eq(vec![
            Item::Break("DO NOT"),
            Item::Part("BECOME"),
            Item::Part(" "),
            Item::Break("ADDIC"),
            Item::Break("TED TO"),
            Item::Part("OXYGEN"),
        ]))
    }
}
