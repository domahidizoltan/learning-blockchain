use crate::helper::get_env_var;

use ethers::{
    contract::ContractFactory,
    contract::ContractInstance,
    middleware::SignerMiddleware,
    prelude::Wallet,
    providers::{Http, Provider},
    signers::{LocalWallet, Signer},
    types::Address,
};
use ethers_contract::Contract;
use ethers_solc::{CompilerOutput, Solc};
use k256::Secp256k1;
use std::{path::Path, sync::Arc};

const PRIVATE_KEY: &str = "PRIVATE_KEY";
const ENDPOINT: &str = "ENDPOINT";
const CHAIN_ID: &str = "CHAIN_ID";
const CONTRACTS_PATH: &str = "CONTRACTS_PATH";

#[derive(Debug, thiserror::Error)]
pub enum EthereumClientError {
    #[error("failed to init client: {}", .0)]
    AppError(#[from] crate::AppError),

    #[error("{}", .0)]
    ClientInitError(String, #[source] Box<dyn std::error::Error>),

    #[error("could not find contract source")]
    ContractSourceNotFound(),

    #[error("could not compile contracts")]
    ContractCompilationError(#[source] Box<dyn std::error::Error>),

    #[error("could not parse address")]
    AddressParseError(#[source] Box<dyn std::error::Error>),

    #[error("could not find contract {}", .0)]
    ContractNotFound(String),

    #[error("could not create deployer")]
    DeployerCreationError(#[source] Box<dyn std::error::Error>),

    #[error("could not deploy contract")]
    ContractDeploymentError(#[source] Box<dyn std::error::Error>),
}

pub type ContractInstanceType = ContractInstance<
    Arc<SignerMiddleware<ethers_providers::Provider<Http>, Wallet<ecdsa::SigningKey<Secp256k1>>>>,
    SignerMiddleware<ethers_providers::Provider<Http>, Wallet<ecdsa::SigningKey<Secp256k1>>>,
>;

pub struct EthereumClient {
    client: std::sync::Arc<SignerMiddleware<Provider<Http>, LocalWallet>>,
    contracts: CompilerOutput,
}

impl EthereumClient {
    pub fn new() -> Result<Self, EthereumClientError> {
        let chain_id = get_env_var(CHAIN_ID)?.parse::<u64>().map_err(|e| {
            EthereumClientError::ClientInitError(
                "chain id could not be parsed".to_owned(),
                e.into(),
            )
        })?;

        let provider = get_env_var(ENDPOINT)
            .map(Provider::<Http>::try_from)?
            .map_err(|e| {
                EthereumClientError::ClientInitError(
                    "could not connect to endpoint".to_owned(),
                    e.into(),
                )
            })?;

        let wallet = get_env_var(PRIVATE_KEY)
            .map(|pk| pk[2..].parse::<LocalWallet>())?
            .map_err(|e| {
                EthereumClientError::ClientInitError(
                    "could not parse private key".to_owned(),
                    e.into(),
                )
            })?;

        let wallet_with_chain_id = wallet.with_chain_id(chain_id);
        let client = SignerMiddleware::new(provider, wallet_with_chain_id);

        EthereumClient::compile_contracts().map(|contracts| EthereumClient {
            client: std::sync::Arc::new(client),
            contracts,
        })
    }

    fn compile_contracts() -> Result<CompilerOutput, EthereumClientError> {
        let source = get_env_var(CONTRACTS_PATH)
            .map(|path| Path::new(&path).canonicalize())?
            .map_err(|_| EthereumClientError::ContractSourceNotFound())?;

        Solc::default()
            .compile_source(source)
            .map_err(|e| EthereumClientError::ContractCompilationError(e.into()))
    }

    pub fn get_client(&self) -> std::sync::Arc<SignerMiddleware<Provider<Http>, LocalWallet>> {
        self.client.clone()
    }

    pub async fn contract_from_address(
        &self,
        contract_name: &str,
        contract_address: &str,
    ) -> Result<ContractInstanceType, EthereumClientError> {
        let address = contract_address
            .parse::<Address>()
            .map_err(|e| EthereumClientError::AddressParseError(e.into()))?;

        let (abi, _bytecode, _runtime_bytecode) = match self.contracts.find(contract_name) {
            Some(compiled) => compiled.into_parts_or_default(),
            None => {
                return Err(EthereumClientError::ContractNotFound(
                    contract_name.to_string(),
                ))
            }
        };

        Ok(Contract::new(address, abi, self.client.clone()))
    }

    pub async fn deploy_contract(
        &self,
        contract_name: &str,
    ) -> Result<ContractInstanceType, EthereumClientError> {
        let (abi, bytecode, _runtime_bytecode) = match self.contracts.find(contract_name) {
            Some(compiled) => compiled.into_parts_or_default(),
            None => {
                return Err(EthereumClientError::ContractNotFound(
                    contract_name.to_string(),
                ))
            }
        };

        let factory = ContractFactory::new(abi, bytecode, self.client.clone());

        let contract = factory
            .deploy(())
            .map_err(|e| EthereumClientError::DeployerCreationError(e.into()))?
            .confirmations(0usize)
            .send()
            .await;

        contract.map_err(|e| EthereumClientError::ContractDeploymentError(e.into()))
    }
}
