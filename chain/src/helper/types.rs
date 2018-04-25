use std::fmt;
use std::str::FromStr;
use std::num::ParseIntError;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Consensus round index.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct Round(pub u32);

impl Round {
    /// Returns zero value of the round.
    ///
    /// # Examples
    ///
    /// ```
    /// use exonum::helpers::Round;
    ///
    /// let round = Round::zero();
    /// assert_eq!(0, round.0);
    /// ```
    pub fn zero() -> Self {
        Round(0)
    }

    /// Returns first value of the round.
    ///
    /// # Examples
    ///
    /// ```
    /// use exonum::helpers::Round;
    ///
    /// let round = Round::first();
    /// assert_eq!(1, round.0);
    /// ```
    pub fn first() -> Self {
        Round(1)
    }

    /// Returns next value of the round.
    ///
    /// # Examples
    ///
    /// ```
    /// use exonum::helpers::Round;
    ///
    /// let round = Round(20);
    /// let next_round = round.next();
    /// assert_eq!(21, next_round.0);
    /// ```
    pub fn next(&self) -> Self {
        Round(self.0 + 1)
    }

    /// Returns previous value of the round.
    ///
    /// # Panics
    ///
    /// Panics if `self.0` is equal to zero.
    ///
    /// # Examples
    ///
    /// ```
    /// use exonum::helpers::Round;
    ///
    /// let round = Round(10);
    /// let previous_round = round.previous();
    /// assert_eq!(9, previous_round.0);
    /// ```
    pub fn previous(&self) -> Self {
        assert_ne!(0, self.0);
        Round(self.0 - 1)
    }

    /// Increments the round value.
    ///
    /// # Examples
    ///
    /// ```
    /// use exonum::helpers::Round;
    ///
    /// let mut round = Round::zero();
    /// round.increment();
    /// assert_eq!(1, round.0);
    /// ```
    pub fn increment(&mut self) {
        self.0 += 1;
    }

    /// Decrements the round value.
    ///
    /// # Panics
    ///
    /// Panics if `self.0` is equal to zero.
    ///
    /// # Examples
    ///
    /// ```
    /// use exonum::helpers::Round;
    ///
    /// let mut round = Round(20);
    /// round.decrement();
    /// assert_eq!(19, round.0);
    /// ```
    pub fn decrement(&mut self) {
        assert_ne!(0, self.0);
        self.0 -= 1;
    }

    /// Returns the iterator over rounds in the range from `self` to `to - 1`.
    ///
    /// # Examples
    ///
    /// ```
    /// use exonum::helpers::Round;
    ///
    /// let round = Round::zero();
    /// let mut iter = round.iter_to(Round(2));
    /// assert_eq!(Some(Round(0)), iter.next());
    /// assert_eq!(Some(Round(1)), iter.next());
    /// assert_eq!(None, iter.next());
    /// ```
    pub fn iter_to(&self, to: Round) -> RoundRangeIter {
        RoundRangeIter {
            next: *self,
            last: to,
        }
    }
}