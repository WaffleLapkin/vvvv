use std::{borrow::Cow, cell, collections};

use crate::{error::TooManyOptionsError, SwitchAlreadySetError};

/// Represents a type which can be used as a switch flag
///
/// ## Examples
///
/// ```rust
/// use vvvv::{tr::Switch, SwitchAlreadySetError};
///
/// enum Agree {
///     Yes,
///     No,
/// }
///
/// impl Switch for Agree {
///    fn set(&mut self) -> Result<(), SwitchAlreadySetError> {
///        match self {
///             Self::Yes => Err(SwitchAlreadySetError),
///             Self::No => {
///                 *self = Self::Yes;
///                 Ok(())
///             }
///        }
///     }
///
///     fn is_set(&self) -> bool {
///         matches!(self, Self::Yes)
///     }
/// }
/// ```
pub trait Switch {
    /// Sets flag
    ///
    /// Returns error if flag was already set.
    fn set(&mut self) -> Result<(), SwitchAlreadySetError>;

    /// Check if flag is set
    fn is_set(&self) -> bool;
}

impl Switch for bool {
    #[inline]
    fn set(&mut self) -> Result<(), SwitchAlreadySetError> {
        if *self {
            Err(SwitchAlreadySetError)
        } else {
            *self = true;
            Ok(())
        }
    }

    #[inline]
    fn is_set(&self) -> bool {
        *self
    }
}

/// Represents a type which can be used as a counter flag
///
/// ## Examples
///
/// ```rust
/// use vvvv::{tr::Counter, TooManyOptionsError};
///
/// struct AtMost5(u8);
///
/// impl Counter for AtMost5 {
///     fn inc(&mut self) -> Result<(), TooManyOptionsError> {
///         if self.0 >= 5 {
///             Err(TooManyOptionsError)
///         } else {
///             self.0 += 1;
///             Ok(())
///         }
///     }
/// }
/// ```
pub trait Counter {
    /// Increments counter.
    ///
    /// Returns error on overflow.
    fn inc(&mut self) -> Result<(), TooManyOptionsError>;
}

impl Counter for u8 {
    #[inline]
    fn inc(&mut self) -> Result<(), TooManyOptionsError> {
        *self = self.checked_add(1).ok_or(TooManyOptionsError)?;
        Ok(())
    }
}

impl Counter for u16 {
    #[inline]
    fn inc(&mut self) -> Result<(), TooManyOptionsError> {
        *self = self.checked_add(1).ok_or(TooManyOptionsError)?;
        Ok(())
    }
}

impl Counter for u32 {
    #[inline]
    fn inc(&mut self) -> Result<(), TooManyOptionsError> {
        *self = self.checked_add(1).ok_or(TooManyOptionsError)?;
        Ok(())
    }
}

impl Counter for u64 {
    #[inline]
    fn inc(&mut self) -> Result<(), TooManyOptionsError> {
        *self = self.checked_add(1).ok_or(TooManyOptionsError)?;
        Ok(())
    }
}

impl Counter for usize {
    #[inline]
    fn inc(&mut self) -> Result<(), TooManyOptionsError> {
        *self = self.checked_add(1).ok_or(TooManyOptionsError)?;
        Ok(())
    }
}

/// Counterpart to [`ToOwned`] for types which has borrowed data (i.e.: `Type<'_>`).
///
/// ## Examples
///
/// ```
/// use vvvv::tr::IntoOwned;
///
/// // You can't implement `ToOwned` for Ref<'_, T> as it implements `Clone`
/// #[derive(Debug, Clone)]
/// struct Ref<'a, T>(&'a T);
///
/// impl<T: Clone> IntoOwned for Ref<'_, T> {
///     type Owned = T;
///
///     fn into_owned(self) -> Self::Owned {
///         self.0.clone()
///     }
/// }
/// ```
pub trait IntoOwned {
    type Owned;

    fn into_owned(self) -> Self::Owned;
}

impl<T> IntoOwned for &T 
where
    T: ToOwned,
{
    type Owned = <T as ToOwned>::Owned;

    fn into_owned(self) -> Self::Owned {
        self.to_owned()
    }
}

impl<T> IntoOwned for Cow<'_, T>
where
    T: ToOwned,
{
    type Owned = T::Owned;

    fn into_owned(self) -> <Self as IntoOwned>::Owned {
        self.into_owned()
    }
}

impl<T> IntoOwned for cell::Ref<'_, T>
where   
    T: Clone,
{
    type Owned = T;

    fn into_owned(self) -> Self::Owned {
        self.clone()
    }
}

impl<T> IntoOwned for cell::RefMut<'_, T>
where   
    T: Clone,
{
    type Owned = T;

    fn into_owned(self) -> Self::Owned {
        self.clone()
    }
}

impl<T> IntoOwned for collections::binary_heap::PeekMut<'_, T>
where   
    T: Ord + Clone,
{
    type Owned = T;

    fn into_owned(self) -> Self::Owned {
        self.clone()
    }
}
