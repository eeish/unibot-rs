use dotenv::dotenv;
use ethers::{
    abi::AbiDecode,
    prelude::*,
    providers::{Provider, Ws},
};
use eyre::Result;
use std::env;
use std::sync::Arc;

pub mod utils;
use utils::client::*;
use utils::debug_print::*;

mod constants;
mod env_store;
mod router;

abigen!(
    IUniswapV2Router02,
    r#"[
        swapExactTokensForTokens(uint256 amountIn, uint256 amountOutMin, address[] calldata path, address to, uint256 deadline)
        swapTokensForExactTokens(uint amountOut, uint amountInMax, address[] calldata path, address to,uint deadline)
        swapExactETHForTokens(uint amountOutMin, address[] calldata path, address to, uint deadline)
        swapTokensForExactETH(uint amountOut, uint amountInMax, address[] calldata path, address to, uint deadline)])
        swapExactTokensForETH(uint amountIn, uint amountOutMin, address[] calldata path, address to, uint deadline)])
        swapETHForExactTokens(uint amountOut, address[] calldata path, address to, uint deadline)])
    ]"#,
);

////abigen!(IUniswapV2Router02, "../contracts/UniswapV2Router02.json");

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    let infura_url: String = env::var("INFURA_MAINNET_WS").unwrap();
    let univ2_router: Address = env::var("UNIV2_ROUTE").unwrap().parse().unwrap();

    let envstore = env_store::EnvStore::new("INFURA_MAINNET_WS", "ETH_PRIVATE_KEY")?;

    let client = UniswapV2Client::new(envstore).await.unwrap();
    let client = Arc::new(client);

    let mut stream = client.get_pending_txs().await;

    while let Some(tx_hash) = stream.next().await {
        let tx = client.get_transaction(tx_hash).await;

        if tx.is_some() {
            let tx = tx.unwrap();
            parse_tx(&tx, &univ2_router);
        } else {
            continue;
        }
    }
    Ok(())
}

fn parse_tx(tx: &Transaction, router: &Address) {
    if tx.to.unwrap() == *router {
        println!("Uni transaction founded: tx={:?}", tx.hash);

        //// let mut pair_path: H160 = "0x0000000000000000000000000000000000000000"
        ////     .parse::<H160>()
        ////     .unwrap();

        if let Ok(decoded) = SwapExactTokensForTokensCall::decode(&tx.input) {
            let amount_in = decoded.amount_in;
            let amount_out_min = decoded.amount_out_min;
            let mut path = decoded.path.into_iter();
            let from = path.next().unwrap();
            let to = path.next().unwrap();
            let address_to = decoded.to;
            let deadline = decoded.deadline;
            println!(
                "{:?} {:?} {:?} {:?} {:?} {:?}",
                amount_in, amount_out_min, from, to, address_to, deadline
            );
        } else if let Ok(decoded) = SwapTokensForExactTokensCall::decode(&tx.input) {
            let amount_out = decoded.amount_out;
            let amount_in_max = decoded.amount_in_max;
            let mut path = decoded.path.into_iter();
            let from = path.next().unwrap();
            let to = path.next().unwrap();
            let address_to = decoded.to;
            let deadline = decoded.deadline;
            println!(
                "{:?} {:?} {:?} {:?} {:?} {:?}",
                amount_out, amount_in_max, from, to, address_to, deadline
            );
        } else if let Ok(decoded) = SwapExactETHForTokensCall::decode(&tx.input) {
            //// only implements this abi
            let amount_out_min = decoded.amount_out_min;
            let path = decoded.path;
            let address_to = decoded.to;
            let deadline = decoded.deadline;

            router::swap_eth_for_exact_tokens_router(amount_out_min, path, address_to, deadline);
        } else if let Ok(decoded) = SwapTokensForExactETHCall::decode(&tx.input) {
            let amount_out = decoded.amount_out;
            let amount_in_max = decoded.amount_in_max;
            let mut path = decoded.path.into_iter();
            let from = path.next().unwrap();
            let to = path.next().unwrap();
            let address_to = decoded.to;
            let deadline = decoded.deadline;
            println!(
                "{:?} {:?} {:?} {:?} {:?} {:?}",
                amount_out, amount_in_max, from, to, address_to, deadline
            );
        } else if let Ok(decoded) = SwapExactTokensForETHCall::decode(&tx.input) {
            let amount_in = decoded.amount_in;
            let amount_out_min = decoded.amount_out_min;
            let mut path = decoded.path.into_iter();
            let from = path.next().unwrap();
            let to = path.next().unwrap();
            let address_to = decoded.to;
            let deadline = decoded.deadline;
            print_type_of(&path);
            println!(
                "{:?} {:?} {:?} {:?} {:?} {:?}",
                amount_in, amount_out_min, from, to, address_to, deadline
            );
        } else if let Ok(decoded) = SwapETHForExactTokensCall::decode(&tx.input) {
            let amount_out = decoded.amount_out;
            let mut path = decoded.path.into_iter();
            let from = path.next().unwrap();
            let to = path.next().unwrap();
            let address_to = decoded.to;
            let deadline = decoded.deadline;

            println!(
                "{:?} {:?} {:?}  {:?} {:?}",
                amount_out, from, to, address_to, deadline
            );
        } else {
            println!("AbiError");
        }
    } else {
        println!("from:  {:?} -> {:?}", tx.from, tx.to)
    }
}
