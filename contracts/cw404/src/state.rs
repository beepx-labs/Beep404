use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

use cosmwasm_std::{
    to_json_binary, Addr, Binary, BlockInfo, CosmosMsg, CustomMsg, StdResult, Uint128, WasmMsg,
};

use cw721::{ContractInfoResponse, Expiration};
use cw_storage_plus::{Index, IndexList, IndexedMap, Item, Map, MultiIndex};

pub const OWNER: Item<String> = Item::new("owner");

pub const NAME: Item<String> = Item::new("name");
pub const SYMBOL: Item<String> = Item::new("symbol");
pub const BASE_TOKEN_URI: Item<String> = Item::new("base_token_uri");
pub const DECIMALS: Item<u8> = Item::new("decimals");
pub const TOTAL_SUPPLY: Item<Uint128> = Item::new("total_supply");
pub const MINTED: Item<Uint128> = Item::new("minted");
pub const WHITELIST: Map<String, bool> = Map::new("whitelist");
/// Approval in native representation
pub const GET_APPROVED: Map<String, String> = Map::new("get_approved");
/// Allowance of user in fractional representation
pub const ALLOWANCE: Map<(String, String), Uint128> = Map::new("cw20_allowance");
pub const BALANCES: Map<&Addr, Uint128> = Map::new("balance");
/// Owner of a tokenID in native representation
pub const OWNER_OF: Map<String, String> = Map::new("owner_of");
/// Array of owned ids in native representation
pub const OWNED: Map<String, Vec<Uint128>> = Map::new("owned");
/// @dev Tracks indices for the _owned mapping
pub const OWNED_INDEX: Map<String, Uint128> = Map::new("owned_index");
pub const APPROVED_FOR_ALL: Map<(String, String), bool> = Map::new("approved_for_all");

/// Additional features
/// @dev prevents being burnt due to transfers made in mistake
pub const LOCKED: Map<String, bool> = Map::new("locked");


pub struct Cw721Contract<'a, T, C, E, Q>
where
    T: Serialize + DeserializeOwned + Clone,
    Q: CustomMsg,
    E: CustomMsg,
{
    pub contract_info: Item<'a, ContractInfoResponse>,
    pub token_count: Item<'a, u64>,
    /// Stored as (granter, operator) giving operator full control over granter's account
    pub operators: Map<'a, (&'a Addr, &'a Addr), Expiration>,
    pub tokens: IndexedMap<'a, &'a str, TokenInfo<T>, TokenIndexes<'a, T>>,
    pub withdraw_address: Item<'a, String>,

    pub(crate) _custom_response: PhantomData<C>,
    pub(crate) _custom_query: PhantomData<Q>,
    pub(crate) _custom_execute: PhantomData<E>,
}

impl<T, C, E, Q> Default for Cw721Contract<'static, T, C, E, Q>
where
    T: Serialize + DeserializeOwned + Clone,
    E: CustomMsg,
    Q: CustomMsg,
{
    fn default() -> Self {
        Self::new(
            "nft_info",
            "num_tokens",
            "operators",
            "tokens",
            "tokens__owner",
            "withdraw_address",
        )
    }
}

impl<'a, T, C, E, Q> Cw721Contract<'a, T, C, E, Q>
where
    T: Serialize + DeserializeOwned + Clone,
    E: CustomMsg,
    Q: CustomMsg,
{
    fn new(
        contract_key: &'a str,
        token_count_key: &'a str,
        operator_key: &'a str,
        tokens_key: &'a str,
        tokens_owner_key: &'a str,
        withdraw_address_key: &'a str,
    ) -> Self {
        let indexes = TokenIndexes {
            owner: MultiIndex::new(token_owner_idx, tokens_key, tokens_owner_key),
        };
        Self {
            contract_info: Item::new(contract_key),
            token_count: Item::new(token_count_key),
            operators: Map::new(operator_key),
            tokens: IndexedMap::new(tokens_key, indexes),
            withdraw_address: Item::new(withdraw_address_key),
            _custom_response: PhantomData,
            _custom_execute: PhantomData,
            _custom_query: PhantomData,
        }
    }

    // pub fn token_count(&self, storage: &dyn Storage) -> StdResult<u64> {
    //     Ok(self.token_count.may_load(storage)?.unwrap_or_default())
    // }

    // pub fn increment_tokens(&self, storage: &mut dyn Storage) -> StdResult<u64> {
    //     let val = self.token_count(storage)? + 1;
    //     self.token_count.save(storage, &val)?;
    //     Ok(val)
    // }

    // pub fn decrement_tokens(&self, storage: &mut dyn Storage) -> StdResult<u64> {
    //     let val = self.token_count(storage)? - 1;
    //     self.token_count.save(storage, &val)?;
    //     Ok(val)
    // }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TokenInfo<T> {
    /// The owner of the newly minted NFT
    pub owner: Addr,
    /// Approvals are stored here, as we clear them all upon transfer and cannot accumulate much
    pub approvals: Vec<Approval>,

    /// Universal resource identifier for this NFT
    /// Should point to a JSON file that conforms to the ERC721
    /// Metadata JSON Schema
    pub token_uri: Option<String>,

    /// You can add any custom metadata here when you extend cw721-base
    pub extension: T,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct Approval {
    /// Account that can transfer/send the token
    pub spender: Addr,
    /// When the Approval expires (maybe Expiration::never)
    pub expires: Expiration,
}

impl Approval {
    pub fn is_expired(&self, block: &BlockInfo) -> bool {
        self.expires.is_expired(block)
    }
}

pub struct TokenIndexes<'a, T>
where
    T: Serialize + DeserializeOwned + Clone,
{
    pub owner: MultiIndex<'a, Addr, TokenInfo<T>, String>,
}

impl<'a, T> IndexList<TokenInfo<T>> for TokenIndexes<'a, T>
where
    T: Serialize + DeserializeOwned + Clone,
{
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<TokenInfo<T>>> + '_> {
        let v: Vec<&dyn Index<TokenInfo<T>>> = vec![&self.owner];
        Box::new(v.into_iter())
    }
}

pub fn token_owner_idx<T>(_pk: &[u8], d: &TokenInfo<T>) -> Addr {
    d.owner.clone()
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub struct Cw20ReceiveMsg {
    pub sender: String,
    pub amount: Uint128,
    pub msg: Binary,
}

// This is just a helper to properly serialize the above message
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
enum ReceiverExecuteMsg {
    Receive(Cw20ReceiveMsg),
}

impl Cw20ReceiveMsg {
    /// serializes the message
    pub fn into_binary(self) -> StdResult<Binary> {
        let msg = ReceiverExecuteMsg::Receive(self);
        to_json_binary(&msg)
    }

    /// creates a cosmos_msg sending this struct to the named contract
    pub fn into_cosmos_msg<T: Into<String>>(self, contract_addr: T) -> StdResult<CosmosMsg> {
        let msg = self.into_binary()?;
        let execute = WasmMsg::Execute {
            contract_addr: contract_addr.into(),
            msg,
            funds: vec![],
        };
        Ok(execute.into())
    }
}
