use cryptocurrency_kit::crypto::{hash, CryptoHash, Hash};
use cryptocurrency_kit::storage::values::StorageValue;
use rmps::decode::Error;
use rmps::{Deserializer, Serializer};
use serde::{Deserialize, Serialize};

use std::borrow::Borrow;
use std::borrow::Cow;
use std::cmp::Ordering;
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};
use std::io::Cursor;

pub type Height = u64;

pub trait Proposal: Debug + Display {
    fn height(&self) -> Height;
}

#[derive(Debug)]
pub struct Request<T: Proposal + CryptoHash + StorageValue> {
    proposal: T,
}

#[derive(Debug, Clone, Eq, Deserialize, Serialize)]
pub struct View {
    pub round: u64,
    pub height: Height,
}

implement_cryptohash_traits! {View}
implement_storagevalue_traits! {View}

impl Proposal for View {
    fn height(&self) -> Height {
        self.height
    }
}

impl Display for View {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "height:{}, round: {}", self.height, self.round)
    }
}

impl PartialEq for View {
    fn eq(&self, other: &Self) -> bool {
        self.height == other.height && self.round == other.round
    }
}

impl PartialOrd for View {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let order = self.height.partial_cmp(&other.height);
        match order {
            Some(order) => match order {
                Ordering::Equal => self.round.partial_cmp(&other.round),
                _ => Some(order),
            },
            None => unreachable!(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::io::{self, Write};

    #[test]
    fn test_view() {
        (0..10).for_each(|i| {
            let view = View {
                round: i as u64,
                height: (i + 1) as Height,
            };
            writeln!(io::stdout(), "{}", view).unwrap();
            let expect_view = view.clone();
            let buf = view.into_bytes();
            let got_view = View::from_bytes(Cow::from(buf));
            assert_eq!(got_view.height, expect_view.height);
            assert_eq!(got_view.round, expect_view.round);
        });
    }

    #[test]
    fn test_cmp() {
        {
            let (mut a, mut b) = (
                View {
                    height: 1,
                    round: 1,
                },
                View {
                    height: 1,
                    round: 1,
                },
            );
            assert_eq!(a, b);

            let (mut a, mut b) = (
                View {
                    height: 2,
                    round: 1,
                },
                View {
                    height: 1,
                    round: 1,
                },
            );
            assert_ne!(a, b);

            let (mut a, mut b) = (
                View {
                    height: 2,
                    round: 1,
                },
                View {
                    height: 2,
                    round: 2,
                },
            );
            assert_ne!(a, b);
        }

        /// Greeter
        {
            let (mut a, mut b) = (
                View {
                    height: 1,
                    round: 1,
                },
                View {
                    height: 1,
                    round: 0,
                },
            );
            assert!(a > b);
            let (mut a, mut b) = (
                View {
                    height: 1,
                    round: 1,
                },
                View {
                    height: 0,
                    round: 10,
                },
            );
            assert!(a > b);
        }

        /// Less
        {
            let (mut a, mut b) = (
                View {
                    height: 1,
                    round: 0,
                },
                View {
                    height: 1,
                    round: 1,
                },
            );
            assert!(a < b);
            let (mut a, mut b) = (
                View {
                    height: 0,
                    round: 12,
                },
                View {
                    height: 1,
                    round: 10,
                },
            );
            assert!(a < b);
        }

        /// GreeterEq
        {
            let (mut a, mut b) = (
                View {
                    height: 1,
                    round: 1,
                },
                View {
                    height: 1,
                    round: 1,
                },
            );
            assert!(a >= b);
            let (mut a, mut b) = (
                View {
                    height: 2,
                    round: 1,
                },
                View {
                    height: 1,
                    round: 1,
                },
            );
            assert!(a >= b);
            let (mut a, mut b) = (
                View {
                    height: 1,
                    round: 1,
                },
                View {
                    height: 1,
                    round: 0,
                },
            );
            assert!(a >= b);
            let (mut a, mut b) = (
                View {
                    height: 1,
                    round: 1,
                },
                View {
                    height: 0,
                    round: 10,
                },
            );
            assert!(a >= b);
        }

        /// LessEq
        {
            let (mut a, mut b) = (
                View {
                    height: 1,
                    round: 1,
                },
                View {
                    height: 1,
                    round: 1,
                },
            );
            assert!(a <= b);
            let (mut a, mut b) = (
                View {
                    height: 1,
                    round: 1,
                },
                View {
                    height: 2,
                    round: 1,
                },
            );
            assert!(a <= b);
            let (mut a, mut b) = (
                View {
                    height: 1,
                    round: 0,
                },
                View {
                    height: 1,
                    round: 1,
                },
            );
            assert!(a <= b);
            let (mut a, mut b) = (
                View {
                    height: 0,
                    round: 12,
                },
                View {
                    height: 1,
                    round: 10,
                },
            );
            assert!(a <= b);
        }
    }
}
