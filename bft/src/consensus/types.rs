

use cryptocurrency_kit::crypto::{CryptoHash, hash, Hash};
use cryptocurrency_kit::storage::values::StorageValue;
use rmps::decode::Error;
use rmps::{Deserializer, Serializer};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::io::Cursor;

use std::borrow::Borrow;
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};

pub type Height = u64;

pub trait Proposal: Debug + Display {
    fn height(&self) -> Height;
}

#[derive(Debug)]
pub struct Request<T: Proposal + CryptoHash + StorageValue> {
    proposal: T,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
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

#[cfg(test)]
mod test {
    use super::*;
    use std::io::{self, Write};

    #[test]
    fn test_view() {
        (0..10).for_each(|i| {
            let view = View{
                round: i as u64,
                height: (i+1) as Height,
            };
            writeln!(io::stdout(), "{}", view).unwrap();
            let expect_view = view.clone();
            let buf = view.into_bytes();
            let got_view = View::from_bytes(Cow::from(buf));
            assert_eq!(got_view.height, expect_view.height);
            assert_eq!(got_view.round, expect_view.round);
        });
    }
}