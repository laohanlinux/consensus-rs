use crate::types::{Height, transaction::Transaction, block::Block};

use super::{util, slot};

// TODO: add_delegate
fn add_delegate() {}

/// Generates delegates list and checks if block generator publicKey maches delegate id.
pub fn validate_block_slot(block: Block) -> bool {
    // get the height delegates list
    let block_height = block.height();
    let delegates = slot::get_active_delegates(block_height);
    let slot_time = block.header().time as i64;

    let current_slot = slot::get_slot_number(slot_time);
    let idx = current_slot as usize % delegates.len();
    let delegate_id = delegates[idx];

    util::equal_delegate(delegate_id, &block.header().proposer.as_bytes())
}


//// TODO: Opz
//pub fn generate_delegate_list<'a>(height: Height) -> (Timespec, Vec<&'a str>){
//   let generator_id = slot::get_active_delegates(height);
//}

pub fn get_block_slot_data<'a>(slot: i64, height: Height) -> Option<(&'a str, i64)> {
    let current_slot = slot;
    let delegates = slot::get_active_delegates(height);
    let delegate_pos = current_slot % slot::DELEGATES;
    let delegate_id = delegates.get(delegate_pos as usize);
    Some((delegate_id.unwrap(), slot::get_slot_time(slot)))
}

// TODO:
// height --> slot_list
// 根据当前的slot，找到高度为height的相应见证人id以及相对应的slot
//
//fn get_block_slot_data<'a>(slot: i64, height: Height) -> Option<(&'a str, i64)> {
//    let current_slot = slot;
//    let last_slot = slot::get_last_slot(current_slot);
//    let delegates = slot::get_active_delegates(height);
//
//    for _slot in current_slot..last_slot {
//        let delegate_pos = _slot % slot::DELEGATES;
//        let deletegate_id = delegates.get(delegate_pos as usize);
//        if deletegate_id.is_none() {
//            continue;
//        }
//        return Some((deletegate_id.unwrap(), slot::get_slot_time(_slot)));
//    }
//    None
//}

//
//fn get_keys_sort_by_vote() {
//
//}
//
//// TODO
//fn get_accounts(_: String) {
//
//}

#[cfg(test)]
mod tests {
    use std::io::{self, Write};
    use super::*;

    #[test]
    fn test_get_block_slot_data() {
        for height in 1..12 {
            let slot = height - 1;
            let (delegate_id, slot_time) = get_block_slot_data(slot, Height(height as u64)).unwrap();
            let slot_number = super::slot::get_slot_number(slot_time);
            writeln!(io::stdout(), "deletegate_id: {}, slot_number: {}, slot_time: {}", delegate_id, slot_number, slot_time);
        }
    }
}
