use crate::{FromArgs, PollInit};

/// Counterpart to [`FromArgs`] which does not borrow from args.
pub trait FromArgsOwned: Sized
where
    for<'a> Self: FromArgs<'a, Init = <Self as FromArgsOwned>::OwnInit>,
{
    type OwnInit: PollInitOwned<OwnOutput = Self>;
}

impl<T, I> FromArgsOwned for T
where
    for<'a> T: FromArgs<'a, Init = I>,
    I: PollInitOwned<OwnOutput = T>,
{
    type OwnInit = I;
}

/// Counterpart to [`PollInit`] which produces owned values.
pub trait PollInitOwned
where
    for<'a> Self: PollInit<
        'a,
        Output = <Self as PollInitOwned>::OwnOutput,
        Err = <Self as PollInitOwned>::OwnErr,
    >,
{
    type OwnOutput;
    type OwnErr;
}

impl<T, O, E> PollInitOwned for T
where
    for<'a> T: PollInit<'a, Output = O, Err = E>,
{
    type OwnOutput = O;

    type OwnErr = E;
}
