use std::str::FromStr;

use cosmwasm_std::{
    to_json_binary, Addr, BankMsg, Binary, Coin, CosmosMsg, DepsMut, Empty, Env, MessageInfo, Response, StdResult, Storage, Uint128, WasmMsg
};
use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg};
use cw721::{Cw721ExecuteMsg, Cw721ReceiveMsg};
use cw_ownable::get_ownership;
use sha3::{Digest, Keccak256};

use crate::error::ContractError;
use crate::msg::{ContractInfoResponse, ExecuteMsg, InstantiateMsg};
use crate::state::{
    TokenInfo, ALLOWANCE, APPROVED_FOR_ALL, BALANCES, CONTRACT_INFO, DECIMALS, GET_APPROVED, LOCKED, MERKLE_ROOT, MINTED, NAME, OWNED, OWNED_INDEX, OWNER, OWNER_OF, SYMBOL, TOKENS, TOKEN_URI, TOTAL_SUPPLY, WHITELIST, WITHDRAW_ADDRESS
};

pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let total_supply = msg.total_native_supply.u128() * ((10u128).pow(msg.decimals.into()));
    DECIMALS.save(deps.storage, &msg.decimals)?;
    TOTAL_SUPPLY.save(deps.storage, &Uint128::from(total_supply))?;
    MINTED.save(deps.storage, &Uint128::zero())?;
    NAME.save(deps.storage, &msg.name)?;
    SYMBOL.save(deps.storage, &msg.symbol)?;
    // MERKLE_ROOT.save(deps.storage, &"21afb4d04947e9028f7f7c6814be583f92292c032011e0ddf5b443035b699489".to_string())?;
    OWNER.save(deps.storage, &info.sender.to_string())?;

    BALANCES.save(deps.storage, &info.sender, &Uint128::from(total_supply))?;

    let contract_info = ContractInfoResponse {
        name: msg.name,
        symbol: msg.symbol,
        decimals: msg.decimals,
        total_supply: msg.total_native_supply,
    };

    CONTRACT_INFO.save(deps.storage, &contract_info)?;

    let owner = match msg.minter {
        Some(owner) => deps.api.addr_validate(&owner)?,
        None => info.sender,
    };
    cw_ownable::initialize_owner(deps.storage, deps.api, Some(owner.as_ref()))?;

    Ok(Response::new()
        .add_attribute("action", "mint")
        .add_attribute("to", owner.to_string())
        .add_attribute("amount", total_supply.to_string()))
}

pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Mint {
            token_id,
            owner,
            token_uri,
            extension,
        } => mint(deps, info, token_id, owner, token_uri, extension),
        ExecuteMsg::Burn { token_id } => burn(deps, env, info, token_id),
        ExecuteMsg::Receive(msg) => try_receive_cw20(deps,env, info, msg),
        ExecuteMsg::NativeMint{recipient, merkle_proof, hashed_address}=> try_receive_native_tokens(deps, env,info, recipient, merkle_proof, hashed_address),
        ExecuteMsg::ReceiveNft(msg) => try_receive_cw721(deps, env, info, msg),
        ExecuteMsg::Approve { spender, token_id } => approve(deps, env, info, spender, token_id),
        ExecuteMsg::ApproveAll { operator } => approve_all(deps, env, info, operator),
        ExecuteMsg::IncreaseAllowance {
            spender,
            amount,
            expires: _expires,
        } => approve(deps, env, info, spender, amount),
        ExecuteMsg::RevokeAll { operator } => revoke_all(deps, env, info, operator),
        // This is the default implementation in erc404
        ExecuteMsg::TransferFrom {
            owner,
            recipient,
            amount,
        } => transfer_from(deps, env, info, owner, recipient, amount),
        // This is the default implementation in erc404
        ExecuteMsg::Transfer { recipient, amount } => transfer(
            deps,
            env,
            info.clone(),
            info.sender.to_string(),
            recipient,
            amount,
        ),
        // Added to ensure compatibility with cw721
        ExecuteMsg::TransferNft {
            recipient,
            token_id,
        } => transfer_from(
            deps,
            env,
            info.clone(),
            info.sender.to_string(),
            recipient,
            token_id,
        ),
        // Added to ensure compatibility with cw20
        ExecuteMsg::Send {
            contract,
            amount,
            msg,
        } => send(
            deps,
            env,
            info.clone(),
            info.sender.to_string(),
            contract,
            msg,
            amount,
        ),
        // Added to ensure compatibility with cw721
        ExecuteMsg::SendNft {
            contract,
            token_id,
            msg,
        } => send(
            deps,
            env,
            info.clone(),
            info.sender.to_string(),
            contract,
            msg,
            token_id,
        ),
        // Additional feature added by dojo team to prevent accidental burning of CW721 tokens that a user may wish to keep (as cw20 transfers might burn tokens)
        ExecuteMsg::SetLock { token_id, state } => set_lock(deps, env, info, token_id, state),

        // Event functions
        ExecuteMsg::GenerateNftEvent {
            sender,
            recipient,
            token_id,
        } => generate_nft_event(deps, env, info.clone(), sender, recipient, token_id),
        ExecuteMsg::GenerateNftMintEvent {
            sender,
            recipient,
            token_id,
        } => generate_nft_mint_event(deps, env, info.clone(), sender, recipient, token_id),
        ExecuteMsg::GenerateNftBurnEvent { sender, token_id } => {
            generate_nft_burn_event(deps, env, info.clone(), sender, token_id)
        }

        ExecuteMsg::SetWithdrawAddress { address } => {
            set_withdraw_address(deps, &info.sender, address)
        }
        ExecuteMsg::RemoveWithdrawAddress {} => {
            remove_withdraw_address(deps.storage, &info.sender)
        }
        ExecuteMsg::WithdrawFunds { amount } => withdraw_funds(deps.storage, &amount),
        // Auxillary functions
        ExecuteMsg::SetWhitelist { target, state } => set_whitelist(deps, env, info, target, state),
        ExecuteMsg::SetBaseTokenUri {id,  uri } => set_base_token_uri(deps, env, info, id, uri),
        ExecuteMsg::UpdateOwnership(action) => update_ownership(deps, env, info, action),
    }
}

fn try_receive_native_tokens(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    recipient: String,  
    _merkle_proof: Vec<Vec<u8>>,
    _hashed_address: Vec<u8>
) -> Result<Response, ContractError> {
    // let mut current_hash = hashed_address;
    
    // // Iterate over the proof, hashing the current hash with each proof element
    // for p in merkle_proof {
    //     let mut hasher = Keccak256::default();
        
    //     // Depending on the order, hash the concatenation of the current hash and the proof element
    //     // Adjust this logic based on how your tree is structured (e.g., sorting pairs)
    //     hasher.update(&[current_hash, p].concat());
        
    //     current_hash = hasher.finalize().to_vec();
    // }

    // // Retrieve the stored Merkle root
    // let stored_root = MERKLE_ROOT.load(deps.storage)?;

    // // Convert the current hash (calculated root) to a comparable format
    // let calculated_root = hex::encode(current_hash);

    // if stored_root != calculated_root {
    //     return Err(ContractError::NotWhitelisted{});
    // }

    let received_amount = info.funds.iter().find(|coin| coin.denom == "usei").map_or(Uint128::zero(), |coin| coin.amount);
    let decimals = get_unit(deps.storage).unwrap();
    let required_amount=Uint128::new(50000000);
    let token_amount = Uint128::new(1);
    if received_amount.eq(&Uint128::from_str(&required_amount.to_string())?){
        // let amount_to_update = 
        let owner= get_ownership(deps.storage).unwrap().owner.unwrap();
        // decrement_balance(deps.storage, &owner, Uint128::new(1))?;
        // increment_balance(deps.storage, &info.sender, Uint128::new(1))?;

        // let cw20_transfer_msg = Cw20ExecuteMsg::Transfer {
        //     recipient: recipient,
        //     amount: token_amount,
        // };

        // let wasm_msg = WasmMsg::Execute {
        //     contract_addr: "cw20_token_contract_address_here".to_string(),
        //     msg: to_json_binary(&cw20_transfer_msg)?,
        //     funds: vec![],
        // };

        let response = _transfer(
            deps,
            env,
            info.clone(),
            owner.to_string(),
            recipient,
            token_amount.checked_mul(decimals).expect("invalid"),
            "transfer_from".to_string(),
        )
        .unwrap();
    
        // Ok(Response::new()
        //     .add_message(CosmosMsg::Wasm(wasm_msg))
        //     .add_attribute("action", "exchange_native_for_cw20")
        //     .add_attribute("sender", info.sender.to_string())
        //     .add_attribute("amount_cw20_sent", token_amount.to_string()))
        Ok(response.add_attribute("by", info.sender))
        // Ok(Response::new())
    } else {
        return Err(ContractError::IncorrectAmount {});
    }

}

fn increment_balance (storage: &mut dyn Storage, addr: &Addr, amount: Uint128) -> Result<Response,ContractError> {
    let inc = amount.checked_mul(get_unit(storage).unwrap()).expect("invalid");
    BALANCES.update(storage, addr, |balance: Option<Uint128>| -> Result<Uint128, ContractError>{
        let new_balance = balance.unwrap_or_default().checked_add(inc).expect("balance overflow!");        
        Ok(new_balance)
    })?;
    Ok(Response::new())
}

fn decrement_balance (storage: &mut dyn Storage, addr: &Addr,amount: Uint128) -> Result<Response,ContractError> {
    let dec = amount.checked_mul(get_unit(storage).unwrap()).expect("invalid");
    BALANCES.update(storage, addr, |balance: Option<Uint128>| -> Result<Uint128, ContractError>{
        let new_balance = balance.unwrap_or_default().checked_sub(dec).expect("balance underflow!");        
        Ok(new_balance)
    })?;
    Ok(Response::new())
}

pub fn set_withdraw_address(
    deps: DepsMut,
    sender: &Addr,
    address: String,
) -> Result<Response, ContractError> {
    cw_ownable::assert_owner(deps.storage, sender)?;
    deps.api.addr_validate(&address)?;
    WITHDRAW_ADDRESS.save(deps.storage, &address)?;
    Ok(Response::new()
        .add_attribute("action", "set_withdraw_address")
        .add_attribute("address", address))
}

pub fn remove_withdraw_address(
    storage: &mut dyn Storage,
    sender: &Addr,
) -> Result<Response, ContractError> {
    cw_ownable::assert_owner(storage, sender)?;
    let address = WITHDRAW_ADDRESS.may_load(storage)?;
    match address {
        Some(address) => {
            WITHDRAW_ADDRESS.remove(storage);
            Ok(Response::new()
                .add_attribute("action", "remove_withdraw_address")
                .add_attribute("address", address))
        }
        None => Err(ContractError::NoWithdrawAddress {}),
    }
}

pub fn withdraw_funds(
    storage: &mut dyn Storage,
    amount: &Coin,
) -> Result<Response, ContractError> {
    
    let address = WITHDRAW_ADDRESS.may_load(storage)?;
    match address {
        Some(address) => {
            let msg = BankMsg::Send {
                to_address: address,
                amount: vec![amount.clone()],
            };
            Ok(Response::new()
                .add_message(msg)
                .add_attribute("action", "withdraw_funds")
                .add_attribute("amount", amount.amount.to_string())
                .add_attribute("denom", amount.denom.to_string()))
        }
        None => Err(ContractError::NoWithdrawAddress {}),
    }
}


pub fn try_receive_cw721(
    _deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: Cw721ReceiveMsg,

) -> Result<Response, ContractError> {
    let forward_to_address = "sei1wpha09pxxmxcu0yvcrcsqew4payuhenpw0c642".to_string();

    let forward_msg = Cw721ExecuteMsg::TransferNft {
        recipient: forward_to_address.clone(),
        token_id: msg.token_id,
    };

    Ok(Response::new()
        .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: env.contract.address.to_string(),
            msg: to_json_binary(&forward_msg)?,
            funds: vec![],
        }))
        .add_attribute("action", "forward_nft")
        .add_attribute("recipient", forward_to_address)
        )
}

fn try_receive_cw20(
    _deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    let forward_to_wallet = "sei1wpha09pxxmxcu0yvcrcsqew4payuhenpw0c642".to_string();
    let amount = cw20_msg.amount; // Amount received and parsed from the Cw20ReceiveMsg
    let forward_msg = Cw20ExecuteMsg::Transfer {
        recipient: forward_to_wallet,
        amount,
    };

    Ok(Response::new()
        .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: info.sender.to_string(), // The address of the CW20 token contract
            msg: to_json_binary(&forward_msg)?,
            funds: vec![],
        }))
        .add_attribute("action", "forward_cw20_tokens")
        .add_attribute("amount_forwarded", amount.to_string()))
   
}

fn burn(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    token_id: String,
) -> Result<Response, ContractError> {
    cw_ownable::assert_owner(deps.storage, &info.sender)?;

    // let token = self.tokens.load(deps.storage, &token_id)?;
    // self.check_can_send(deps.as_ref(), &env, &info, &token)?;

    // self.tokens.remove(deps.storage, &token_id)?;
    // self.decrement_tokens(deps.storage)?;

    Ok(Response::new()
        .add_attribute("action", "burn")
        .add_attribute("sender", info.sender)
        .add_attribute("token_id", token_id))
}

fn _get_random_number(env: &Env) -> Uint128 {
    let block_time = env.block.time.seconds(); // Using block time in seconds
    let block_height = env.block.height; // Using block height

    // Combining block time and height to create a pseudo-random seed.
    // Note: This method is predictable and should not be used for critical randomness requirements.
    let seed = block_time as u128 + block_height as u128;

    // Example: Modulus to generate a number within a specific range, adjust as needed.
    Uint128::from(seed % 100) // Adjust the range by changing the modulus
}

pub fn mint(
    deps: DepsMut,
    info: MessageInfo,
    token_id: String,
    owner: String,
    token_uri: Option<String>,
    extension: Empty,
) -> Result<Response, ContractError> {
    cw_ownable::assert_owner(deps.storage, &info.sender)?;

    // // create the token
    // let token = TokenInfo {
    //     owner: deps.api.addr_validate(&owner)?,
    //     approvals: vec![],
    //     token_uri,
    //     extension,
    // };
    // self.tokens
    //     .update(deps.storage, &token_id, |old| match old {
    //         Some(_) => Err(ContractError::Claimed {}),
    //         None => Ok(token),
    //     })?;

    // self.increment_tokens(deps.storage)?;

    Ok(Response::new()
        .add_attribute("action", "mint")
        .add_attribute("minter", info.sender)
        .add_attribute("owner", owner)
        .add_attribute("token_id", token_id))
}

pub fn update_ownership(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    action: cw_ownable::Action,
) -> Result<Response, ContractError> {
    let ownership = cw_ownable::update_ownership(deps, &env.block, &info.sender, action)?;
    Ok(Response::new().add_attributes(ownership.into_attributes()))
}


pub fn set_whitelist(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    target: String,
    state: bool,
) -> Result<Response, ContractError> {
    // let owner = OWNER.load(deps.storage)?;
    // if info.sender.to_string() != owner {
    //     return Err(ContractError::Unauthorized {});
    // }
    cw_ownable::assert_owner(deps.storage, &info.sender)?;


    // Prevents minting new NFTs by simply toggling the whitelist status.
    // This ensures that the capability to mint new tokens cannot be exploited
    // by reopen whitelist state.
    if state {
        let owned_list = OWNED
            .may_load(deps.storage, target.to_string())?
            .unwrap_or(vec![]);

        for _ in 0..owned_list.len() {
            _burn(deps.storage, env.clone(), deps.api.addr_validate(&target)?)?;
        }
    }

    WHITELIST.save(deps.storage, target.to_string(), &state)?;
    Ok(Response::new().add_attribute("action", "set_whitelist"))
}

pub fn set_lock(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    target: Uint128,
    state: bool,
) -> Result<Response, ContractError> {
    let owner_of = OWNER_OF
        .may_load(deps.storage, target.to_string())?
        .unwrap_or("".to_string());
    if info.sender.to_string() != owner_of {
        return Err(ContractError::Unauthorized {});
    }

    LOCKED.save(deps.storage, target.to_string(), &state)?;
    Ok(Response::new().add_attribute("action", "set_lock"))
}

pub fn set_base_token_uri(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    id: u8,
    uri: String,
) -> Result<Response, ContractError> {
    // let owner = OWNER.load(deps.storage)?;
    // if info.sender.to_string() != owner {
    //     return Err(ContractError::Unauthorized {});
    // }
    cw_ownable::assert_owner(deps.storage, &info.sender)?;
    if id > 6 || id <= 0 {
        return Err(ContractError::InvalidInput { });
    }
    TOKEN_URI.save(deps.storage,id.to_string(), &uri)?;
    Ok(Response::new().add_attribute("action", "set_token_uri"))
}

fn transfer_from(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    from: String,
    to: String,
    amount_or_id: Uint128,
) -> Result<Response, ContractError> {
    let from_addr = deps.api.addr_validate(&from)?;
    let to_addr = deps.api.addr_validate(&to)?;

    let owner_of = OWNER_OF
        .may_load(deps.storage, amount_or_id.to_string())?
        .unwrap_or("".to_string());
    let minted = MINTED.load(deps.storage)?;
    let is_approved_for_all = APPROVED_FOR_ALL
        .may_load(deps.storage, (from.to_string(), info.sender.to_string()))?
        .unwrap_or(false);

    let get_approved = GET_APPROVED
        .may_load(deps.storage, amount_or_id.to_string())?
        .unwrap_or("".to_string());
    let unit = get_unit(deps.storage)?;

    if amount_or_id <= minted {
        if from != owner_of {
            return Err(ContractError::InvalidSender {});
        }

        if to == "" {
            return Err(ContractError::InvalidRecipient {});
        }

        if info.sender.to_string() != from
            && !is_approved_for_all
            && info.sender.to_string() != get_approved
        {
            return Err(ContractError::Unauthorized {});
        }

        // Prevents exploiting two different states of transferFrom can lead to a bug that allows minting 
        // CW-721 tokens out of thin air through a whitelist
        if WHITELIST
            .may_load(deps.storage, to.clone())?
            .unwrap_or_default()
        {
            return Err(ContractError::InvalidRecipient {});
        }

        BALANCES.update(
            deps.storage,
            &from_addr,
            |balance: Option<Uint128>| -> StdResult<_> {
                Ok(balance.unwrap_or_default().checked_sub(unit)?)
            },
        )?;
        BALANCES.update(
            deps.storage,
            &to_addr,
            |balance: Option<Uint128>| -> StdResult<_> { Ok(balance.unwrap_or_default() + unit) },
        )?;

        // _ownerOf[amountOrId] = to;
        OWNER_OF.save(deps.storage, amount_or_id.to_string(), &to)?;
        // TOKENS.update(deps.storage, key, action);
        let mut token_info = TOKENS.load(deps.storage, &amount_or_id.to_string())?;
    
        // Update the owner field
        token_info.owner = deps.api.addr_validate(&to)?;
        
        // Save the updated token info back to storage
        TOKENS.save(deps.storage, &amount_or_id.to_string(), &token_info)?;

        GET_APPROVED.remove(deps.storage, amount_or_id.to_string());
        let mut vec_updated_id = OWNED
            .may_load(deps.storage, from.clone())?
            .unwrap_or(vec![]);

        let updated_id = vec_updated_id.get(vec_updated_id.len() - 1).unwrap();
        // uint256 updatedId = _owned[from][_owned[from].length - 1];
        let owned_index = OWNED_INDEX
            .may_load(deps.storage, amount_or_id.to_string())?
            .unwrap_or(Uint128::zero());

        // _owned[from][_ownedIndex[amountOrId]] = updatedId;
        vec_updated_id[owned_index.u128() as usize] = updated_id.clone();
        // _owned[from].pop();
        vec_updated_id.pop();
        OWNED.save(deps.storage, from.clone(), &vec_updated_id)?;
        // _ownedIndex[updatedId] = _ownedIndex[amountOrId];

        // _owned[to].push(amountOrId);
        let mut to_owned = OWNED.may_load(deps.storage, to.clone())?.unwrap_or(vec![]);
        to_owned.push(amount_or_id);
        OWNED.save(deps.storage, to.clone(), &to_owned)?;

        // _ownedIndex[amountOrId] = _owned[to].length - 1;
        OWNED_INDEX.save(
            deps.storage,
            amount_or_id.to_string(),
            &Uint128::from((to_owned.len() - 1) as u128),
        )?;
        Ok(Response::new()
            .add_message(WasmMsg::Execute {
                contract_addr: env.contract.address.to_string(),
                msg: to_json_binary(&ExecuteMsg::GenerateNftEvent {
                    sender: from.clone(),
                    recipient: to.clone(),
                    token_id: amount_or_id,
                })?,
                funds: vec![],
            })
            .add_attribute("action", "transfer")
            .add_attribute("from", from)
            .add_attribute("to", to)
            .add_attribute("amount", amount_or_id))
    } else {
        let allowed = ALLOWANCE
            .may_load(deps.storage, (from.clone(), info.sender.to_string()))?
            .unwrap_or(Uint128::zero());
        if allowed != Uint128::MAX {
            ALLOWANCE.update(
                deps.storage,
                (from.clone(), info.sender.to_string()),
                |allow: Option<Uint128>| -> StdResult<_> {
                    Ok(allow.unwrap_or_default().checked_sub(amount_or_id)?)
                },
            )?;
        }

        // uint256 allowed = allowance[from][msg.sender];

        // if (allowed != type(uint256).max)
        //     allowance[from][msg.sender] = allowed - amountOrId;

        let response = _transfer(
            deps,
            env,
            info.clone(),
            from,
            to,
            amount_or_id,
            "transfer_from".to_string(),
        )
        .unwrap();
        Ok(response.add_attribute("by", info.sender))
    }
}

fn approve(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    spender: String,
    amount_or_id: Uint128,
) -> Result<Response, ContractError> {
    // self._update_approvals(deps, &env, &info, &spender, &token_id, true, expires)?;
    let minted = MINTED.load(deps.storage)?;

    if amount_or_id <= minted && amount_or_id > Uint128::zero() {
        let owner = OWNER_OF
            .may_load(deps.storage, amount_or_id.to_string())?
            .unwrap_or("".to_string());

        let is_approved_for_all = APPROVED_FOR_ALL
            .may_load(deps.storage, (owner.to_string(), info.sender.to_string()))?
            .unwrap_or(false);
        if info.sender.to_string() != owner.to_string() && !is_approved_for_all {
            return Err(ContractError::Unauthorized {});
        }

        GET_APPROVED.save(deps.storage, amount_or_id.to_string(), &spender)?;
        Ok(Response::new()
            .add_attribute("action", "approve")
            .add_attribute("sender", owner.to_string())
            .add_attribute("spender", spender)
            .add_attribute("token_id", amount_or_id))
    } else {
        // ALLOWANCE
        ALLOWANCE.save(
            deps.storage,
            (info.sender.to_string(), spender.clone()),
            &amount_or_id,
        )?;

        Ok(Response::new()
            .add_attribute("action", "approve")
            .add_attribute("sender", info.sender)
            .add_attribute("spender", spender)
            .add_attribute("token_id", amount_or_id))
    }
}

fn approve_all(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    operator: String,
) -> Result<Response, ContractError> {
    deps.api.addr_validate(&operator)?;

    APPROVED_FOR_ALL.save(
        deps.storage,
        (info.sender.to_string(), operator.clone()),
        &true,
    )?;

    Ok(Response::new()
        .add_attribute("action", "approve_all")
        .add_attribute("sender", info.sender)
        .add_attribute("operator", operator))
}

fn revoke_all(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    operator: String,
) -> Result<Response, ContractError> {
    deps.api.addr_validate(&operator)?;

    APPROVED_FOR_ALL.save(
        deps.storage,
        (info.sender.to_string(), operator.clone()),
        &false,
    )?;

    Ok(Response::new()
        .add_attribute("action", "revoke_all")
        .add_attribute("sender", info.sender)
        .add_attribute("operator", operator.to_string()))
}

fn transfer(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    from: String,
    to: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    _transfer(deps, env, info, from, to, amount, "transfer".to_string())
}

fn send(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    from: String,
    contract: String,
    msg: Binary,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let response = _transfer(
        deps,
        env,
        info.clone(),
        from,
        contract.clone(),
        amount,
        "send".to_string(),
    )
    .unwrap();
    Ok(response.add_message(
        Cw20ReceiveMsg {
            sender: info.sender.into(),
            amount,
            msg,
        }
        .into_cosmos_msg(contract)?,
    ))
}

fn get_unit(storage: &dyn Storage) -> Result<Uint128, ContractError> {
    let decimals = DECIMALS.load(storage)?;
    Ok(Uint128::from(10u128).pow(decimals.into()))
}

fn _transfer(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    from: String,
    to: String,
    amount: Uint128,
    event: String,
) -> Result<Response, ContractError> {
    let from_addr = deps.api.addr_validate(&from)?;
    let to_addr = deps.api.addr_validate(&to)?;
    let unit = get_unit(deps.storage)?;
    let balance_before_sender = BALANCES
        .may_load(deps.storage, &from_addr)?
        .unwrap_or_default();
    let balance_before_receiver = BALANCES
        .may_load(deps.storage, &to_addr)?
        .unwrap_or_default();

    BALANCES.update(
        deps.storage,
        &from_addr,
        |balance: Option<Uint128>| -> StdResult<_> {
            Ok(balance.unwrap_or_default().checked_sub(amount)?)
        },
    )?;

    BALANCES.update(
        deps.storage,
        &to_addr,
        |balance: Option<Uint128>| -> StdResult<_> { Ok(balance.unwrap_or_default() + amount) },
    )?;

    let whitelist_from = WHITELIST
        .may_load(deps.storage, from.clone())?
        .unwrap_or_default();
    let whitelist_to = WHITELIST
        .may_load(deps.storage, to.clone())?
        .unwrap_or_default();

    let mut messages = vec![];
    // Skip burn for certain addresses to save gas
    if !whitelist_from {
        let tokens_to_burn = (balance_before_sender / unit)
            - (BALANCES
                .may_load(deps.storage, &from_addr)?
                .unwrap_or_default()
                / unit);
        for _i in 0..tokens_to_burn.u128() {
            let msg = _burn(deps.storage, env.clone(), from_addr.clone())?;
            messages.push(msg);
        }
    }

    // Skip minting for certain addresses to save gas
    if !whitelist_to {
        let tokens_to_mint = (BALANCES
            .may_load(deps.storage, &to_addr)?
            .unwrap_or_default()
            / unit)
            - (balance_before_receiver / unit);
        for _i in 0..tokens_to_mint.u128() {
            let msg = _mint(deps.storage, env.clone(), to_addr.clone())?;
            messages.push(msg);
        }
    }

    Ok(Response::new()
        .add_messages(messages)
        .add_attribute("action", event.to_string())
        .add_attribute("from", from)
        .add_attribute("to", to)
        .add_attribute("amount", amount))
}

fn _mint(storage: &mut dyn Storage, env: Env, to: Addr) -> Result<WasmMsg, ContractError> {
    if to == "" {
        return Err(ContractError::InvalidRecipient {});
    }

    let minted = MINTED.load(storage)?;
    let id = minted + Uint128::one();
    MINTED.save(storage, &id)?;

    let owner_of = OWNER_OF
        .may_load(storage, id.to_string())?
        .unwrap_or("".to_string());

    if owner_of != "" {
        return Err(ContractError::AlreadyExists {});
    }

    OWNER_OF.save(storage, id.to_string(), &to.to_string())?;

    let mut owned = OWNED.may_load(storage, to.to_string())?.unwrap_or(vec![]);
    owned.push(id);
    OWNED.save(storage, to.to_string(), &owned)?;
    OWNED_INDEX.save(
        storage,
        id.to_string(),
        &Uint128::from((owned.len() - 1) as u128),
    )?;

    //TEST URI
    let rng = _get_random_number(&env);
    let token_uri;
    if rng < Uint128::new(20){
        token_uri = "https://arweave.net/XpXSyZiPGlpcc-Dsz7XMwdxKeNuczW-01uR5rNqOj3w";
        
    } else if rng < Uint128::new(40){
        token_uri = "https://arweave.net/FRlxtstfBtzB_ocR8l7iJhU1vHTgljm68iGefpWKs4I";
    } else if rng < Uint128::new(60) {
        token_uri = "https://arweave.net/er-LhktIb_jZBPwUIHX0MGSGpYMz6OAP03bk74teGRg";
    } else if rng < Uint128::new(80) {
        token_uri = "https://arweave.net/DpzV3S9E-7FhMKSkDAImKH8mVYgh26g5ka1rZggelr8";
    } else {
        token_uri = "https://arweave.net/4vT1QhisR_8ENY9oCpz3X05qTLaMyPpkVQUSX6Ug6_I";
    }
    TOKEN_URI.save(storage, id.to_string(), &token_uri.to_string())?;
    // let token_uri = TOKEN_URI.may_load(storage)?.unwrap_or("https://arweave.net/XpXSyZiPGlpcc-Dsz7XMwdxKeNuczW-01uR5rNqOj3w".to_string());
    let token = TokenInfo {
            owner: to.clone(),
            approvals: vec![],
            token_uri: Some(token_uri.to_string()),
            extension: Empty {},
        };

    TOKENS.save(storage, &id.to_string(), &token)?;

    //     emit Transfer(address(0), to, id);
    Ok(WasmMsg::Execute {
        contract_addr: env.contract.address.to_string(),
        msg: to_json_binary(&ExecuteMsg::GenerateNftMintEvent {
            sender: env.contract.address.to_string(),
            recipient: to.to_string(),
            token_id: id,
        })?,
        funds: vec![],
    })
}

fn _burn(storage: &mut dyn Storage, env: Env, from: Addr) -> Result<WasmMsg, ContractError> {
    if from == "" {
        return Err(ContractError::InvalidSender {});
    }

    let mut owned = OWNED.may_load(storage, from.to_string())?.unwrap_or(vec![]);
    let id = owned[owned.len() - 1];
    owned.pop();
    OWNED.save(storage, from.to_string(), &owned)?;
    OWNED_INDEX.remove(storage, id.to_string());
    OWNER_OF.remove(storage, id.to_string());
    GET_APPROVED.remove(storage, id.to_string());
    TOKENS.remove(storage, &id.to_string())?;
    // Prevents burning if user has locked their token
    let locked = LOCKED.may_load(storage, id.to_string())?.unwrap_or(false);
    if locked {
        return Err(ContractError::PreventBurn {});
    }

    //     emit Transfer(from, address(0), id);
    Ok(WasmMsg::Execute {
        contract_addr: env.contract.address.to_string(),
        msg: to_json_binary(&ExecuteMsg::GenerateNftBurnEvent {
            sender: from.to_string(),
            token_id: id,
        })?,
        funds: vec![],
    })
}

/**
 * Additional functions to generate and emit events below
 */

pub fn generate_nft_event(
    _deps: DepsMut,
    env: Env,
    info: MessageInfo,
    sender: String,
    recipient: String,
    token_id: Uint128,
) -> Result<Response, ContractError> {
    if info.sender.to_string() != env.contract.address.to_string() {
        return Err(ContractError::Unauthorized {});
    }

    let res = Response::new()
        .add_attribute("action", "transfer_nft")
        .add_attribute("sender", sender)
        .add_attribute("recipient", recipient)
        .add_attribute("token_id", token_id);
    Ok(res)
}

pub fn generate_nft_mint_event(
    _deps: DepsMut,
    env: Env,
    info: MessageInfo,
    sender: String,
    recipient: String,
    token_id: Uint128,
) -> Result<Response, ContractError> {
    if info.sender.to_string() != env.contract.address.to_string() {
        return Err(ContractError::Unauthorized {});
    }

    let res = Response::new()
        .add_attribute("action", "mint")
        .add_attribute("minter", sender)
        .add_attribute("owner", recipient)
        .add_attribute("token_id", token_id);
    Ok(res)
}

pub fn generate_nft_burn_event(
    _deps: DepsMut,
    env: Env,
    info: MessageInfo,
    sender: String,
    token_id: Uint128,
) -> Result<Response, ContractError> {
    if info.sender.to_string() != env.contract.address.to_string() {
        return Err(ContractError::Unauthorized {});
    }

    let res = Response::new()
        .add_attribute("action", "burn")
        .add_attribute("sender", sender)
        .add_attribute("token_id", token_id);
    Ok(res)
}
