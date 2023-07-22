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

#[derive(Clone)]
pub struct PairState {
    pub amount_out: U256,
    pub new_reserve_a: U256,
    pub new_reserve_b: U256,
}

impl PairState {
    pub fn from_tuple(t: (U256, U256, U256)) -> Self {
        Self {
            amount_out: t.0,
            new_reserve_a: t.1,
            new_reserve_b: t.2,
        }
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

pub fn get_univ2_data_given_out(
    user_min_recv: U256,
    reserve_from: U256,
    reserve_to: U256,
) -> (U256, U256, U256) {
    // Use checked_sub to avoid underflow
    let mut new_reserve_to = match reserve_to.checked_sub(user_min_recv) {
        Some(result) => result,
        None => return (U256::zero(), reserve_from, reserve_to), // return early with appropriate values
    };

    let numerator = reserve_from * user_min_recv * 1000;
    let denominator = reserve_to * 997;
    let a_amount_in = numerator / denominator + 1;

    let (mut new_reserve_from, ok) = reserve_from.overflowing_add(a_amount_in);
    if ok {
        new_reserve_from = U256::MAX;
    }

    (a_amount_in, new_reserve_from, new_reserve_to)
}

pub fn get_univ2_data_given_in(
    amountA_in: U256,
    reserve_a: U256,
    reserve_b: U256,
) -> (U256, U256, U256) {
    let amount_in_with_fee = amountA_in * 997;
    let numerator = amount_in_with_fee * reserve_b;
    let denominator = amount_in_with_fee + (reserve_a * 1000);
    let amountB_out = numerator / denominator;

    let (mut new_reserve_b, ok) = reserve_b.overflowing_sub(amountB_out);
    if !ok {
        new_reserve_b = U256::from(1);
    }

    let (mut new_reserve_a, ok) = reserve_a.overflowing_add(amountA_in);
    if !ok {
        new_reserve_a = U256::MAX;
    }
    (amountB_out, new_reserve_a, new_reserve_b)
}

pub fn calc_sandwich_optima_in(
    user_amount_in: U256,
    user_min_recv_token: U256,
    reserve_weth: U256,
    reserve_token: U256,
) -> U256 {
    let call_f = |amountIn: U256| -> U256 {
        let frontrunState = get_univ2_data_given_in(amountIn, reserve_weth, reserve_token);
        let victimState = get_univ2_data_given_in(user_amount_in, frontrunState.1, frontrunState.2);
        victimState.0
    };

    // FIXME: ge function with U256
    let pass_f = |amountOut: U256| -> bool { amountOut.ge(&user_min_recv_token) };

    let optimal_weth_in = binary_search(U256::from(0), U256::from(100), call_f, pass_f);
    optimal_weth_in
}

pub fn binary_search<F, G>(mut left: U256, mut right: U256, cal_func: F, pass_func: G) -> U256
where
    F: Fn(U256) -> U256,
    G: Fn(U256) -> bool,
{
    /// tolerance is 1%
    let tolerance = 100;

    while right.saturating_sub(left) > U256::one() {
        let mid = (right.saturating_add(left)) >> 1;

        let gap = right.saturating_sub(left);

        if gap.gt(&(mid / tolerance)) {
            let out = cal_func(mid);

            if pass_func(out) {
                left = mid;
            } else {
                right = mid;
            }
        } else {
            return mid;
        }
    }

    right
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

    #[test]
    fn test_get_univ2_data_given_out() {
        let res: (U256, U256, U256) = (U256::from(13), U256::from(1246), U256::from(23300));
        assert_eq!(
            res,
            get_univ2_data_given_out(U256::from(233), U256::from(1233), U256::from(23533))
        );
    }

    // generate pub fn binary_search<F, G>(left: U256, right: U256, cal_func: F, pass_func: G) -> U256  test
    #[test]
    fn test_binary_search() {
        let left = U256::from(1);
        let right = U256::from(3);
        let pass_func = |amount_out: U256| -> bool { amount_out.ge(&U256::from(10)) };

        assert_eq!(U256::from(2), binary_search(left, right, |x| x, pass_func));
    }
}
