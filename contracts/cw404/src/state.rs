use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

use cosmwasm_std::{
    Addr, BlockInfo, CustomMsg, Empty, Uint128
};

use cw721::Expiration;
use cw_storage_plus::{Index, IndexList, IndexedMap, Item, Map, MultiIndex};

use crate::msg::ContractInfoResponse;
pub const MERKLE_ROOT: Item<String> = Item::new("merkle_root");
pub const OWNER: Item<String> = Item::new("owner");
pub const WITHDRAW_ADDRESS: Item<String>= Item::new("withdraw_address");
pub const NAME: Item<String> = Item::new("name");
pub const SYMBOL: Item<String> = Item::new("symbol");
pub const TOKEN_URI: Map<String, String> = Map::new("token_uri");
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

pub const CONTRACT_INFO: Item<ContractInfoResponse> = Item::new("contract_info");

pub const TOKENS: IndexedMap<'static, &'static str, TokenInfo<Empty>, TokenIndexes<'static,Empty>> = IndexedMap::new("tokens", TokenIndexes {
    owner: MultiIndex::new(token_owner_idx, "tokens", "tokens__owner"),
});

pub const LOCKED: Map<String, bool> = Map::new("locked");


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TokenInfo<T> {
    pub owner: Addr,
    pub approvals: Vec<Approval>,
    pub token_uri: Option<String>,
    pub extension: T,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct Approval {
    pub spender: Addr,
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