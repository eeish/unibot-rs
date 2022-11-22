use ethers::prelude::*;
use ethers::types::Address;
use ethers::utils::keccak256;
use hex;

abigen!(
    IUniswapV2Pair,
    r#"[
        function getReserves() external view returns (uint112 reserve0, uint112 reserve1, uint32 blockTimestampLast)
    ]"#,
);

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

    //// Attention here, ethers-rs: abi encoding not work
    //// let input = abi::encode(&vec![Token::Address(from), Token::Address(to)]);
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
