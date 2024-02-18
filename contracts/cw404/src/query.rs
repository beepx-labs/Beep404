use cw20::{BalanceResponse, TokenInfoResponse};

use cosmwasm_std::{to_json_binary, Addr, Binary, BlockInfo, Deps, Empty, Env, Order, StdResult, Uint128};

use cw721::{AllNftInfoResponse, NftInfoResponse, NumTokensResponse, OwnerOfResponse, TokensResponse};
use cw_storage_plus::Bound;

use crate::msg::{ContractInfoResponse, MinterResponse, QueryMsg, UserInfoResponse};
use crate::state::{
    Approval, TokenInfo, BALANCES, CONTRACT_INFO, DECIMALS, LOCKED, MINTED, NAME, OWNED, OWNED_INDEX, OWNER_OF, SYMBOL, TOKENS, TOKEN_URI, TOTAL_SUPPLY
};

const DEFAULT_LIMIT: u32 = 10;
const MAX_LIMIT: u32 = 1000;

fn contract_info(deps: Deps) -> StdResult<ContractInfoResponse> {
    let contract_info = CONTRACT_INFO.load(deps.storage)?;
    Ok(contract_info)
}

fn num_tokens(deps: Deps) -> StdResult<NumTokensResponse> {
    let count = MINTED.may_load(deps.storage)?.unwrap_or(Uint128::zero());
    Ok(NumTokensResponse {
        count: count.u128() as u64,
    })
}

fn nft_info(deps: Deps, token_id: String) -> StdResult<NftInfoResponse> {
    let base_uri = TOKEN_URI.load(deps.storage, token_id)?; // TODO: remove this line
    Ok(NftInfoResponse {
        token_uri: Some(base_uri),
        extension: None,
    })
}

fn owner_of(
    deps: Deps,
    _env: Env,
    token_id: String,
    _include_expired: bool,
) -> StdResult<OwnerOfResponse> {
    let owner = OWNER_OF
        .may_load(deps.storage, token_id)?
        .unwrap_or("".to_string());
    Ok(OwnerOfResponse {
        owner,
        approvals: vec![],
    })
}

fn user_info(deps: Deps, _env: Env, address: String) -> StdResult<UserInfoResponse> {
    let owned = OWNED.may_load(deps.storage, address.clone())?.unwrap_or(vec![]);
    let owned_index = OWNED_INDEX
        .may_load(deps.storage, address.clone())?
        .unwrap_or(Uint128::zero());
    let balances = BALANCES
        .may_load(deps.storage, &deps.api.addr_validate(&address)?)?
        .unwrap_or(Uint128::zero());
    Ok(UserInfoResponse {
        owned,
        owned_index,
        balances,
    })
}

fn is_locked(deps: Deps, _env: Env, token_id: String) -> StdResult<bool> {
    let locked = LOCKED.may_load(deps.storage, token_id)?.unwrap_or(false);
    Ok(locked)
}

pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Minter {} => to_json_binary(&minter(deps)?),
        QueryMsg::ContractInfo {} => to_json_binary(&contract_info(deps)?),
        QueryMsg::Balance { address } => {
            let user = deps.api.addr_validate(&address)?;
            let balance = BALANCES
                .may_load(deps.storage, &user)?
                .unwrap_or(Uint128::zero());

            to_json_binary(&BalanceResponse { balance })
        }
        QueryMsg::TokenInfo {} => {
            let name = NAME.load(deps.storage)?;
            let symbol = SYMBOL.load(deps.storage)?;
            let decimals = DECIMALS.load(deps.storage)?;
            let total_supply = TOTAL_SUPPLY.load(deps.storage)?;
            to_json_binary(&TokenInfoResponse {
                name,
                symbol,
                decimals,
                total_supply,
            })
        }
        QueryMsg::NftInfo { token_id } => to_json_binary(&nft_info(deps, token_id)?),
        QueryMsg::OwnerOf {
            token_id,
            include_expired,
        } => to_json_binary(&owner_of(
            deps,
            env,
            token_id,
            include_expired.unwrap_or(false),
        )?),
        QueryMsg::UserInfo { address } => to_json_binary(&user_info(deps, env, address)?),
        QueryMsg::AllNftInfo {
            token_id,
            include_expired,
        } => to_json_binary(&all_nft_info(
            deps,
            env,
            token_id,
            include_expired.unwrap_or(false),
        )?),
        QueryMsg::NumTokens {} => to_json_binary(&num_tokens(deps)?),
        QueryMsg::Ownership {  } => to_json_binary(&ownership(deps)?),
        QueryMsg::Tokens {
            owner,
            start_after,
            limit,
        } => to_json_binary(&tokens(deps, owner, start_after, limit)?),
        QueryMsg::AllTokens { start_after, limit } => {
            to_json_binary(&all_tokens(deps, start_after, limit)?)
        }

    }
}

fn all_nft_info(
    deps: Deps,
    env: Env,
    token_id: String,
    include_expired: bool,
) -> StdResult<AllNftInfoResponse> {
    let info = TOKENS.load(deps.storage, &token_id)?;
    Ok(AllNftInfoResponse {
        access: OwnerOfResponse {
            owner: info.owner.to_string(),
            approvals: humanize_approvals(&env.block, &info, include_expired),
        },
        info: NftInfoResponse {
            token_uri: info.token_uri,
            extension: None,
        },
    })
}

fn all_tokens(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<TokensResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = start_after.map(|s| Bound::ExclusiveRaw(s.into()));

    let tokens: StdResult<Vec<String>> = TOKENS
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| item.map(|(k, _)| k))
        .collect();

    Ok(TokensResponse { tokens: tokens? })
}

fn tokens(
    deps: Deps,
    owner: String,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<TokensResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = start_after.map(|s| Bound::ExclusiveRaw(s.into()));

    let owner_addr = deps.api.addr_validate(&owner)?;
    let tokens: Vec<String> =TOKENS
        .idx
        .owner
        .prefix(owner_addr)
        .keys(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .collect::<StdResult<Vec<_>>>()?;

    Ok(TokensResponse { tokens })
}

pub fn minter(deps: Deps) -> StdResult<MinterResponse> {
    let minter = cw_ownable::get_ownership(deps.storage)?
        .owner
        .map(|a| a.into_string());

    Ok(MinterResponse { minter })
}

pub fn ownership(deps: Deps) -> StdResult<cw_ownable::Ownership<Addr>> {
    cw_ownable::get_ownership(deps.storage)
}

fn humanize_approvals<T>(
    block: &BlockInfo,
    info: &TokenInfo<T>,
    include_expired: bool,
) -> Vec<cw721::Approval> {
    info.approvals
        .iter()
        .filter(|apr| include_expired || !apr.is_expired(block))
        .map(humanize_approval)
        .collect()
}

fn humanize_approval(approval: &Approval) -> cw721::Approval {
    cw721::Approval {
        spender: approval.spender.to_string(),
        expires: approval.expires,
    }
}
