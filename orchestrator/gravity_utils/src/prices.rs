use clarity::address::Address as EthAddress;
use clarity::Uint256;
use web30::amm::DAI_CONTRACT_ADDRESS;
use web30::amm::WETH_CONTRACT_ADDRESS;
use web30::client::Web3;
use web30::jsonrpc::error::Web3Error;
use clarity::{
    abi::{encode_call, Token},
    constants::{TT160M1, TT24M1},
    Address, PrivateKey,
};
use lazy_static::lazy_static;
use std::collections::HashMap;

lazy_static! {
    pub static ref UNISWAP_ROUTER02_ADDRESS: Address = 
    Address::parse_and_validate("0x9Ac64Cc6e4415144C455BD8E4837Fea55603e5c3").unwrap();
}

#[allow(clippy::too_many_arguments)]
pub async fn get_uniswap_price(
    web3: &Web3,
    caller_address: Address,
    path: Vec<Address>,
    amount: Uint256,                               // The amount of tokens offered up
    uniswap_router: Option<Address>, // The default router will be used if none is provided
) -> Result<Uint256, Web3Error> {
    let router = uniswap_router.unwrap_or(*UNISWAP_ROUTER02_ADDRESS);

    let mut path_tokens = vec![];
    for token in path.iter() {
        path_tokens.push(Token::Address(token.clone()));
    }

    let tokens: [Token; 2] = [
        Token::Uint(amount),
        Token::Dynamic(path_tokens),
    ];
    debug!("tokens is  {:?}", tokens);
    let payload = encode_call(
        "getAmountsOut(uint256,address[])",
        &tokens,
    )?;
    let result = web3
        .simulate_transaction(router, 0u8.into(), payload, caller_address, None)
        .await?;
    Ok(Uint256::from_bytes_be(match result.get(result.len() - 32..result.len()) {
        Some(val) => val,
        None => {
            return Err(Web3Error::ContractCallError(
                "Bad response from swap price".to_string(),
            ))
        }
    }))
}
/// utility function, gets the price of a given ERC20 token in uniswap in WETH given the erc20 address and amount
pub async fn get_weth_price(
    token: EthAddress,
    amount: Uint256,
    pubkey: EthAddress,
    web3: &Web3,
    paths: &HashMap<EthAddress, Vec<EthAddress>>,
    weth_contract_address: EthAddress,
    router_contract_address: EthAddress,
) -> Result<Uint256, Web3Error> {
    if token == *WETH_CONTRACT_ADDRESS {
        return Ok(amount);
    } else if amount == 0u8.into() {
        return Ok(0u8.into());
    }

    // TODO: Make sure the market is not too thin
    let price = get_uniswap_price(
            web3,
            pubkey,
            paths.get(&token).unwrap_or(&vec![token, weth_contract_address]).clone(),
            amount.clone(),
            Some(router_contract_address),
        )
        .await;
    price
}