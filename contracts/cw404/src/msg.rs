use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Binary, Coin, Empty, Uint128, Uint256};
use cw20::Cw20ReceiveMsg;
use cw721::Cw721ReceiveMsg;
use cw_ownable::{cw_ownable_execute, cw_ownable_query};
use cw_utils::Expiration;

#[cw_serde]
pub struct InstantiateMsg {
    // Name of the NFT contract
    pub name: String,
    // Symbol of the NFT contract
    pub symbol: String,
    // Decimals of erc404 token
    pub decimals: u8,
    // Supply of NFTs max
    pub total_native_supply: Uint128,

    pub minter: Option<String>,
}

// This is like Cw721ExecuteMsg but we add a Mint command for an owner
// to make this stand-alone. You will likely want to remove mint and
// use other control logic in any contract that inherits this.
#[cw_ownable_execute]
#[cw_serde]
pub enum ExecuteMsg {
    // Transfer is a base message to move a token to another account without triggering actions
    Receive(Cw20ReceiveMsg),
    ReceiveNft(Cw721ReceiveMsg),
    NativeMint{
        recipient: String,
        merkle_proof: Vec<Vec<u8>>,
        hashed_address: Vec<u8>,
    },
    SetWithdrawAddress { address: String },
    RemoveWithdrawAddress {},
    WithdrawFunds { amount: Coin },
    TransferFrom {
        owner: String,
        recipient: String,
        amount: Uint128,
    },
    Transfer {
        recipient: String,
        amount: Uint128,
    },
    TransferNft {
        recipient: String,
        token_id: Uint128,
    },
    Send {
        contract: String,
        amount: Uint128,
        msg: Binary,
    },
    SendNft {
        contract: String,
        token_id: Uint128,
        msg: Binary,
    },
    IncreaseAllowance {
        spender: String,
        amount: Uint128,
        expires: Option<Expiration>,
    },
    Approve {
        spender: String,
        token_id: Uint128,
    },
    ApproveAll {
        operator: String,
    },
    RevokeAll {
        operator: String,
    },
    SetWhitelist {
        target: String,
        state: bool,
    },
    SetBaseTokenUri {
        id: u8,
        uri: String,
    },

    Mint {
        token_id: String,
        owner: String,
        token_uri: Option<String>,
        extension: Empty,
    },
    Burn { token_id: String },
}

#[cw_ownable_query]
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(cw721::OwnerOfResponse)]
    OwnerOf {
        token_id: String,
        include_expired: Option<bool>,
    },

    #[returns(cw721::OwnerOfResponse)]
    UserInfo { address: String },

    #[returns(cw721::NumTokensResponse)]
    NumTokens {},

    #[returns(cw721::ContractInfoResponse)]
    ContractInfo {},

    #[returns(cw721::NftInfoResponse)]
    NftInfo { token_id: String },

    #[returns(cw20::BalanceResponse)]
    Balance { address: String },

    #[returns(cw20::TokenInfoResponse)]
    TokenInfo {},

    #[returns(cw721::TokensResponse)]
    Tokens {
        owner: String,
        start_after: Option<String>,
        limit: Option<u32>,
    },

    #[returns(cw721::TokensResponse)]
    AllTokens {
        start_after: Option<String>,
        limit: Option<u32>,
    },

    #[returns(cw721::AllNftInfoResponse)]
    AllNftInfo {
        token_id: String,
        // unset or false will filter out expired approvals, you must set to true to see them
        include_expired: Option<bool>,
    },

    // Return the minter
    #[returns(MinterResponse)]
    Minter {},
}

// Shows who can mint these tokens
#[cw_serde]
pub struct MinterResponse {
    pub minter: Option<String>,
}

#[cw_serde]
pub struct UserInfoResponse {
    pub owned: Vec<Uint128>,
    pub owned_index: Uint128,
    pub balances: Uint128,
}

#[cw_serde]
pub struct ContractInfoResponse {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub total_supply: Uint128,
}
