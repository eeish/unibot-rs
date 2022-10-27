use ethers::prelude::*;
use std::time::{SystemTime, UNIX_EPOCH};

fn time() -> u64 {
    let time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    return time.as_secs();
}

pub fn swap_eth_for_exact_tokens_router(
    amount_out_min: U256,
    pair_from: Address,
    pair_to: Address,
    to: Address,
    deadline: U256,
) {
    if U256::from(time()) > deadline {
        println!("deadline exceeded, can't mev op");
        return;
    }


}


pub fn get_univ2_exact_weth_token_min_recv {
}
