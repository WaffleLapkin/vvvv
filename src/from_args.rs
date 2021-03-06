use crate::{FromArgs, Parse, PollInit};

/// Iterator over parse errors and parsed value.
///
/// Created by [`from_args_iter`](crate::from_args_iter).
#[derive(Debug)]
pub struct FromArgsIter<'a, T: FromArgs<'a>, I: Iterator<Item = &'a str>> {
    pub(crate) parser: Parse<'a, I>,
    pub(crate) init: Option<T::Init>,
}

impl<'a, T, I> Iterator for FromArgsIter<'a, T, I>
where
    T: FromArgs<'a>,
    I: Iterator<Item = &'a str>,
{
    type Item = Result<T, crate::Error<'a, <T::Init as PollInit<'a>>::Err>>;

    fn next(&mut self) -> Option<Self::Item> {
        let initializer = self.init.as_mut()?;

        loop {
            match self.parser.next() {
                Some(token) => match initializer.poll_init(token) {
                    Err(err) => return Some(Err(err)),
                    Ok(()) => (),
                },
                None => return Some(self.init.take().unwrap().finish()),
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (1, None)
    }
}
