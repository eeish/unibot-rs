use std::{mem::swap, sync::Arc};

use crate::env_store::EnvStore;
use crate::utils::contract_abi::UniswapV2Router02;

use ethers::{
    prelude::{k256::ecdsa::SigningKey, ContractError, SignerMiddleware},
    providers::{Middleware, Provider, Ws},
    signers::{LocalWallet, Signer, Wallet},
    types::{Address, U256},
};

pub type UniswapV2Middleware = SignerMiddleware<Provider<Ws>, Wallet<SigningKey>>;

pub struct UniswapV2Client {
    envstore: EnvStore,
    provider: Arc<UniswapV2Middleware>,
    router: UniswapV2Router02<UniswapV2Middleware>,
}
