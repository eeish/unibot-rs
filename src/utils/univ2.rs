use ethers::prelude::*;
use hex;
use sha3::{Digest, Keccak256};

pub fn get_univ2_exact_weth_token_min_recv(amount_out_min: U256, path: Vec<Address>) {
    let user_min_recv = amount_out_min;

    for index in (path.capacity() - 1)..1 {
        let from = path[index];
        let to = path[index - 1];

        let pair_address = get_uni_pair_address(from, to);
    }
}

pub fn get_uni_pair_address(from: Address, to: Address) -> Address {
    let (from, to) = sort_token(from, to);

    //// TODO remove hard code uniswapv2 factory address
    let factory = "5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f"
        .parse::<Address>()
        .unwrap();

    //// TODO remove hard code init code
    let init_code =
        hex::decode("96e8ac4277198ff8b6f785478aa9a39f403cb768dd02cbee326c3e7da348845f").unwrap();

    let salt = create2_salt(address_pop(from.as_bytes()), address_pop(to.as_bytes()));

    let pair_address = ethers::core::utils::get_create2_address(factory, salt, init_code.clone());

    pair_address
}

pub fn sort_token(from: Address, to: Address) -> (Address, Address) {
    if from < to {
        (from, to)
    } else {
        (to, from)
    }
}

fn create2_salt(token0: &[u8; 20], token1: &[u8; 20]) -> [u8; 32] {
    let mut hasher = Keccak256::new();
    hasher.update(token0);
    hasher.update(token1);

    let mut code_hash = [0; 32];
    code_hash.copy_from_slice(&hasher.finalize());

    code_hash
}

fn address_pop(addr: &[u8]) -> &[u8; 20] {
    addr.try_into()
        .expect("address raw bytes array length can't cast to 20")
}
