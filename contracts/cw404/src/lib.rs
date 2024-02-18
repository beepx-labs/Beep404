// pub mod contract;
pub mod error;
mod execute;
pub mod msg;
mod query;
pub mod state;

pub use crate::error::ContractError;
pub use crate::msg::{ExecuteMsg, InstantiateMsg, MinterResponse, QueryMsg};

// pub mod entry {
//     use super::*;

//     #[cfg(not(feature = "library"))]
//     use cosmwasm_std::entry_point;
//     use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};

//     // This makes a conscious choice on the various generics used by the contract
//     #[cfg_attr(not(feature = "library"), entry_point)]
//     pub fn instantiate(
//         deps: DepsMut,
//         env: Env,
//         info: MessageInfo,
//         msg: InstantiateMsg,
//     ) -> Result<Response, ContractError> {
//         cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

//         // let tract = Cw721Contract::<Extension, Empty, Empty, Empty>::default();

//         crate::execute::instantiate(deps, env, info, msg)
//     }

//     #[cfg_attr(not(feature = "library"), entry_point)]
//     pub fn execute(
//         deps: DepsMut,
//         env: Env,
//         info: MessageInfo,
//         msg: ExecuteMsg,
//     ) -> Result<Response, ContractError> {
//         // let tract = Cw721Contract::<Extension, Empty, Empty, Empty>::default();
//         crate::execute::execute(deps, env, info, msg)
//     }

//     #[cfg_attr(not(feature = "library"), entry_point)]
//     pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
//         // let tract = Cw721Contract::<Extension, Empty, Empty, Empty>::default();
//         crate::query::query(deps, env, msg)
//     }

//     #[cfg_attr(not(feature = "library"), entry_point)]
//     pub fn migrate(deps: DepsMut, _env: Env, _msg: Empty) -> Result<Response, ContractError> {
//         // make sure the correct contract is being upgraded, and it's being
//         // upgraded from the correct version.
//         cw2::assert_contract_version(deps.as_ref().storage, CONTRACT_NAME, EXPECTED_FROM_VERSION)?;

//         // update contract version
//         cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

//         Ok(Response::default())
//     }
// }
use cosmwasm_std::Empty;

// Version info for migration
pub const CONTRACT_NAME: &str = "beepx:cw404";
pub const CONTRACT_VERSION: &str = "0.1.0";

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    crate::execute::instantiate(deps, env, info, msg)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    crate::execute::execute(deps, env, info, msg)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    crate::query::query(deps, env, msg)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: Empty) -> Result<Response, ContractError> {
    Ok(Response::default())
}

