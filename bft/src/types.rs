use serde::{Serialize, Serializer, Deserialize, Deserializer};

use std::time::Instant;

#[derive(Debug)]
pub struct Time(Instant);

impl Time {

}


impl Serialize for Time {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        serializer.serialize_u64(self.0.elapsed().as_secs())
    }
}

//impl<'de> Deserialize<'de> for Time {
//    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//    where
//        D: Deserializer<'de>
//    {
//        enum Field {Ins};
//
//        deserializer.deserialize_identifier(Time)
//    }
//}