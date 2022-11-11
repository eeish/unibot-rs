use crate::constants::*;
use crate::env_store::EnvStore;
use crate::utils::contract_abi::UniswapV2Router02;

use std::env::VarError;
use std::sync::Arc;

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
}
