use crate::constants::*;
use crate::env_store::EnvStore;
use crate::utils::contract_abi::UniswapV2Router02;
use crate::utils::univ2;

use ethers::prelude::*;
use ethers::utils::keccak256;
use std::env::VarError;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use eyre::Result;

use ethers::{
    prelude::{
        k256::ecdsa::SigningKey, ContractError, SignerMiddleware, SubscriptionStream, Transaction,
        TxHash,
    },
    providers::{Middleware, Provider, ProviderError, Ws, WsClientError},
    signers::{LocalWallet, Signer, Wallet},
    types::Address,
};

use hex::FromHexError;

#[derive(Debug)]
pub enum UniswapV2Error {
    ClientError(WsClientError),
    ContractError(ContractError<UniswapV2Middleware>),
    HexError(FromHexError),
    IntoError(String),
    ProviderError(ProviderError),
    SigningError(ethers::core::k256::ecdsa::Error),
    VarError(VarError),
}

pub type UniswapV2Middleware = SignerMiddleware<Provider<Ws>, Wallet<SigningKey>>;

pub struct UniswapV2Client {
    envstore: EnvStore,
    provider: Arc<UniswapV2Middleware>,
    router: UniswapV2Router02<UniswapV2Middleware>,
}

abigen!(
    IUniswapV2Pair,
    r#"[
        function getReserves() external view returns (uint112 reserve0, uint112 reserve1, uint32 blockTimestampLast)
    ]"#,
);

fn time() -> u64 {
    let time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    return time.as_secs();
}

impl<'a> UniswapV2Client {
    pub async fn new(env: EnvStore) -> Result<Self, UniswapV2Error> {
        let provider = Provider::new(
            Ws::connect(format!("{}", env.get_ws_url()))
                .await
                .map_err(|e| UniswapV2Error::ClientError(e))?,
        );

        let chain_id = provider
            .get_chainid()
            .await
            .map_err(|e| UniswapV2Error::ProviderError(e))?;

        let wallet = LocalWallet::from(
            SigningKey::from_bytes(env.get_private_key())
                .map_err(|e| UniswapV2Error::SigningError(e))?,
        )
        .with_chain_id(chain_id.as_u64());

        let provider = Arc::new(SignerMiddleware::new(provider.clone(), wallet));

        Ok(UniswapV2Client {
            envstore: env,
            router: UniswapV2Router02::new(
                UNIV2_ROUTER02_ADDRESS.parse::<Address>().unwrap(),
                provider.clone(),
            ),
            provider: provider,
        })
    }

    pub async fn get_pending_txs(&self) -> SubscriptionStream<'_, Ws, TxHash> {
        self.provider.subscribe_pending_txs().await.unwrap()
    }

    pub async fn get_transaction(&self, tx: TxHash) -> Option<Transaction> {
        self.provider.get_transaction(tx).await.unwrap()
    }

    pub async fn swap_eth_for_exact_tokens(
        &self,
        user_amount_in: U256,
        amount_out_min: U256,
        path: Vec<Address>,
        to: Address,
        deadline: U256,
    ) {
        if U256::from(time()) > deadline {
            println!("deadline exceeded, can't mev op");
            return;
        }

        let user_min_recv = self
            .get_univ2_exact_weth_token_min_recv(amount_out_min, &path)
            .await;

        let weth = path[0];
        let token = path[1];

        let pair_to_sandwich = self.get_uni_pair_address(weth, token);
        let (weth_reserve, token_reserve) =
            self.get_univ2_reserve(pair_to_sandwich, weth, token).await;
    }

    pub async fn get_univ2_exact_weth_token_min_recv(
        &self,
        amount_out_min: U256,
        path: &Vec<Address>,
    ) -> U256 {
        let user_min_recv = amount_out_min;

        for index in (path.capacity() - 1)..1 {
            let from = path[index];
            let to = path[index - 1];

            let pair_address = self.get_uni_pair_address(from, to);
            let (reserve_from, reserve_to) = self.get_univ2_reserve(pair_address, from, to).await;
            let (a_amount_in, new_reserve_from, new_reserve_to) = univ2::get_univ2_data_given_out(
                user_min_recv,
                U256::from(reserve_from),
                U256::from(reserve_to),
            );

            let user_min_recv = a_amount_in;
        }

        return user_min_recv;
    }

    pub fn get_uni_pair_address(&self, from: Address, to: Address) -> Address {
        let (from, to) = univ2::sort_token(from, to);

        //// TODO remove hard code uniswapv2 factory address
        let factory = "5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f"
            .parse::<Address>()
            .unwrap();

        //// TODO remove hard code init code
        let init_code_hash =
            hex::decode("96e8ac4277198ff8b6f785478aa9a39f403cb768dd02cbee326c3e7da348845f")
                .unwrap();

        let mut extend_byte_array = from.as_bytes().to_vec();
        let to_byte_array = to.as_bytes().to_vec();
        extend_byte_array.extend(to_byte_array);

        //// Attention here, ethers-rs: abi encoding not work
        //// let input = abi::encode(&vec![Token::Address(from), Token::Address(to)]);
        let salt = keccak256(&extend_byte_array);

        let salt2 = hex::decode("4aafb64a36177dc82e7ace74cf60cc655659bc049da9533b5f7a6881bea995c6")
            .unwrap();

        let pair_address = ethers::core::utils::get_create2_address_from_hash(
            factory,
            salt.to_vec(),
            init_code_hash.to_vec(),
        );

        pair_address
    }

    pub async fn get_univ2_reserve(
        &self,
        pair_address: Address,
        from: Address,
        to: Address,
    ) -> (u128, u128) {
        let (from_, to_) = univ2::sort_token(from, to);

        let pair = IUniswapV2Pair::new(pair_address, Arc::clone(&self.provider));
        let (reserve0, reserve1, _timestamp) = pair.get_reserves().call().await.unwrap();

        if from == from_ {
            (reserve0, reserve1)
        } else {
            (reserve1, reserve0)
        }
    }
}
