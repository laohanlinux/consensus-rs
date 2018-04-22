use time::{self, Timespec, Duration};
use chrono::*;


///
///     [1, 2, 3, 4], [5, 6, 7, 8], [9, 10]
///     slot1           slot2      slot3(current slot)
///     next_slot = [13, 14, 15, 16]
const interval: i64 = 3;
const delegates: i64 = 51;

pub fn get_time(time_spec: Timespec) -> i64{
     return epoch_time(time_spec)
}

// accurate to milliseconds
pub fn get_real_time(time_spec: Timespec) -> i64 {
    let epoch_time = get_time(time_spec);
    (epoch_time + begin_epoch_time()) * 1000
}

pub fn get_slot_number(epoch_time: i64) -> i64 {
    return epoch_time / interval
}

pub fn get_slot_time(slot: i64) -> i64{
    return slot * interval
}

// current slot + 1
pub fn get_next_slot() -> i64 {
    let time_now = time::get_time();
    let epoch_time = get_time(time_now);
    let slot = get_slot_number(epoch_time);
    slot + 1
}

pub fn get_last_slot(next_slot: i64) -> i64 {
    next_slot + delegates
}

// [time_spec - begin_time]
fn epoch_time(time_spec: Timespec) -> i64 {
    let epoch_time = begin_epoch_time();
    time_spec.sec - epoch_time
}

// return begin epoch time
fn begin_epoch_time() -> i64 {
    let epoch_time = DateTime::parse_from_rfc2822("Fri, 14 Jul 2017 02:40:00 +0000").unwrap();
    epoch_time.timestamp()
}

#[cfg(test)]
mod tests {
    use std::io::{self, Write};
    use super::offset::LocalResult;
    use super::TimeZone;

    #[test]
    fn test_epoch_time(){
        println!("Hello Word ....");
        let epoch = super::DateTime::parse_from_rfc2822("Fri, 14 Jul 2017 02:40:00 +0000").unwrap();
        writeln!(io::stdout(), "{}", epoch.timestamp());

        let time_now = super::time::get_time();
        let epoch_time = super::epoch_time(time_now);
        writeln!(io::stdout(), "epoch time {}", epoch_time);
    }

    #[test]
    fn test_get_real_time(){
        let time_now = super::time::get_time();
        writeln!(io::stdout(), "real time {}", super::get_real_time(time_now));
    }

    #[test]
    fn test_get_slot_number(){
        let time_now = super::time::get_time();
        let epoch_time = super::get_time(time_now);
        writeln!(io::stdout(), "epoch time {}, slot number {}",epoch_time, super::get_slot_number(epoch_time));
    }

    #[test]
    fn test_get_next_slot_number(){
        let time_now = super::time::get_time();
        let epoch_time = super::get_time(time_now);
        let slot_number = super::get_slot_number(epoch_time);

        writeln!(io::stdout(), "prev slot number {}, next slot number {}", slot_number, super::get_next_slot());
    }
}