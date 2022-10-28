use ethers::prelude::*;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::utils::univ2;

fn time() -> u64 {
    let time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    return time.as_secs();
}

pub fn swap_eth_for_exact_tokens_router(
    amount_out_min: U256,
    path: Vec<Address>,
    to: Address,
    deadline: U256,
) {
    if U256::from(time()) > deadline {
        println!("deadline exceeded, can't mev op");
        return;
    }

    univ2::get_univ2_exact_weth_token_min_recv(amount_out_min, path);
}
