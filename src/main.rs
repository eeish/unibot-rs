use dotenv::dotenv;
use ethers::{
    abi::AbiDecode,
    prelude::*,
    providers::{Provider, Ws},
};
use eyre::Result;
use std::env;
use std::sync::Arc;

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

///abigen!(IUniswapV2Router02, "../contracts/UniswapV2Router02.json");

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    let infura_url: String = env::var("INFURA_MAINNET_WS").unwrap();
    let univ2_router: Address = env::var("UNIV2_ROUTE").unwrap().parse().unwrap();

    let client = Provider::<Ws>::connect(infura_url).await?;
    let client = Arc::new(client);

    let mut stream = client.subscribe_pending_txs().await?;

    while let Some(tx) = stream.next().await {
        let tx = client.get_transaction(tx).await?;

        if tx.is_none() {
            continue;
        } else {
            let tx = tx.unwrap();
            if tx.to.unwrap() == univ2_router {
                println!("Uni transaction founded: tx={:?}", tx.hash);

                if let decoded = SwapExactTokensForTokensCall::decode(&tx.input)
                    .and_then(SwapExactETHForTokensCall::decode(&tx.input))
                {
                    println!("no abi match");
                }

                let mut path = decoded.path.into_iter();
                let from = path.next().unwrap();
                let to = path.next().unwrap();
                println!(
                    "Swapped {} of token {} for {} of token {}",
                    decoded.amount_in, from, decoded.amount_out_min, to
                );
            } else {
                println!("from:  {:?} -> {:?}", tx.from, tx.to)
            }
        }
    }
    Ok(())
}
