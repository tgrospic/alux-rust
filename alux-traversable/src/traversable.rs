use crate::effect::{FromResidual, Residual, TryEffect, WithOutput};
use core::ops::ControlFlow;
use extend::ext;

/// Extends optional values with traversal and sequencing operations.
#[ext(name = OptionTraversableExt)]
pub impl<T> Option<T> {
    /// Sequencing operation on [Option] type when inner type is `Applicative` or `Monad`.
    /// See [sequence](OptionTraversableExt::sequence) for traverse with identity closure.
    /// Defined by [Conor McBride](https://doi.org/10.1017/S0956796807006326) (2005) in Haskell2010 base
    /// [Data.Traversable](https://hackage.haskell.org/package/base-4.21.0.0/docs/Data-Traversable.html).
    /// > **Traversable** structures support element-wise **sequencing** of **Applicative** effects
    /// (thus also **Monad** effects) to construct new structures of the **same shape** as the input.
    ///
    /// ```hs
    /// class (Functor t, Foldable t) => Traversable t where
    ///   traverse :: Applicative f => (a -> f b) -> t a -> f (t b)
    /// ```
    /// From this Haskell definition `t` is [Option] and `f` is whichever effect the closure states.
    ///
    /// # Examples
    ///
    /// ```
    /// use alux_traversable::*;
    ///
    /// let r: Result<_, ()> = Some(42).traverse(|x| Ok(x + 100));
    ///
    /// assert_eq!(r, Ok(Some(142)));
    ///
    /// let o = Some(42).traverse(|x| Some(x + 100));
    ///
    /// assert_eq!(o, Some(Some(142)));
    /// ```
    #[inline]
    fn traverse<F, Effect>(self, f: F) -> WithOutput<Effect, Option<Effect::Output>>
    where
        F: FnOnce(T) -> Effect,
        Effect: TryEffect,
        Effect::Residual: Residual<Option<Effect::Output>>,
    {
        // Traverse defined in terms of `sequence`.
        // NOTE: Traversable minimal definition is `traverse` or `sequence` so only one needs to be
        //       implemented and other can be derived.
        //       https://hackage.haskell.org/package/base-4.21.0.0/docs/Data-Traversable.html
        // self.map(f).sequence()

        // Or defined directly by pattern matching.
        match self {
            Some(value) => match f(value).branch() {
                ControlFlow::Continue(output) => TryEffect::from_output(Some(output)),
                ControlFlow::Break(residual) => FromResidual::from_residual(residual),
            },
            None => TryEffect::from_output(None),
        }
    }

    /// Similar to [traverse](OptionTraversableExt::traverse), but with inner value wrapped inside
    /// [Option] so it has effect of filtering None values.
    ///
    /// # Examples
    ///
    /// ```
    /// use alux_traversable::*;
    ///
    /// let r: Result<_, ()> = Some(42).traverse_opt(|x| Ok(Some(x + 100)));
    ///
    /// assert_eq!(r, Ok(Some(142)));
    /// ```
    #[inline]
    fn traverse_opt<F, R, Effect>(self, f: F) -> WithOutput<Effect, Option<R>>
    where
        F: FnOnce(T) -> Effect,
        Effect: TryEffect<Output = Option<R>>,
        Effect::Residual: Residual<Option<R>>,
    {
        // Traverse (opt) defined in terms of `sequence` (opt).
        // NOTE: Traversable minimal definition is `traverse` or `sequence` so only one needs to be
        //       implemented and other can be derived.
        //       https://hackage.haskell.org/package/base-4.21.0.0/docs/Data-Traversable.html
        // self.map(f).sequence_opt()

        // Or defined directly by pattern matching.
        match self {
            Some(value) => match f(value).branch() {
                ControlFlow::Continue(output) => TryEffect::from_output(output),
                ControlFlow::Break(residual) => FromResidual::from_residual(residual),
            },
            None => TryEffect::from_output(None),
        }
    }

    /// An alias for [transpose](Option::transpose), a _correct_ name for this function, and stated
    /// for whichever effect the option carries rather than for [Result] alone. See also
    /// [traverse](OptionTraversableExt::traverse) variant that accepts a mapping closure.
    /// Defined by [Conor McBride](https://doi.org/10.1017/S0956796807006326) (2005) in Haskell2010 base
    /// [Data.Traversable](https://hackage.haskell.org/package/base-4.21.0.0/docs/Data-Traversable.html).
    /// > **Traversable** structures support element-wise **sequencing** of **Applicative** effects
    /// (thus also **Monad** effects) to construct new structures of the **same shape** as the input.
    ///
    /// ```hs
    /// class (Functor t, Foldable t) => Traversable t where
    ///   sequence :: Applicative f => t (f a) -> f (t a)
    /// ```
    /// From this Haskell definition `t` is [Option] and `f` is the effect it carries.
    ///
    /// # Examples
    ///
    /// ```
    /// use alux_traversable::*;
    ///
    /// let r: Result<_, ()> = Some(Ok(42)).sequence();
    ///
    /// assert_eq!(r, Ok(Some(42)));
    ///
    /// let o = Some(Some(42)).sequence();
    ///
    /// assert_eq!(o, Some(Some(42)));
    /// ```
    #[inline]
    fn sequence(self) -> WithOutput<T, Option<T::Output>>
    where
        T: TryEffect,
        T::Residual: Residual<Option<T::Output>>,
    {
        // 1. Sequence defined in terms of `traverse`.
        // NOTE: Traversable minimal definition is `traverse` or `sequence` so only one needs to be
        //       implemented and other can be derived.
        //       https://hackage.haskell.org/package/base-4.21.0.0/docs/Data-Traversable.html
        // self.traverse(identity)

        // 2. Sequence defined as alias for `Option::transpose`, which states `Result` only, so the
        //    same thing is stated here for whichever effect is carried.
        match self {
            Some(effect) => match effect.branch() {
                ControlFlow::Continue(output) => TryEffect::from_output(Some(output)),
                ControlFlow::Break(residual) => FromResidual::from_residual(residual),
            },
            //   ^- Functor, then Applicative `pure` on the way out
            None => TryEffect::from_output(None),
            //      ^- `pure` again, over an option carrying nothing
        }
    }

    /// Similar to [sequence](OptionTraversableExt::sequence), but with inner value wrapped inside
    /// [Option] so it has effect of filtering None values.
    ///
    /// # Examples
    ///
    /// ```
    /// use alux_traversable::*;
    ///
    /// let r: Result<_, ()> = Some(Ok(Some(42))).sequence_opt();
    ///
    /// assert_eq!(r, Ok(Some(42)));
    /// ```
    #[inline]
    fn sequence_opt<R>(self) -> WithOutput<T, Option<R>>
    where
        T: TryEffect<Output = Option<R>>,
        T::Residual: Residual<Option<R>>,
    {
        // Sequence (opt) defined in terms of `traverse` (opt).
        // NOTE: Traversable minimal definition is `traverse` or `sequence` so only one needs to be
        //       implemented and other can be derived.
        //       https://hackage.haskell.org/package/base-4.21.0.0/docs/Data-Traversable.html
        // self.traverse_opt(identity)

        // Or defined directly by pattern matching.
        match self {
            Some(effect) => match effect.branch() {
                ControlFlow::Continue(output) => TryEffect::from_output(output),
                ControlFlow::Break(residual) => FromResidual::from_residual(residual),
            },
            None => TryEffect::from_output(None),
        }
    }
}

/// Extends iterators with traversal and sequencing operations.
#[ext(name = IterTraversableExt)]
pub impl<This> This
where
    This: Iterator,
{
    /// Sequencing operation on [Iterator] type when inner type is `Applicative` or `Monad`.
    /// See [`IterTraversableExt::sequence`] for traverse with identity closure.
    /// Defined by [Conor McBride](https://doi.org/10.1017/S0956796807006326) (2005) in Haskell2010 base
    /// [Data.Traversable](https://hackage.haskell.org/package/base-4.21.0.0/docs/Data-Traversable.html).
    /// > **Traversable** structures support element-wise **sequencing** of **Applicative** effects
    /// (thus also **Monad** effects) to construct new structures of the **same shape** as the input.
    ///
    /// ```hs
    /// class (Functor t, Foldable t) => Traversable t where
    ///   traverse :: Applicative f => (a -> f b) -> t a -> f (t b)
    /// ```
    /// From this Haskell definition `t` is [Iterator] and `f` is the effect that is sequenced.
    ///
    /// # Examples
    ///
    /// ```
    /// use alux_traversable::*;
    ///
    /// let r: Result<_, ()> = [1, 2, 3].into_iter().traverse(|x| Ok(x + x));
    ///
    /// assert_eq!(r, Ok(vec![2, 4, 6]));
    ///
    /// let r: Result<_, ()> = Some(42).into_iter().traverse(|x| Ok(x + x));
    ///
    /// assert_eq!(r, Ok(vec![84]));
    /// ```
    #[inline]
    fn traverse<F, T, Effect>(self, f: F) -> WithOutput<Effect, Vec<Effect::Output>>
    where
        This: Iterator<Item = T>,
        F: FnMut(T) -> Effect,
        Effect: TryEffect,
        Effect::Residual: Residual<Vec<Effect::Output>>,
    {
        // Traverse defined in terms of `sequence`.
        // NOTE: Traversable minimal definition is `traverse` or `sequence` so only one needs to be
        //       implemented and other can be derived.
        //       https://hackage.haskell.org/package/base-4.21.0.0/docs/Data-Traversable.html
        // self.map(f).sequence()

        // Or defined directly, which is what `sequence` here is anyway: the walk stops where the
        // effect stops, and the vector is sized from the iterator's own hint.
        self.map(f).sequence()
    }

    /// Sequencing operation on [Iterator] type when inner type is `Applicative` or `Monad`.
    /// See [`IterTraversableExt::sequence`] for traverse with identity closure.
    /// Defined by [Conor McBride](https://doi.org/10.1017/S0956796807006326) (2005) in Haskell2010 base
    /// [Data.Traversable](https://hackage.haskell.org/package/base-4.21.0.0/docs/Data-Traversable.html).
    /// > **Traversable** structures support element-wise **sequencing** of **Applicative** effects
    /// (thus also **Monad** effects) to construct new structures of the **same shape** as the input.
    ///
    /// ```hs
    /// class (Functor t, Foldable t) => Traversable t where
    ///   traverse :: Applicative f => (a -> f b) -> t a -> f (t b)
    /// ```
    /// From this Haskell definition `t` is [Iterator] and `f` is the effect that is sequenced.
    ///
    /// # Examples
    ///
    /// ```
    /// use alux_traversable::*;
    ///
    /// let r: Result<_, ()> = [1, 2, 3].into_iter().traverse_iter(|x| Ok(Some(x + x)));
    ///
    /// assert_eq!(r, Ok(vec![2, 4, 6]));
    ///
    /// let r: Result<_, ()> = Some(42).into_iter().traverse_iter(|x| Ok(Some(x + x)));
    ///
    /// assert_eq!(r, Ok(vec![84]));
    /// ```
    #[inline]
    fn traverse_opt<F, T, R, Effect>(self, f: F) -> WithOutput<Effect, Vec<R>>
    where
        This: Iterator<Item = T>,
        F: FnMut(T) -> Effect,
        Effect: TryEffect<Output = Option<R>>,
        Effect::Residual: Residual<Vec<R>>,
    {
        // Traverse defined in terms of `sequence`.
        // NOTE: Traversable minimal definition is `traverse` or `sequence` so only one needs to be
        //       implemented and other can be derived.
        //       https://hackage.haskell.org/package/base-4.21.0.0/docs/Data-Traversable.html
        // self.map(f).sequence_opt()

        self.map(f).sequence_opt()
    }

    /// The same as [`IterTraversableExt::traverse_opt`], but accepts more general result
    /// value as `Iterator`.
    ///
    /// NOTE: The effect is general here. The structure walked is still [Iterator] into [Vec], which
    /// is the remaining half of a general Traversable interface (API).
    ///
    /// # Examples
    ///
    /// ```
    /// use alux_traversable::*;
    ///
    /// let r: Result<_, ()> = [1, 2, 3].into_iter().traverse_iter(|x| Ok(Some(x + x)));
    ///
    /// assert_eq!(r, Ok(vec![2, 4, 6]));
    ///
    /// let r: Result<_, ()> = [1, 2, 3].into_iter().traverse_iter(|x| Ok(vec![x, x + x]));
    ///
    /// assert_eq!(r, Ok(vec![1, 2, 2, 4, 3, 6]));
    /// ```
    #[inline]
    fn traverse_iter<F, T, I, R, Effect>(self, f: F) -> WithOutput<Effect, Vec<R>>
    where
        This: Iterator<Item = T>,
        F: FnMut(T) -> Effect,
        Effect: TryEffect<Output = I>,
        Effect::Residual: Residual<Vec<R>>,
        I: IntoIterator<Item = R>,
    {
        // Traverse defined in terms of `sequence`.
        // NOTE: Traversable minimal definition is `traverse` or `sequence` so only one needs to be
        //       implemented and other can be derived.
        //       https://hackage.haskell.org/package/base-4.21.0.0/docs/Data-Traversable.html
        // self.map(f).sequence_iter()

        self.map(f).sequence_iter()
    }

    /// Sequencing operation on [Iterator] type when inner type is `Applicative` or `Monad`.
    /// See [`IterTraversableExt::sequence`] for traverse with identity closure.
    /// Defined by [Conor McBride](https://doi.org/10.1017/S0956796807006326) (2005) in Haskell2010 base
    /// [Data.Traversable](https://hackage.haskell.org/package/base-4.21.0.0/docs/Data-Traversable.html).
    /// > **Traversable** structures support element-wise **sequencing** of **Applicative** effects
    /// (thus also **Monad** effects) to construct new structures of the **same shape** as the input.
    ///
    /// ```hs
    /// class (Functor t, Foldable t) => Traversable t where
    ///   traverse :: Applicative f => (a -> f b) -> t a -> f (t b)
    /// ```
    /// From this Haskell definition `t` is [Iterator] and `f` is the effect that is sequenced.
    ///
    /// # Examples
    ///
    /// ```
    /// use alux_traversable::*;
    ///
    /// let r: Result<_, ()> = [Ok(1), Ok(2), Ok(3)].into_iter().sequence();
    ///
    /// assert_eq!(r, Ok(vec![1, 2, 3]));
    /// ```
    #[inline]
    fn sequence<Effect>(self) -> WithOutput<Effect, Vec<Effect::Output>>
    where
        This: Iterator<Item = Effect>,
        Effect: TryEffect,
        Effect::Residual: Residual<Vec<Effect::Output>>,
    {
        // Sequence defined in terms of `traverse`.
        // NOTE: Traversable minimal definition is `traverse` or `sequence` so only one needs to be
        //       implemented and other can be derived.
        //       https://hackage.haskell.org/package/base-4.21.0.0/docs/Data-Traversable.html
        // self.traverse(identity)

        let mut collected = Vec::with_capacity(self.size_hint().0);
        for effect in self {
            match effect.branch() {
                ControlFlow::Continue(output) => collected.push(output),
                ControlFlow::Break(residual) => return FromResidual::from_residual(residual),
            }
        }

        TryEffect::from_output(collected)
    }

    /// Sequencing operation on [Iterator] type when inner type is `Applicative` or `Monad`.
    /// See [`IterTraversableExt::sequence`] for traverse with identity closure.
    /// Defined by [Conor McBride](https://doi.org/10.1017/S0956796807006326) (2005) in Haskell2010 base
    /// [Data.Traversable](https://hackage.haskell.org/package/base-4.21.0.0/docs/Data-Traversable.html).
    /// > **Traversable** structures support element-wise **sequencing** of **Applicative** effects
    /// (thus also **Monad** effects) to construct new structures of the **same shape** as the input.
    ///
    /// ```hs
    /// class (Functor t, Foldable t) => Traversable t where
    ///   traverse :: Applicative f => (a -> f b) -> t a -> f (t b)
    /// ```
    /// From this Haskell definition `t` is [Iterator] and `f` is the effect that is sequenced.
    ///
    /// # Examples
    ///
    /// ```
    /// use alux_traversable::*;
    ///
    /// let r: Result<_, ()> = [Ok(Some(1)), Ok(Some(2)), Ok(None), Ok(Some(3))].into_iter().sequence_opt();
    ///
    /// assert_eq!(r, Ok(vec![1, 2, 3]));
    /// ```
    #[inline]
    fn sequence_opt<R, Effect>(self) -> WithOutput<Effect, Vec<R>>
    where
        This: Iterator<Item = Effect>,
        Effect: TryEffect<Output = Option<R>>,
        Effect::Residual: Residual<Vec<R>>,
    {
        // Sequence (opt) defined in terms of `traverse` (opt).
        // NOTE: Traversable minimal definition is `traverse` or `sequence` so only one needs to be
        //       implemented and other can be derived.
        //       https://hackage.haskell.org/package/base-4.21.0.0/docs/Data-Traversable.html
        // self.traverse_opt(identity)

        let mut collected = Vec::new();
        for effect in self {
            match effect.branch() {
                ControlFlow::Continue(output) => collected.extend(output),
                ControlFlow::Break(residual) => return FromResidual::from_residual(residual),
            }
        }

        TryEffect::from_output(collected)
    }

    /// The same as [`IterTraversableExt::sequence_opt`], but accepts more general result
    /// value as `Iterator`.
    ///
    /// NOTE: The effect is general here. The structure walked is still [Iterator] into [Vec], which
    /// is the remaining half of a general Traversable interface (API).
    ///
    /// # Examples
    ///
    /// ```
    /// use alux_traversable::*;
    ///
    /// let r: Result<_, ()> = [Ok(Some(1)), Ok(Some(2)), Ok(None), Ok(Some(3))].into_iter().sequence_iter();
    ///
    /// assert_eq!(r, Ok(vec![1, 2, 3]));
    ///
    /// let r: Result<_, ()> = [Ok(vec![1, 2]), Ok(vec![]), Ok(vec![3])].into_iter().sequence_iter();
    ///
    /// assert_eq!(r, Ok(vec![1, 2, 3]));
    /// ```
    #[inline]
    fn sequence_iter<R, I, Effect>(self) -> WithOutput<Effect, Vec<R>>
    where
        This: Iterator<Item = Effect>,
        Effect: TryEffect<Output = I>,
        Effect::Residual: Residual<Vec<R>>,
        I: IntoIterator<Item = R>,
    {
        // Sequence defined in terms of `traverse`.
        // NOTE: Traversable minimal definition is `traverse` or `sequence` so only one needs to be
        //       implemented and other can be derived.
        //       https://hackage.haskell.org/package/base-4.21.0.0/docs/Data-Traversable.html
        // self.traverse_iter(identity)

        // Or defined directly by pattern matching.
        let mut collected = Vec::new();
        for effect in self {
            match effect.branch() {
                ControlFlow::Continue(output) => collected.extend(output),
                ControlFlow::Break(residual) => return FromResidual::from_residual(residual),
            }
        }

        TryEffect::from_output(collected)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_traverse() {
        // Result Ok

        for input in [Some(42), None] {
            let res: Result<_, ()> = input.traverse(Ok);

            assert_eq!(res, Ok(input));
        }

        let res: Result<Option<i32>, ()> = None.traverse(|()| Err(()));

        assert_eq!(res, Ok(None));

        // Result Err

        let res: Result<Option<i32>, ()> = Some(42).traverse(|_| Err(()));

        assert_eq!(res, Err(()));
    }

    #[test]
    fn test_traverse_opt() {
        // Result Ok

        let res: Result<_, ()> = Some(42).traverse_opt(|x| Ok(Some(x + x)));

        assert_eq!(res, Ok(Some(84)));

        let res: Result<_, ()> = Option::<i32>::None.traverse_opt(|x| Ok(Some(x + x)));

        assert_eq!(res, Ok(None));

        let res: Result<Option<i32>, ()> = Some(42).traverse_opt(|_| Ok(None));

        assert_eq!(res, Ok(None));

        let res: Result<Option<i32>, ()> = None.traverse_opt(|_: u32| Err(()));

        assert_eq!(res, Ok(None));

        // Result Err

        let res: Result<Option<i32>, ()> = Some(42).traverse_opt(|_| Err(()));

        assert_eq!(res, Err(()));
    }

    #[test]
    fn test_traverse_iter() {
        // Result Ok
        let input_vec = [vec![1, 2], vec![], vec![3]];
        let input_opt = [Some(1), Some(2), None, Some(3)];

        let res_vec: Result<Vec<i32>, ()> =
            input_vec.clone().into_iter().traverse_iter(|xs| Ok(xs.into_iter().map(|x| x + x)));

        let res_opt: Result<Vec<i32>, ()> = input_opt.into_iter().traverse_iter(|_| Ok(vec![].into_iter()));

        assert_eq!(res_vec, Ok(vec![2, 4, 6]));
        assert_eq!(res_opt, Ok(vec![]));

        // Simplest error
        let err = Result::<Vec<i32>, ()>::Err(());

        // Traverse empty
        let res_vec: Result<Vec<i32>, ()> = [].into_iter().traverse_iter(|_: i32| err.clone());
        let res_opt: Result<Vec<i32>, ()> = None.into_iter().traverse_iter(|_: i32| err.clone());

        assert_eq!(res_vec, Ok(vec![]));
        assert_eq!(res_opt, Ok(vec![]));

        // Result Err

        let res_vec: Result<Vec<i32>, ()> = [1].into_iter().traverse_iter(|_| err.clone());
        let res_opt: Result<Vec<i32>, ()> = Some(1).into_iter().traverse_iter(|_| err.clone());

        assert_eq!(res_vec, Err(()));
        assert_eq!(res_opt, Err(()));
    }

    #[test]
    fn test_sequence() {
        for (input, expected) in [
            // Result Ok
            (Some(Ok(42)), Ok(Some(42))),
            (None, Ok(None)),
            // Result Err
            (Some(Err(())), Err(())),
        ] {
            let res = input.sequence();

            assert_eq!(res, expected);
        }
    }

    #[test]
    fn test_sequence_opt() {
        for (input, expected) in [
            // Result Ok
            (Some(Ok(Some(42))), Ok(Some(42))),
            (Some(Ok(None)), Ok(None)),
            (None, Ok(None)),
            // Result Err
            (Some(Err(())), Err(())),
        ] {
            let res = input.sequence_opt();

            assert_eq!(res, expected);
        }
    }

    #[test]
    fn test_sequence_iter() {
        for (input_vec, input_opt, expected) in [
            // Result Ok
            (
                // Input Vec
                vec![Ok(vec![1, 2]), Ok(vec![]), Ok(vec![3])],
                // Input Option
                vec![Ok(Some(1)), Ok(Some(2)), Ok(None), Ok(Some(3))],
                // Expected result
                Ok(vec![1, 2, 3]),
            ),
            (vec![Ok(vec![])], vec![Ok(None)], Ok(vec![])),
            // Result Err
            (vec![Err(())], vec![Err(())], Err(())),
        ] {
            let res_vec = input_vec.into_iter().sequence_iter();
            let res_opt = input_opt.into_iter().sequence_iter();

            assert_eq!(res_vec, expected);
            assert_eq!(res_opt, expected);
        }
    }
}

#[cfg(test)]
mod effect_instance_tests {
    use super::*;

    #[test]
    fn an_option_effect_sequences_exactly_as_a_result_does() {
        assert_eq!(Some(42).traverse(|x| Some(x + x)), Some(Some(84)));
        assert_eq!(Some(42).traverse(|_: i32| Option::<i32>::None), None);
        assert_eq!(Option::<i32>::None.traverse(|_| Option::<i32>::None), Some(None));

        assert_eq!(Some(Some(42)).sequence(), Some(Some(42)));
        assert_eq!(Some(Option::<i32>::None).sequence(), None);
        assert_eq!(Option::<Option<i32>>::None.sequence(), Some(None));

        assert_eq!([1, 2, 3].into_iter().traverse(|x| Some(x + x)), Some(vec![2, 4, 6]));
        assert_eq!([1, 2, 3].into_iter().traverse(|_: i32| Option::<i32>::None), None);
        assert_eq!([Some(1), Some(2)].into_iter().sequence(), Some(vec![1, 2]));
        assert_eq!([Some(1), None].into_iter().sequence(), None);
    }

    #[test]
    fn an_option_effect_filters_and_flattens_the_same_way() {
        assert_eq!(Some(42).traverse_opt(|x| Some(Some(x + x))), Some(Some(84)));
        assert_eq!(Some(42).traverse_opt(|_: i32| Some(Option::<i32>::None)), Some(None));
        assert_eq!(Some(Some(Some(42))).sequence_opt(), Some(Some(42)));

        let filtered = [Some(1), None, Some(3)].into_iter().traverse_opt(Some);
        assert_eq!(filtered, Some(vec![1, 3]));

        let flattened = [1, 2].into_iter().traverse_iter(|x| Some(vec![x, x + x]));
        assert_eq!(flattened, Some(vec![1, 2, 2, 4]));

        let stopped = [Some(vec![1]), None].into_iter().sequence_iter();
        assert_eq!(stopped, None);
    }

    #[test]
    fn a_walk_stops_where_the_effect_stops() {
        let mut seen = 0;
        let stopped: Result<Vec<i32>, ()> = [1, 2, 3].into_iter().traverse(|value| {
            seen += 1;
            if value == 2 { Err(()) } else { Ok(value) }
        });

        assert_eq!(stopped, Err(()));
        assert_eq!(seen, 2, "the third element is never handed to the closure");
    }
}

#[cfg(test)]
mod iterator_instance_tests {
    use super::IterTraversableExt;

    #[test]
    fn iterator_traverse_preserves_shape_and_error() {
        let success: Result<Vec<_>, ()> = [1, 2, 3].into_iter().traverse(|x| Ok(x + x));
        assert_eq!(success, Ok(vec![2, 4, 6]));

        let empty: Result<Vec<i32>, ()> = [].into_iter().traverse(|x: i32| Ok(x));
        assert_eq!(empty, Ok(vec![]));

        let failure: Result<Vec<i32>, ()> = [1].into_iter().traverse(|_| Err(()));
        assert_eq!(failure, Err(()));

        let mut total = 0;
        let stateful: Result<Vec<_>, ()> = [1, 2, 3].into_iter().traverse(|value| {
            total += value;
            Ok(total)
        });
        assert_eq!(stateful, Ok(vec![1, 3, 6]));
    }

    #[test]
    fn iterator_traverse_opt_filters_none_and_preserves_error() {
        let success: Result<Vec<_>, ()> = [Some(1), None, Some(3)].into_iter().traverse_opt(Ok);
        assert_eq!(success, Ok(vec![1, 3]));

        let failure: Result<Vec<i32>, ()> = [1].into_iter().traverse_opt(|_| Err(()));
        assert_eq!(failure, Err(()));
    }

    #[test]
    fn iterator_sequence_preserves_shape_and_error() {
        let success = [Ok(1), Ok(2), Ok(3)].into_iter().sequence();
        assert_eq!(success, Ok::<_, ()>(vec![1, 2, 3]));

        let failure = [Ok(1), Err(()), Ok(3)].into_iter().sequence();
        assert_eq!(failure, Err(()));
    }

    #[test]
    fn iterator_sequence_opt_filters_none_and_preserves_error() {
        let success = [Ok(Some(1)), Ok(None), Ok(Some(3))].into_iter().sequence_opt();
        assert_eq!(success, Ok::<_, ()>(vec![1, 3]));

        let failure = [Ok(Some(1)), Err(())].into_iter().sequence_opt();
        assert_eq!(failure, Err(()));
    }
}
