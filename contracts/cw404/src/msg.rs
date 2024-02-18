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

    // The minter is the only one who can create new NFTs.
    // This is designed for a base NFT that is controlled by an external program
    // or contract. You will likely replace this with custom logic in custom NFTs
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
    /// Removes the withdraw address, so fees are sent to the contract. Only owner can call this.
    RemoveWithdrawAddress {},
    /// Withdraw from the contract to the given address. Anyone can call this,
    /// which is okay since withdraw address has been set by owner.
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
    // Allows operator to transfer / send the token from the owner's account.
    // If expiration is set, then this allowance has a time/height limit
    Approve {
        spender: String,
        token_id: Uint128,
    },
    // Allows operator to transfer / send any token from the owner's account.
    // If expiration is set, then this allowance has a time/height limit
    ApproveAll {
        operator: String,
    },
    // Remove previously granted ApproveAll permission
    RevokeAll {
        operator: String,
    },
    GenerateNftEvent {
        sender: String,
        recipient: String,
        token_id: Uint128,
    },
    GenerateNftMintEvent {
        sender: String,
        recipient: String,
        token_id: Uint128,
    },
    GenerateNftBurnEvent {
        sender: String,
        token_id: Uint128,
    },
    SetWhitelist {
        target: String,
        state: bool,
    },
    SetLock {
        token_id: Uint128,
        state: bool,
    },
    SetBaseTokenUri {
        id: u8,
        uri: String,
    },

    Mint {
        /// Unique ID of the NFT
        token_id: String,
        /// The owner of the newly minter NFT
        owner: String,
        /// Universal resource identifier for this NFT
        /// Should point to a JSON file that conforms to the ERC721
        /// Metadata JSON Schema
        token_uri: Option<String>,
        /// Any custom extension used by this contract
        extension: Empty,
    },
    Burn { token_id: String },
}

#[cw_ownable_query]
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    // Return the owner of the given token, error if token does not exist
    #[returns(cw721::OwnerOfResponse)]
    OwnerOf {
        token_id: String,
        // unset or false will filter out expired approvals, you must set to true to see them
        include_expired: Option<bool>,
    },
    #[returns(bool)]
    IsLocked { token_id: String },
    #[returns(cw721::OwnerOfResponse)]
    UserInfo { address: String },
    // Return operator that can access all of the owner's tokens.
    // #[returns(cw721::ApprovalResponse)]
    // Approval {
    //     token_id: String,
    //     spender: String,
    //     include_expired: Option<bool>,
    // },
    // Return approvals that a token has
    // #[returns(cw721::ApprovalsResponse)]
    // Approvals {
    //     token_id: String,
    //     include_expired: Option<bool>,
    // },
    // Return approval of a given operator for all tokens of an owner, error if not set
    // #[returns(cw721::OperatorResponse)]
    // Operator {
    //     owner: String,
    //     operator: String,
    //     include_expired: Option<bool>,
    // },
    // List all operators that can access all of the owner's tokens
    // #[returns(cw721::OperatorsResponse)]
    // AllOperators {
    //     owner: String,
    //     // unset or false will filter out expired items, you must set to true to see them
    //     include_expired: Option<bool>,
    //     start_after: Option<String>,
    //     limit: Option<u32>,
    // },
    // Total number of tokens issued
    #[returns(cw721::NumTokensResponse)]
    NumTokens {},

    // With MetaData Extension.
    // Returns top-level metadata about the contract
    #[returns(cw721::ContractInfoResponse)]
    ContractInfo {},
    // With MetaData Extension.
    // Returns metadata about one particular token, based on *ERC721 Metadata JSON Schema*
    // but directly from the contract
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
    // With MetaData Extension.
    // Returns the result of both `NftInfo` and `OwnerOf` as one query as an optimization
    // for clients
    #[returns(cw721::AllNftInfoResponse)]
    AllNftInfo {
        token_id: String,
        // unset or false will filter out expired approvals, you must set to true to see them
        include_expired: Option<bool>,
    },

    // With Enumerable extension.
    // Returns all tokens owned by the given address, [] if unset.
    // #[returns(cw721::TokensResponse)]
    // Tokens {
    //     owner: String,
    //     start_after: Option<String>,
    //     limit: Option<u32>,
    // },
    // With Enumerable extension.
    // Requires pagination. Lists all token_ids controlled by the contract.
    // #[returns(cw721::TokensResponse)]
    // AllTokens {
    //     start_after: Option<String>,
    //     limit: Option<u32>,
    // },

    // Return the minter
    #[returns(MinterResponse)]
    Minter {},
    // Extension query
    // #[returns(())]
    // Extension { msg: Q },

    // #[returns(Option<String>)]
    // GetWithdrawAddress {},
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
