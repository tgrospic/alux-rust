//! The effect a traversal sequences, stated as the standard library states it.
//!
//! A traversal walks a structure and sequences an effect out of it. What the effect *is* has one
//! shape: a value that either carries an output onward or stops the walk with a residual. That is
//! what `Result` and `Option` have in common, and the only thing a traversal needs of either.
//!
//! `std::ops::Try` and `std::ops::Residual` state exactly this, and are unstable. So they are
//! stated here, with the same associated names and the same meaning, and the traversals are
//! written against these. When the standard traits stabilize, these are what they replace: the
//! call sites do not mention either trait, so nothing a caller writes changes.
//!
//! Both are sealed, because `std::ops::Try` cannot be implemented outside the standard library.
//! Sealing them states the same limit now, so no impl written today is stranded by the swap.

use core::convert::Infallible;
use core::ops::ControlFlow;

/// States a value that either carries an output onward or stops with a residual.
///
/// Mirrors [`std::ops::Try`], declaration for declaration, so that replacing this with the
/// standard trait is a `use` and nothing else.
pub trait TryEffect: FromResidual<Self::Residual> + sealed::Sealed {
    /// The value this effect carries when it carries one.
    type Output;

    /// What is left when the walk stops: the effect itself, with no output in it.
    type Residual;

    /// States this effect carrying one output.
    fn from_output(output: Self::Output) -> Self;

    /// States whether the walk carries on with an output, or stops with a residual.
    fn branch(self) -> ControlFlow<Self::Residual, Self::Output>;
}

/// States this effect stopped, from what was left when it stopped.
///
/// Mirrors [`std::ops::FromResidual`]. A traversal only ever rebuilds the very effect it took
/// apart, so the only impls are the ones taking an effect's own residual.
pub trait FromResidual<Residual = <Self as TryEffect>::Residual>: sealed::Sealed {
    /// States this effect stopped, carrying what the walk stopped with.
    fn from_residual(residual: Residual) -> Self;
}

/// States which effect a residual belongs to, once an output type is chosen.
///
/// Mirrors [`std::ops::Residual`]. This is what lets a traversal state the effect it was handed
/// while carrying something else: the same `Result` error or the same `Option` absence, over a
/// `Vec` where the walk was over one element.
pub trait Residual<Output>: sealed::Sealed {
    /// The effect this residual belongs to, carrying `Output`.
    type TryType: TryEffect<Output = Output, Residual = Self>;
}

/// The effect `Effect` is, carrying `Output` instead of what it carried.
///
/// Mirrors `ChangeOutputType` in the standard library, which is unexported. A traversal states
/// this as its result: sequencing `Result<T, E>` out of a `Vec` states `Result<Vec<T>, E>`, and
/// the error is the one the walk was already carrying.
pub type WithOutput<Effect, Output> = <<Effect as TryEffect>::Residual as Residual<Output>>::TryType;

impl<T, E> TryEffect for Result<T, E> {
    type Output = T;
    type Residual = Result<Infallible, E>;

    fn from_output(output: Self::Output) -> Self {
        Ok(output)
    }

    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        match self {
            Ok(output) => ControlFlow::Continue(output),
            Err(error) => ControlFlow::Break(Err(error)),
        }
    }
}

impl<T, E> FromResidual<Result<Infallible, E>> for Result<T, E> {
    fn from_residual(residual: Result<Infallible, E>) -> Self {
        match residual {
            Err(error) => Err(error),
            Ok(infallible) => match infallible {},
        }
    }
}

impl<T, E> Residual<T> for Result<Infallible, E> {
    type TryType = Result<T, E>;
}

impl<T> TryEffect for Option<T> {
    type Output = T;
    type Residual = Option<Infallible>;

    fn from_output(output: Self::Output) -> Self {
        Some(output)
    }

    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        match self {
            Some(output) => ControlFlow::Continue(output),
            None => ControlFlow::Break(None),
        }
    }
}

impl<T> FromResidual<Option<Infallible>> for Option<T> {
    fn from_residual(residual: Option<Infallible>) -> Self {
        match residual {
            None => None,
            Some(infallible) => match infallible {},
        }
    }
}

impl<T> Residual<T> for Option<Infallible> {
    type TryType = Option<T>;
}

mod sealed {
    /// Seals the effect traits, so what states an effect is what the standard library will state.
    pub trait Sealed {}

    impl<T, E> Sealed for Result<T, E> {}

    impl<T> Sealed for Option<T> {}
}
