use slot;
use time::Timespec;
use chain::blockchain::Block;
use chain::helpers::Height;
use util;

// height --> slot_list
// 根据当前的slot，找到高度为height的相应见证人id以及相对应的slot
// 
fn get_block_slot_date<'a>(slot: i64, height: u64) -> Option<(&'a str, i64)>{
    let current_slot = slot;
    let last_slot = slot::get_last_slot(current_slot);
    let delegates = slot::get_active_delegates(height);


    for _slot in current_slot..last_slot {
        let delegate_pos = _slot % slot::DELEGATES;
        let deletegate_id = delegates.get(delegate_pos as usize);
        if deletegate_id.is_none() {
            continue;
        }
        return Some((deletegate_id.unwrap(), slot::get_slot_time(_slot)));
    }
    None
}

pub fn validate_block_slot(block: Block) {
    // get the height delegates list
    let block_height = u64::from(block.height());
    let delegates = slot::get_active_delegates(block_height);
    let slot_time = block.timestamp() as i64;

    let current_slot = slot::get_slot_number(slot_time);
    let idx = current_slot as usize % delegates.len();
    let delegate_id = delegates[idx];

//    if util::equal_delegate(delegate_id, &block.generator) {
//
//    }
}

#[cfg(test)]
mod tests{
    use std::io::{self, Write};
    use super::*;

    #[test]
    fn test_get_block_slot_data(){
        for height in 1..12 {
            let slot = height - 1;
            let (delegate_id, slot_time) = get_block_slot_date(slot, height).unwrap();
            let slot_number = super::slot::get_slot_number(slot_time);
            writeln!(io::stdout(), "deletegate_id: {}, slot_number: {}, slot_time: {}", delegate_id,slot_number, slot_time);
        }
    }
}