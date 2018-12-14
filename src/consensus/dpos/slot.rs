use time::{self, Timespec, Duration};
use chrono::*;
use cryptocurrency_kit::ethkey::Address;

use crate::types::Height;

///
///     [1, 2, 3, 4], [5, 6, 7, 8], [9, 10]
///     slot0           slot1      slot2(current slot)
///     next_slot = [13, 14, 15, 16]
pub const INTERVAL: i64 = 3;
pub const DELEGATES: i64 = 11;
pub const ACTIVE_DELEGATES:[&str; DELEGATES as usize] = [
    "a",
    "b",
    "c",
    "d",
    "e",
    "f",
    "g",
    "h",
    "i",
    "j",
    "k"];

lazy_static! {
    static ref ACTIVE_DELEGATES_LIST: Vec<&'static str> = {
        let mut active_delegates = vec![];
        {
            active_delegates.push("a");
            active_delegates.push("b");
            active_delegates.push("c");
            active_delegates.push("d");
            active_delegates.push("e");
            active_delegates.push("f");
            active_delegates.push("g");
            active_delegates.push("h");
            active_delegates.push("i");
            active_delegates.push("j");
            active_delegates.push("k");
        }
        active_delegates
    };
}

pub fn get_active_delegates<'a>(height: Height) -> Vec<&'a str> {
//    ACTIVE_DELEGATES.to_vec()
    ACTIVE_DELEGATES_LIST.clone()
}

/// this is a epoch time
pub fn get_time(time_spec: Timespec) -> i64{
     return epoch_time(time_spec)
}

/// real time, accurate to milliseconds
pub fn get_real_time(epoch_spec: i64) -> i64 {
    (epoch_spec + begin_epoch_time()) * 1000
}

/// epoch_time time's slot
pub fn get_slot_number(mut epoch_time: i64) -> i64 {
    if epoch_time == 0 {
        epoch_time = get_time(time::get_time());
    }
    return epoch_time / INTERVAL
}

/// this is epoch time
pub fn get_slot_time(slot: i64) -> i64{
    return slot * INTERVAL
}

// current slot + 1
pub fn get_next_slot() -> i64 {
    let time_now = time::get_time();
    let epoch_time = get_time(time_now);
    let slot = get_slot_number(epoch_time);
    slot + 1
}

pub fn get_last_slot(next_slot: i64) -> i64 {
    next_slot + DELEGATES
}

// [time_spec - begin_time]
fn epoch_time(time_spec: Timespec) -> i64 {
    time_spec.sec - begin_epoch_time()
}

// return begin epoch time
fn begin_epoch_time() -> i64 {
    let epoch_time = DateTime::parse_from_rfc2822("Fri, 14 Jul 2017 02:40:00 +0000").unwrap();
    epoch_time.timestamp()
}

fn round_time(data: Timespec) -> i64 {
    data.sec
}

// calc height round
fn calc_round(height: i64) -> i64{
    let round = (height as f64) / (DELEGATES as f64);
    round.ceil() as i64
}

#[cfg(test)]
mod tests {
    use std::io::{self, Write};

    #[test]
    fn test_epoch_time(){
        println!("Hello Word ....");
        let epoch = super::DateTime::parse_from_rfc2822("Fri, 14 Jul 2017 02:40:00 +0000").unwrap();
        writeln!(io::stdout(), "{}", epoch.timestamp()).unwrap();

        let time_now = super::time::get_time();
        let epoch_time = super::epoch_time(time_now);
        writeln!(io::stdout(), "epoch time {}", epoch_time).unwrap();
    }

    #[test]
    fn test_get_real_time(){
        let time_now = super::time::get_time();
        let epoch_time = super::get_time(time_now);
        assert_eq!(super::get_real_time(epoch_time), time_now.sec *1000);
        writeln!(io::stdout(), "real time {}", super::get_real_time(epoch_time)).unwrap();
    }

    #[test]
    fn test_get_slot_number(){
        let time_now = super::time::get_time();
        let epoch_time = super::get_time(time_now);
        writeln!(io::stdout(), "epoch time {}, slot number {}",epoch_time, super::get_slot_number(epoch_time)).unwrap();
    }

    #[test]
    fn test_get_next_slot_number(){
        let time_now = super::time::get_time();
        let epoch_time = super::get_time(time_now);
        let slot_number = super::get_slot_number(epoch_time);

        writeln!(io::stdout(), "prev slot number {}, next slot number {}", slot_number, super::get_next_slot()).unwrap();
    }

    #[test]
    fn test_round_time(){
        assert_eq!(super::calc_round(1), 1);
        assert_eq!(super::calc_round(10), 1);
        assert_eq!(super::calc_round(11), 1);
        assert_eq!(super::calc_round(12), 2);
    }
}