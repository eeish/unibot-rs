use ethers::abi::*;
use ethers::prelude::*;
use ethers::types::Address;
use ethers::utils::keccak256;
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
    let init_code_hash =
        hex::decode("96e8ac4277198ff8b6f785478aa9a39f403cb768dd02cbee326c3e7da348845f").unwrap();

    let mut extend_byte_array = from.as_bytes().to_vec();
    let to_byte_array = to.as_bytes().to_vec();
    extend_byte_array.extend(to_byte_array);

    /// Attention here, ethers-rs: abi encoding not work
    /// let input = abi::encode(&vec![Token::Address(from), Token::Address(to)]);
    let salt = keccak256(&extend_byte_array);

    let salt2 =
        hex::decode("4aafb64a36177dc82e7ace74cf60cc655659bc049da9533b5f7a6881bea995c6").unwrap();

    let pair_address = ethers::core::utils::get_create2_address_from_hash(
        factory,
        salt.to_vec(),
        init_code_hash.to_vec(),
    );

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

#[cfg(test)]
mod tests {
    use super::*;
    use ethers::abi::Token;
    use ethers::abi::*;
    use ethers::types::Address;
    use ethers::utils::keccak256;
    use std::str;

    #[test]
    fn test_sort_token() {
        let token_usdt = "0xdAC17F958D2ee523a2206206994597C13D831ec7"
            .parse::<Address>()
            .unwrap();
        let token_usdc = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"
            .parse::<Address>()
            .unwrap();
        assert_eq!((token_usdc, token_usdt), sort_token(token_usdt, token_usdc));
    }

    //// #[test]
    //// fn test_create2_salt_gen() {
    ////     let token_usdt = "0xdAC17F958D2ee523a2206206994597C13D831ec7"
    ////         .parse::<ethers::types::Address>()
    ////         .unwrap();
    ////     let token_usdc = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"
    ////         .parse::<ethers::types::Address>()
    ////         .unwrap();

    ////     let input = abi::encode(&vec![
    ////         Token::Address(token_usdt),
    ////         Token::Address(token_usdc),
    ////     ]);
    ////     assert_eq!(
    ////         "0xd4eb34f6fc5238007dd65d8d3f4e0ec192c1af0afab7ee25cd0d05284ddc2f48",
    ////         keccak256(&input)
    ////     );
    //// }
    #[test]
    fn test_get_uni_pair_address() {
        let token_usdt = "dAC17F958D2ee523a2206206994597C13D831ec7"
            .parse::<Address>()
            .unwrap();
        let token_usdc = "A0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"
            .parse::<Address>()
            .unwrap();
        assert_eq!(
            "0x3041CbD36888bECc7bbCBc0045E3B1f144466f5f"
                .parse::<Address>()
                .unwrap(),
            get_uni_pair_address(token_usdt, token_usdc)
        );
    }

    //// #[test]
    //// fn test_address_to_bytes() {
    ////     let token_usdt = "0xdAC17F958D2ee523a2206206994597C13D831ec7"
    ////         .parse::<Address>()
    ////         .unwrap();

    ////     assert_eq!()
    //// }
}
