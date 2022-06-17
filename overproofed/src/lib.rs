#![allow(unused)]
// #![no_std]

// makes bigger binary a tiny bit
// use smallvec::SmallVec;
// pub type Vec<T> = SmallVec<[T; 0]>;

use core::cmp::PartialEq;
use core::marker::PhantomData;
use core::panic::Location;

pub fn default<T: Default>() -> T {
    Default::default()
}

pub fn is_zero<T: Default + PartialEq>(t: T) -> bool {
    t == T::default()
}

pub fn zero<T: Default>() -> T {
    Default::default()
}

macro_rules! derefs {
    ($type:ident $(<$($T:ident),*>)?  => $attr:ident: $target:ty) => {
        impl $(<$($T),*>)? core::ops::Deref for $type $(<$($T),*>)?  {
            type Target = $target;
            fn deref(&self) -> &Self::Target {
                &self.$attr
            }
        }
        impl $(<$($T),*>)? core::ops::DerefMut for $type $(<$($T),*>)?  {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.$attr
            }
        }
    };
}

pub use rules::{Amounts, InMix, Item, Mix, Recipe, WithMixes};

pub type Index = u16;
pub type Value = f32;
// pub struct Kilograms(Value);
// pub struct Percent(Value);

const UNSOLVED: f32 = f32::NAN;
const OVERFLOW: Index = Index::MAX;

pub const OVERFLOW_INDEX: Index = OVERFLOW;

fn is_unsolved(v: f32) -> bool {
    v.is_nan()
}

fn extend_unsolved<const N: usize>(values: &mut Vec<f32>) -> Option<[Index; N]> {
    let mut indexes = [0; N];

    if values.len().checked_add(N)? > values.capacity() {
        return None;
    }

    indexes
        .iter_mut()
        .zip(values.len().try_into().ok()?..)
        .for_each(|(p, index)| *p = index);

    values.extend([UNSOLVED; N]);

    Some(indexes)
}

#[derive(Debug, Clone)]
pub struct Values {
    buf: Vec<f32>,
    did_overflow: Option<usize>,
    overflow_value: f32,
}

impl From<Vec<f32>> for Values {
    fn from(buf: Vec<f32>) -> Values {
        Values { buf, did_overflow: None, overflow_value: f32::INFINITY }
    }
}

impl Values {
    pub fn value_index_or_overflow(&mut self) -> Index {
        let [i] = self.value_indexes_or_overflow();
        i
    }

    pub fn value_indexes_or_overflow<const N: usize>(&mut self) -> [Index; N] {
        if let Some(v) = extend_unsolved(&mut self.buf) {
            v
        } else {
            self.did_overflow.get_or_insert(self.buf.len());
            [OVERFLOW; N]
        }
    }

    /// can be determined if we ran out of room to allocate more values
    ///
    /// notably, overflow is not set if an invalid index was used merely to retrieve a value
    pub fn did_overflow(&self) -> bool {
        self.did_overflow.is_some()
    }

    pub fn how_overflow(&self) -> Option<usize> {
        self.did_overflow.clone()
    }
}

impl Values {
    pub fn value_opt(&self, i: Index) -> Option<Value> {
        self.buf.get(i as usize).cloned()
    }

    pub fn value(&self, i: Index) -> Value {
        if let Some(v) = self.buf.get(i as usize) {
            *v
        } else {
            self.overflow_value
        }
    }

    pub fn value_mut(&mut self, i: Index) -> &mut Value {
        if let Some(v) = self.buf.get_mut(i as usize) {
            v
        } else {
            self.did_overflow.get_or_insert(i as usize);
            &mut self.overflow_value
        }
    }

    pub fn minimal_recipe(&mut self) -> rules::Recipe {
        rules::Recipe {
            dough: rules::Mix {
                total: self.new_item().into(),
                flour: self.new_item().into(),
                nonflour: self.new_item().into(),
                flours: default(),
                nonflours: default(),
            },
            mixes: default(),
        }
    }

    pub fn minimal_mix(&mut self) -> rules::Mix {
        rules::Mix {
            total: self.new_mix_item().into(),
            flour: self.new_mix_item().into(),
            nonflour: self.new_mix_item().into(),
            flours: default(),
            nonflours: default(),
        }
    }

    pub fn new_item(&mut self) -> rules::WithMixes {
        let [weight, bakers, weight_in_mixes, weight_less_mixes, percent_in_mixes, percent_less_mixes] =
            self.value_indexes_or_overflow();
        let amounts = rules::Amounts { weight, bakers };
        rules::WithMixes {
            amounts,
            weight_in_mixes,
            weight_less_mixes,
            percent_in_mixes,
            percent_less_mixes,
        }
    }

    pub fn new_mix_item(&mut self) -> rules::InMix {
        let [weight, bakers, percent_of_total] = self.value_indexes_or_overflow();
        let amounts = rules::Amounts { weight, bakers };
        rules::InMix { amounts, percent_of_total }
    }
}

mod rules {
    use super::{Index, ToWhence, Value, Values, Whence};

    use core::iter::once;

    /* Each item used in a table has:
     * - weight
     * - baker's percent
     *
     * In the total table, items also have:
     * - weight of that item that comes from mixes
     * - percentage of that item that comes from mixes (weight from mixes over total weight)
     * - weight and percent of that item added to the final dough (total minus amount in mixes)
     *
     * In mix tables, items only have one extra field:
     * - percent of the ingredient's weight in the mix out of its total weight in the recipe
     *   (item's weight in this mix / item weight in totals table)
     */

    #[derive(Debug)]
    pub struct Recipe {
        pub dough: Mix,
        pub mixes: Vec<Mix>,
    }

    #[derive(Debug, Clone)]
    pub struct Mix {
        pub total: Item,
        pub flour: Item,
        pub nonflour: Item,
        pub flours: Vec<Option<Item>>,
        pub nonflours: Vec<Option<Item>>,
    }

    #[derive(Debug, Clone)]
    pub enum Item {
        WithMixes(WithMixes),
        InMix(InMix),
    }

    #[derive(Debug, Clone)]
    pub struct WithMixes {
        pub amounts: Amounts,
        // sum of weights of this item in mixes
        pub weight_in_mixes: Index,
        // weight - sum of weights of this item in mixes (aka final)
        pub weight_less_mixes: Index,
        /// percentage of this item's total weight from mixes
        pub percent_in_mixes: Index,
        /// percentage of this item's total weight not from mixes
        pub percent_less_mixes: Index,
    }

    #[derive(Debug, Clone)]
    pub struct InMix {
        pub amounts: Amounts,
        pub percent_of_total: Index,
    }

    #[derive(Debug, Clone)]
    pub struct Amounts {
        pub weight: Index,
        pub bakers: Index,
    }

    derefs!(WithMixes => amounts: Amounts);

    derefs!(InMix => amounts: Amounts);

    impl Item {
        pub fn with_mixes(&self) -> Option<&WithMixes> {
            match self {
                Item::WithMixes(v) => Some(v),
                _ => None,
            }
        }

        pub fn in_mix(&self) -> Option<&InMix> {
            match self {
                Item::InMix(v) => Some(v),
                _ => None,
            }
        }
    }

    impl core::ops::Deref for Item {
        type Target = Amounts;

        fn deref(&self) -> &Self::Target {
            match self {
                Self::WithMixes(v) => v.deref(),
                Self::InMix(v) => v.deref(),
            }
        }
    }

    impl From<WithMixes> for Item {
        fn from(v: WithMixes) -> Self {
            Self::WithMixes(v)
        }
    }

    impl From<InMix> for Item {
        fn from(v: InMix) -> Self {
            Self::InMix(v)
        }
    }

    pub(crate) fn for_recipe_fallback(recipe: &Recipe) -> impl Iterator<Item = Whence<Math>> + '_ {
        let Recipe { dough, mixes } = recipe;

        once(dough.flour.bakers)
            .chain(mixes.iter().map(|mix| mix.flour.bakers))
            .map(|index| Math::TotalFlourBakers100 { index }.to_whence())
    }

    pub(crate) fn for_recipe(recipe: &Recipe) -> impl Iterator<Item = Whence<Math>> + '_ {
        let Recipe { dough, mixes } = recipe;

        return once(dough).chain(mixes).flat_map(for_mix).chain(
            /* iterate each row/item in total and where it's mixed */
            [Key::Total, Key::Flour, Key::NonFlour]
                .into_iter()
                .chain((0..dough.flours.len()).map(Key::Flours))
                .chain((0..dough.nonflours.len()).map(Key::NonFlours))
                .filter_map(|item_key| {
                    Some((
                        dough.get(item_key)?.with_mixes()?,
                        mixes
                            .iter()
                            .filter_map(move |mix| mix.get(item_key)?.in_mix()),
                    ))
                })
                .flat_map(|(t, mixed)| {
                    [
                        mixed
                            .clone()
                            .map(|i| i.weight)
                            .sums_to(t.weight_in_mixes)
                            .to_whence(),
                        mixed
                            .clone()
                            .map(|i| i.percent_of_total)
                            .sums_to(t.percent_in_mixes)
                            .to_whence(),
                    ]
                    .into_iter()
                    .chain(mixed.map(|i| {
                        Math::PercentOf {
                            product: i.weight,
                            pct: i.percent_of_total,
                            of: t.weight,
                        }
                        .to_whence()
                    }))
                }),
        );

        #[derive(Debug, Copy, Clone)]
        enum Key {
            Total,
            Flour,
            NonFlour,
            Flours(usize),
            NonFlours(usize),
        }

        impl Mix {
            fn get(&self, key: Key) -> Option<&Item> {
                use Key::*;
                match key {
                    Total => Some(&self.total),
                    Flour => Some(&self.flour),
                    NonFlour => Some(&self.nonflour),
                    Flours(i) => self.flours.get(i)?.as_ref(),
                    NonFlours(i) => self.nonflours.get(i)?.as_ref(),
                }
            }
        }
    }

    fn for_mix(mix: &Mix) -> impl Iterator<Item = Whence<Math>> + '_ {
        let Mix { total, flour, nonflour, flours, nonflours } = mix;

        [
            /* sum weights */
            [flour.weight, nonflour.weight]
                .sums_to(total.weight)
                .to_whence(),
            flours
                .iter()
                .flatten()
                .map(core::ops::Deref::deref)
                .map(|&Amounts { weight, .. }| weight)
                .sums_to(flour.weight)
                .to_whence(),
            nonflours
                .iter()
                .flatten()
                .map(core::ops::Deref::deref)
                .map(|&Amounts { weight, .. }| weight)
                .sums_to(nonflour.weight)
                .to_whence(),
            /* sum bakers percentages */
            [flour.bakers, nonflour.bakers]
                .sums_to(total.bakers)
                .to_whence(),
            flours
                .iter()
                .flatten()
                .map(core::ops::Deref::deref)
                .map(|&Amounts { bakers, .. }| bakers)
                .sums_to(flour.bakers)
                .to_whence(),
            nonflours
                .iter()
                .flatten()
                .map(core::ops::Deref::deref)
                .map(|&Amounts { bakers, .. }| bakers)
                .sums_to(nonflour.bakers)
                .to_whence(),
        ]
        .into_iter()
        .chain(
            /* bakers percentages as expression of total flour weight */
            [total, flour, nonflour]
                .into_iter()
                .chain(flours.iter().flatten())
                .chain(nonflours.iter().flatten())
                .map(core::ops::Deref::deref)
                .map(|&Amounts { weight, bakers, .. }| {
                    Math::PercentOf { product: weight, pct: bakers, of: flour.weight }.to_whence()
                }),
        )
        .chain(
            /* each item's total weight is the sum of its weight from mixes
             * and weight in added to final dough */
            /* also its percent_in_mixes and percent_less_mixes are ratios of its weight */
            [total, flour, nonflour]
                .into_iter()
                .chain(flours.iter().flatten())
                .chain(nonflours.iter().flatten())
                .flat_map(|i| i.with_mixes())
                .flat_map(
                    |&WithMixes {
                         weight_in_mixes,
                         weight_less_mixes,
                         percent_in_mixes,
                         percent_less_mixes,
                         amounts: Amounts { weight, .. },
                         ..
                     }| {
                        [
                            [weight_in_mixes, weight_less_mixes]
                                .sums_to(weight)
                                .to_whence(),
                            Math::PercentOf {
                                product: weight_in_mixes,
                                pct: percent_in_mixes,
                                of: weight,
                            }
                            .to_whence(),
                            Math::PercentOf {
                                product: weight_less_mixes,
                                pct: percent_less_mixes,
                                of: weight,
                            }
                            .to_whence(),
                        ]
                    },
                ),
        )
    }

    pub type Summands = Box<[Index]>;

    #[derive(Debug)]
    pub enum Math {
        Sum {
            sum: Index,
            ands: Summands,
        },
        PercentOf {
            product: Index,
            pct: Index,
            of: Index,
        },
        /// used to default mix flours' baker's percentage to 100%
        TotalFlourBakers100 {
            index: Index,
        },
    }

    impl Math {
        pub fn indexes(&self) -> impl Iterator<Item = Index> + '_ {
            /* this just here to do compiler error if a variant is added but you forgor to add
             * it below :) */
            match self {
                Math::Sum { .. } => (),
                Math::PercentOf { .. } => (),
                Math::TotalFlourBakers100 { .. } => (),
            };

            let sum = if let Math::Sum { sum, ands } = self {
                Some(once(sum).chain(ands.iter()))
            } else {
                None
            }
            .into_iter();

            let percent_of = if let Math::PercentOf { product, pct, of } = self {
                Some([product, pct, of].into_iter())
            } else {
                None
            }
            .into_iter();

            let flour = if let Math::TotalFlourBakers100 { index } = self {
                Some(once(index))
            } else {
                None
            }
            .into_iter();

            sum.flatten()
                .chain(percent_of.flatten())
                .chain(flour.flatten())
                .cloned()
        }

        /// May return an unsolved value for some PercentOf operations.
        ///
        /// Things like 0.0 / 0.0 or Inf * 0.0 will evaluate to NaN, the unsolved value. So
        /// this can return an unsolved value even if none of the value in the expression are
        /// unsolved. So this is kind a fallible that way.
        ///
        /// If that happens, just discard this rule as you would otherwise but don't treat the
        /// value as solved and maybe there's a better Math somewhere else that will solve for
        /// the value you want.
        pub fn solve_for(&self, solve_for: Index, values: &Values) -> Value {
            let value = |i| values.value(i);

            match self {
                &Math::Sum { sum, ref ands } => {
                    if solve_for == sum {
                        ands.iter().cloned().map(value).sum()
                    } else if ands.contains(&solve_for) {
                        value(sum)
                            - ands
                                .iter()
                                .cloned()
                                .filter(|&i| solve_for != i)
                                .map(value)
                                .sum::<f32>()
                    } else {
                        super::UNSOLVED
                    }
                }
                &Math::PercentOf { product, pct, of } => {
                    if solve_for == product {
                        value(pct) * value(of)
                    } else if solve_for == pct {
                        value(product) / value(of)
                    } else if solve_for == of {
                        value(product) / value(pct)
                    } else {
                        super::UNSOLVED
                    }
                }

                &Math::TotalFlourBakers100 { index } => {
                    if solve_for == index {
                        1.0
                    } else {
                        super::UNSOLVED
                    }
                }
            }
        }

        /// true iff solved and the values are consistent with this Math.
        pub fn check(&self, values: &Values) -> bool {
            let value = |i| values.value(i);

            match self {
                &Math::Sum { sum, ref ands } => {
                    value(sum) == ands.iter().cloned().map(value).sum::<f32>()
                }
                // TODO this floating point comparison probably needs some kind of error margin
                &Math::PercentOf { product, pct, of } => value(product) == value(pct) * value(of),
                &Math::TotalFlourBakers100 { index } => 1.0 == value(index),
            }
        }

        // #[cfg(test)]
        #[cfg(debug_assertions)]
        pub fn display(&self, values: &Values) -> impl core::fmt::Display {
            once(match self {
                Math::Sum { .. } => "sum".to_string(),
                Math::PercentOf { .. } => "pct".to_string(),
                Math::TotalFlourBakers100 { .. } => "flr".to_string(),
            })
            .chain(
                self.indexes()
                    .map(|i| format!(" {}[{}]", values.value(i), i)),
            )
            .collect::<String>()
        }
    }

    #[test]
    fn test_math_solve_sum() {
        use super::is_unsolved;

        let values = Values::from((0..16).map(|n| n as f32).collect::<Vec<_>>());

        assert_eq!(0.0, [].sums_to(0).solve_for(0, &values));
        assert_eq!(6.0, [2, 4].sums_to(0).solve_for(0, &values));
        assert_eq!(5.0, [3, 0].sums_to(8).solve_for(0, &values));

        assert!(is_unsolved([].sums_to(5).solve_for(0, &values)));
    }

    #[test]
    fn test_math_solve_pct() {
        use super::UNSOLVED;
        use Math::PercentOf;

        assert_eq!(
            0.400,
            PercentOf { product: 0, pct: 1, of: 2 }
                .solve_for(0, &Values::from(vec![UNSOLVED, 0.50, 0.800]))
        );
        assert_eq!(
            5.000,
            PercentOf { product: 0, pct: 1, of: 2 }
                .solve_for(1, &Values::from(vec![2.500, UNSOLVED, 0.500]))
        );
        assert_eq!(
            0.40,
            PercentOf { product: 0, pct: 1, of: 2 }
                .solve_for(2, &Values::from(vec![1.000, 2.500, UNSOLVED]))
        );

        assert_eq!(
            0.0,
            PercentOf { product: 0, pct: 1, of: 2 }
                .solve_for(1, &Values::from(vec![0.0, UNSOLVED, 1.0]))
        );
        assert_eq!(
            0.0,
            PercentOf { product: 0, pct: 1, of: 2 }
                .solve_for(1, &Values::from(vec![0.0, UNSOLVED, 1.0]))
        );
        assert_eq!(
            f32::NEG_INFINITY,
            PercentOf { product: 0, pct: 1, of: 2 }
                .solve_for(1, &Values::from(vec![2.0, UNSOLVED, -0.0]))
        );

        /* not exactly intended behaviour but something to be aware of */
        assert!(PercentOf { product: 0, pct: 1, of: 2 }
            .solve_for(1, &Values::from(vec![0.0, UNSOLVED, 0.0]))
            .is_nan());
        assert!(PercentOf { product: 0, pct: 1, of: 2 }
            .solve_for(1, &Values::from(vec![0.0, UNSOLVED, 0.0]))
            .is_nan());
        assert!(PercentOf { product: 0, pct: 1, of: 2 }
            .solve_for(1, &Values::from(vec![UNSOLVED, f32::INFINITY, 0.0]))
            .is_nan());
    }

    #[test]
    fn test_math_check_sum() {
        use super::UNSOLVED;

        let values = Values::from((0..16).map(|n| n as f32).collect::<Vec<_>>());

        assert!([].sums_to(0).check(&values));
        assert!([1, 2].sums_to(3).check(&values));

        assert!(![1, 2].sums_to(4).check(&values));
        assert!(![0, 1]
            .sums_to(2)
            .check(&Values::from(vec![0.0, 1.0, UNSOLVED])));
    }

    #[test]
    fn test_math_check_percent() {
        use super::UNSOLVED;
        use Math::PercentOf;

        assert!(PercentOf { product: 0, pct: 1, of: 2 }.check(&Values::from(vec![3.0, 2.0, 1.5])));
        assert!(PercentOf { product: 0, pct: 1, of: 2 }.check(&Values::from(vec![0.0, 0.0, 1.5])));

        assert!(!PercentOf { product: 0, pct: 1, of: 2 }
            .check(&Values::from(vec![UNSOLVED, 1.0, 1.5])));
        assert!(!PercentOf { product: 0, pct: 1, of: 2 }
            .check(&Values::from(vec![1.0, UNSOLVED, 1.5])));
    }

    trait SumsTo {
        fn sums_to(self, _: Index) -> Math;
    }

    impl<I> SumsTo for I
    where
        I: IntoIterator<Item = Index>,
    {
        fn sums_to(self, sum: Index) -> Math {
            let ands = self.into_iter().collect::<Summands>();
            Math::Sum { sum, ands }
        }
    }

    // wtf is this worth it?
    //
    // trait IterSome {
    //     type Item<'i>
    //     where
    //         Self: 'i;

    //     type ItersSome<'a>: Iterator<Item = Self::Item<'a>>
    //     where
    //         Self: 'a;

    //     fn iter_some<'a>(&'a self) -> Self::ItersSome<'a>;
    // }

    // impl<I> IterSome for Vec<Option<I>> {
    //     type Item<'i> = &'i I
    //         where Self: 'i;

    //     type ItersSome<'a> = std::iter::Flatten<std::slice::Iter<'a, Option<I>>>
    //         where Self: 'a;

    //     fn iter_some<'a>(&'a self) -> Self::ItersSome<'a> {
    //         self.iter().flatten()
    //     }
    // }
}

pub mod solve {
    use super::rules::Recipe;
    use super::{is_unsolved, rules, Index, Value, Values, Whence};

    use core::borrow::BorrowMut;

    #[derive(Debug)]
    pub struct Solver {
        maths: Vec<MathToSolve>,
        // indexes in maths that can be solved
        maths_by_index_to_solve: Vec<usize>,
        // unsolved value indexes paired with indexes in maths where those values are used
        unsolved_value_to_math_index_pairs: Vec<(Index, usize)>,
    }

    #[derive(Debug)]
    struct MathToSolve {
        // math: rules::Math,
        math: Whence<rules::Math>,
        unsolved: usize,
    }

    pub type SolveStep = (Index, Value, usize);

    impl Solver {
        pub fn new(recipe: &Recipe, values: &Values) -> Self {
            let mut maths = Vec::new();
            let mut maths_by_index_to_solve = Vec::new();
            let mut unsolved_value_to_math_index_pairs = Vec::new();

            /* fallback rules are iterated first so that they are popped last from
             * maths_by_index_to_solve */

            for math in rules::for_recipe_fallback(&recipe).chain(rules::for_recipe(&recipe)) {
                let math_index = maths.len();
                let last_pairs_len = unsolved_value_to_math_index_pairs.len();

                unsolved_value_to_math_index_pairs.extend(
                    math.indexes()
                        .filter(|&value_index| is_unsolved(values.value(value_index)))
                        .map(|value_index| (value_index, math_index)),
                );

                let unsolved = unsolved_value_to_math_index_pairs.len() - last_pairs_len;

                if unsolved == 1 {
                    maths_by_index_to_solve.push(math_index);
                }

                if unsolved > 0 {
                    maths.push(MathToSolve { math, unsolved });
                }
            }

            Self { maths, maths_by_index_to_solve, unsolved_value_to_math_index_pairs }
        }

        pub fn math(&self, index: usize) -> Option<&Whence<rules::Math>> {
            self.maths.get(index).map(|MathToSolve { math, .. }| math)
        }

        pub fn unsolved_value_to_math_index_pairs(&self) -> &[(Index, usize)] {
            self.unsolved_value_to_math_index_pairs.as_slice()
        }

        /// on success, yields the value index, value, math index
        pub fn step(&mut self, values: &mut Values) -> Option<SolveStep> {
            let Self { maths, maths_by_index_to_solve, unsolved_value_to_math_index_pairs } = self;

            while let Some(math_index) = maths_by_index_to_solve.pop() {
                let Some(MathToSolve { math, unsolved }) = maths.get(math_index) else {
                    debug_assert!(
                        false,
                        "invalid index {math_index} in maths_by_index_to_solve"
                    );
                    continue;
                };

                if *unsolved < 1 {
                    // this can happen if a value can be solved in more than one way (by
                    // different maths)
                    // suppose maths_by_index_to_solve contains both
                    //   2 + 3 = x
                    //   6 - 1 = x
                    // x is solved on the first pop; skip over the next pop
                    continue;
                }

                let Some(solve_for) = math
                    .indexes()
                    .find(|&value_index| is_unsolved(values.value(value_index)))
                else {
                    debug_assert!(
                            false,
                            "no unsolved value found in math {math_index} {math:?} popped from maths_by_index_to_solve"
                        );
                    continue;
                };

                let value = math.solve_for(solve_for, &values);

                if is_unsolved(value) {
                    /* If this happens, we don't try this math again because it's popped. Hopefully
                     * the value can be solved with some other math later on. */
                    continue;
                }

                *values.value_mut(solve_for) = value;

                /* since solve_for was solved, remove it from unsolved_value_to_math_index_pairs
                 * and update maths where it occurs, possibly queueing them on to
                 * maths_by_index_to_solve */

                for i in (0..unsolved_value_to_math_index_pairs.len()).rev() {
                    let (unsolved_value_index, unsolved_math_index) =
                        unsolved_value_to_math_index_pairs[i];

                    if unsolved_value_index != solve_for {
                        continue;
                    }

                    let Some(unsolved_math) = maths.get_mut(unsolved_math_index) else {
                        debug_assert!(false, "invalid maths index {unsolved_math_index} somewhere in unsolved_value_to_math_index_pairs");
                        continue;
                    };

                    unsolved_math.unsolved -= 1;

                    if unsolved_math.unsolved == 1 {
                        maths_by_index_to_solve.push(unsolved_math_index);
                    }

                    unsolved_value_to_math_index_pairs.swap_remove(i);
                }

                return Some((solve_for, value, math_index));
            }

            None
        }

        pub fn iter<'s>(&'s mut self, values: &'s mut Values) -> Iter<&mut Self, &mut Values> {
            Iter(self, values)
        }
    }

    pub struct Iter<S, V>(S, V)
    where
        S: BorrowMut<Solver>,
        V: BorrowMut<Values>;

    impl<S, V> Iterator for Iter<S, V>
    where
        S: BorrowMut<Solver>,
        V: BorrowMut<Values>,
    {
        type Item = SolveStep;

        fn next(&mut self) -> Option<Self::Item> {
            let Iter(solver, values) = self;
            solver.borrow_mut().step(values.borrow_mut())
        }
    }

    impl<S, V> Iter<S, V>
    where
        S: BorrowMut<Solver>,
        V: BorrowMut<Values>,
    {
        pub fn into_inner(self) -> (S, V) {
            let Iter(solver, values) = self;
            (solver, values)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// try to round a percentage to two decimal places? (good luck)
    ///
    /// ex 0.42321 == 42.321% should be rounded to 0.4232 == 42.32%
    fn round_pct(p: f32) -> f32 {
        (p * 100_00.0).round() / 100_00.0
    }

    // fn really_round_pct(p: f32) -> f32 {
    //     (p * 100.0).round() / 100.0
    // }

    /// round kilograms to grams?
    fn round_g(v: f32) -> f32 {
        (v * 100_0.0).round() / 100_0.0
    }

    #[test]
    fn test_funny_pizza() {
        let mut values = Values::from(Vec::with_capacity(1024));

        let total = values.new_item();
        let flour = values.new_item();
        let nonflour = values.new_item();

        let water = values.new_item();
        let salt = values.new_item();
        let rye = values.new_item();
        let wheat = values.new_item();
        let gluten = values.new_item();
        let double0 = values.new_item();

        // dough/recipe total weight
        *values.value_mut(total.weight) = 0.690;
        // FIXME should be assumed?
        *values.value_mut(flour.bakers) = 1.0;
        *values.value_mut(flour.percent_in_mixes) = 0.15;

        *values.value_mut(rye.percent_in_mixes) = 1.0;
        *values.value_mut(gluten.bakers) = 0.07;
        *values.value_mut(double0.bakers) = 0.25;
        *values.value_mut(water.bakers) = 0.75;
        *values.value_mut(salt.bakers) = 0.028;

        let starter_flour = values.new_mix_item();
        let starter_water = values.new_mix_item();
        let starter_rye = values.new_mix_item();

        *values.value_mut(starter_flour.percent_of_total) = 0.15;
        *values.value_mut(starter_water.bakers) = 1.40;

        let recipe = rules::Recipe {
            dough: rules::Mix {
                total: total.clone().into(),
                flour: flour.clone().into(),
                nonflour: nonflour.into(),
                flours: [&rye, &wheat, &gluten, &double0]
                    .into_iter()
                    .map(|i| Some(rules::Item::from(i.clone())))
                    .collect(),
                nonflours: [&water, &salt]
                    .into_iter()
                    .map(|i| Some(rules::Item::from(i.clone())))
                    .collect(),
            },
            mixes: vec![rules::Mix {
                total: values.new_mix_item().into(),
                flour: starter_flour.clone().into(),
                nonflour: values.new_mix_item().into(),
                flours: vec![Some(starter_rye.clone().into())],
                nonflours: vec![Some(starter_water.clone().into())],
            }],
        };

        assert_eq!(values.did_overflow(), false);

        // dbg!(&recipe);

        let mut solver = solve::Solver::new(&recipe, &values);

        while let Some((index, value, math)) = solver.step(&mut values) {
            // dbg!((index, value, math));
            // eprintln!("{}", solver.math(math).unwrap().display(&values));
            #[allow(dropping_copy_types)]
            drop((index, value, math));
        }

        assert_eq!(solver.unsolved_value_to_math_index_pairs().len(), 0);

        assert_eq!(values.value(total.bakers), 1.778);
        assert_eq!(values.value(flour.bakers), 1.000);

        assert_eq!(round_pct(values.value(rye.bakers)), 0.15);
        assert_eq!(values.value(wheat.bakers), 0.53);

        assert_eq!(round_g(values.value(flour.weight)), 0.388);
        assert_eq!(round_g(values.value(rye.weight)), 0.058);
        assert_eq!(round_g(values.value(wheat.weight)), 0.206);
        assert_eq!(round_g(values.value(gluten.weight)), 0.027);
        assert_eq!(round_g(values.value(double0.weight)), 0.097);
        assert_eq!(round_g(values.value(water.weight)), 0.291);
        assert_eq!(round_g(values.value(salt.weight)), 0.011);

        assert_eq!(round_g(values.value(starter_flour.weight)), 0.058);
        assert_eq!(round_g(values.value(starter_rye.weight)), 0.058);
        assert_eq!(round_g(values.value(starter_water.weight)), 0.081);

        assert_eq!(round_g(values.value(rye.weight_less_mixes)), 0.0);
        assert_eq!(round_g(values.value(water.weight_less_mixes)), 0.210);
    }

    #[test]
    fn test_contradiction() {
        let mut values = Values::from(Vec::with_capacity(1024));

        let total = values.new_item();
        let flour = values.new_item();
        let nonflour = values.new_item();

        let wheat = values.new_item();
        let water = values.new_item();

        *values.value_mut(total.weight) = 1.0;
        *values.value_mut(flour.bakers) = 1.0;
        *values.value_mut(water.bakers) = 1.0;
        *values.value_mut(wheat.weight) = 0.123;

        let recipe = rules::Recipe {
            dough: rules::Mix {
                total: total.clone().into(),
                flour: flour.clone().into(),
                nonflour: nonflour.into(),
                flours: vec![Some(wheat.clone().into())],
                nonflours: vec![Some(water.clone().into())],
            },
            mixes: vec![],
        };

        let mut solver = solve::Solver::new(&recipe, &values);

        while let Some((index, value, math)) = solver.step(&mut values) {
            let math: &Whence<_> = solver.math(math).unwrap();
            eprintln!("{} {}", math.line(), math.display(&values));
            #[allow(dropping_copy_types)]
            drop((index, value, math));
        }

        assert_eq!(values.did_overflow(), false);
        assert_eq!(solver.unsolved_value_to_math_index_pairs().len(), 0);
    }

    /*
    /// from 2009FormulaFormattingSINGLES p.5 diagram 4
    // #[test]
    fn _test_rustic_sourdough_with_three_flours_a_cracked_wheat_soaker_and_a_yeasted_preferment() {
        let white = Ingredient::new_flour("White".to_string());
        let whole = Ingredient::new_flour("Whole".to_string());
        let rye = Ingredient::new_flour("Rye".to_string());

        let water = Ingredient::new_nonflour("Water".to_string());
        let salt = Ingredient::new_nonflour("Salt".to_string());
        let yeast = Ingredient::new_nonflour("Yeast".to_string());
        let cracked_wheat = Ingredient::new_nonflour("Cracked Wheat".to_string());
        let soaker_water = Ingredient::new_nonflour("Soaker Water".to_string());

        let seed = Ingredient::new_nonflour("Seed".to_string());

        let mut dough = Dough::default();
        dough.totals.total.weight = 15_000;
        dough.totals.items.push(white.clone().bakers(0.70));
        dough.totals.items.push(whole.clone().bakers(0.20));
        dough.totals.items.push(rye.clone().bakers(0.10));
        dough.totals.items.push(water.clone().bakers(0.72));
        dough.totals.items.push(salt.clone().bakers(0.02));
        dough.totals.items.push(yeast.clone().bakers(0.0002));
        dough.totals.items.push(cracked_wheat.clone().bakers(0.10));
        dough.totals.items.push(soaker_water.clone().bakers(0.10));
        dough.totals.items.push(seed.clone().bakers(0.017));

        dough.mixes.push(Mix {
            title: "Soaker".to_string(),
            items: vec![
                cracked_wheat.clone().in_other(1.00),
                soaker_water.clone().in_other(1.00),
            ],
            ..default()
        });

        dough.mixes.push(Mix {
            title: "Liquid Yeasted Starter".to_string(),
            // TODO this is an important default probably?
            flour: Amounts { bakers: 1.0, ..default() },
            items: vec![
                white.clone().in_other(0.30),
                whole.clone().in_other(0.05),
                rye.clone().in_other(0.05),
                water.clone().bakers(1.05),
                yeast.clone().bakers(0.0010),
            ],
            ..default()
        });

        dough.mixes.push(Mix {
            title: "Sourdough Starter".to_string(),
            flour: Amounts { bakers: 1.0, ..default() },
            items: vec![
                white.clone().in_other(0.10),
                whole.clone().in_other(0.35),
                rye.clone().in_other(0.30),
                water.clone().bakers(0.56),
                seed.clone().bakers(0.10),
            ],
            ..default()
        });

        dough.finals.resize_with(dough.totals.items.len(), default);

        solve(&mut dough);

        let ingredient_totals = |i| dough.totals.find_ingredient(i).unwrap();

        let soaker = &dough.mixes[0];
        let yeasted = &dough.mixes[1];
        let sourdough = &dough.mixes[2];

        assert_eq!(dough.totals.flour.weight, 7_664);
        assert_eq!(ingredient_totals(&white).weight, 5_365);
        assert_eq!(ingredient_totals(&whole).weight, 1_533);
        assert_eq!(ingredient_totals(&rye).weight, 766);
        assert_eq!(ingredient_totals(&water).weight, 5_518);
        assert_eq!(ingredient_totals(&salt).weight, 153);
        assert_eq!(ingredient_totals(&yeast).weight, 2);
        assert_eq!(ingredient_totals(&cracked_wheat).weight, 766);
        assert_eq!(ingredient_totals(&soaker_water).weight, 766);

        assert_eq!(soaker.find_ingredient(&cracked_wheat).unwrap().weight, 766);
        assert_eq!(soaker.find_ingredient(&soaker_water).unwrap().weight, 766);
        // assert_eq!(soaker.flour, default());
        assert_eq!(soaker.total.weight, 1_532); // the book says 1_533 but 2 * 766 = 1_532 so ...
                                                // FIXME assert_eq!(soaker.total.in_other, 2.00);

        assert_eq!(yeasted.find_ingredient(&white).unwrap().weight, 1_610); // 1_609 in book but rounds .5 down?
        assert_eq!(yeasted.find_ingredient(&whole).unwrap().weight, 77);
        assert_eq!(yeasted.find_ingredient(&rye).unwrap().weight, 38);
        assert_eq!(yeasted.find_ingredient(&water).unwrap().weight, 1_811);
        assert_eq!(yeasted.find_ingredient(&yeast).unwrap().weight, 2);
        assert_eq!(yeasted.flour.weight, 1_725);
        assert_eq!(yeasted.total.weight, 3_538);
        assert_eq!(round_pct(yeasted.total.bakers), 2.051);

        assert_eq!(really_round_pct(ingredient_totals(&white).in_other), 0.40);
        assert_eq!(really_round_pct(ingredient_totals(&whole).in_other), 0.40);
        assert_eq!(really_round_pct(ingredient_totals(&rye).in_other), 0.35);
        // assert_eq!(round_pct(dough.totals.flour.in_other), 0.395);

        assert_eq!(sourdough.find_ingredient(&white).unwrap().weight, 537);
        assert_eq!(sourdough.find_ingredient(&whole).unwrap().weight, 537);
        assert_eq!(sourdough.find_ingredient(&rye).unwrap().weight, 230);
        /* the math on the sourdough's water seems wrong in the reference document?
         * it says 966g and 56% of 1304g. maybe the baker's % was accidentally calculated from the
         * liquid yeasted starter's total flour insted of its own?
         * In this test I'm using the amount I think is correct, the surplus water is added to the
         * final dough at the end. */
        assert_eq!(sourdough.find_ingredient(&water).unwrap().weight, 730);
        assert_eq!(sourdough.flour.weight, 1_304);

        assert_eq!(
            dough.finals,
            vec![
                /* white */ 3_218, /* whole */ 919, /* rye */ 498,
                /* water*/ 2_977, /* salt */ 153, /* yeast */ 0,
                /* cracked_wheat */ 0, /* soaker_water */ 0, /* seed */ 0,
            ]
        );
    }
    */
}

/// wraps a type like `Whence<Math>` to track on what line something is defined on for debugging
/// later on
#[derive(Debug)]
pub struct Whence<T> {
    inner: T,
    #[cfg(debug_assertions)]
    line: u32,
}

pub trait ToWhence: Sized {
    fn to_whence(self) -> Whence<Self>;
}

impl<T: Sized> ToWhence for T {
    #[track_caller]
    fn to_whence(self) -> Whence<Self> {
        #[cfg(debug_assertions)]
        let line = Location::caller().line();
        Whence {
            #[cfg(debug_assertions)]
            line,
            inner: self,
        }
    }
}

derefs!(Whence<T> => inner: T);

impl<T> Whence<T> {
    pub fn into_inner(self) -> T {
        self.inner
    }

    #[cfg(debug_assertions)]
    pub fn line(&self) -> u32 {
        self.line
    }
}
