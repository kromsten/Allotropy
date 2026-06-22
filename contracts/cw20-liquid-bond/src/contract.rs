#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use cosmwasm_std::{
    BankMsg, Binary, Coin, CosmosMsg, Decimal, Deps, DepsMut, Env, MessageInfo, OverflowError, OverflowOperation, Response, StakingMsg, StdError, StdResult, Uint128, Uint256, ensure,
};
use cw2::set_contract_version;
use cw20_base::allowances::execute_burn_from;
use cw20_base::contract::{execute_burn, execute_mint};
use cw20_bonding::curves::DecimalPlaces;
use cw_utils::{must_pay, nonpayable};

use crate::error::ContractError;
use crate::msg::{InstantiateMsg, ExecuteMsg};
use crate::state::{ADMIN, BURNED_TOTAL, CONFIG, CURVE_TYPE, Config, LAST_BALANCE, STAKE_TOTAL, VALIDATORS};
use crate::utils::{to_bonding_msg, updated_curve_slope};
use cw20_base::state::{TOKEN_INFO, TokenInfo, MinterData};
use cw20_bonding::state::{CURVE_STATE, CurveState};

// entrypoints of the contract
use cw20_bonding::contract::{do_execute, do_query};
use cw20_bonding::msg::{CurveFn, QueryMsg};


// version info for migration info
pub const CONTRACT_NAME: &str = "crates.io:cw20-liquid-bond";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub (crate) const DENOM : &str = "uatom";
pub (crate) const DECIMALS : u8 = 6;


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let com_rate = msg.commission_rate.unwrap_or_default();
    ensure!(com_rate >= Decimal::zero() && com_rate <= Decimal::percent(100),ContractError::BadComRate {});

    let data = TokenInfo {
        name: msg.name,
        symbol: msg.symbol,
        decimals: msg.decimals,
        total_supply: Uint128::zero(),
        mint: Some(MinterData {
            minter: env.contract.address,
            cap: None,
        }),
    };

    let com_recipient = msg.commission_recipient.as_ref()
        .map(|addr| deps.api.addr_validate(addr).ok())
        .flatten()
        .unwrap_or_else(|| info.sender.clone());

    let places = DecimalPlaces::new(msg.decimals, DECIMALS);
    
    CONFIG.save(deps.storage, &Config { com_recipient, com_rate })?;
    CURVE_STATE.save(deps.storage, &CurveState::new(DENOM.to_string(), places))?;
    CURVE_TYPE.save(deps.storage, &msg.curve_type)?;
    TOKEN_INFO.save(deps.storage, &data)?;

  
    VALIDATORS.save(deps.storage, &msg.validators)?;
    STAKE_TOTAL.save(deps.storage, &0u128)?;
    LAST_BALANCE.save(deps.storage, &0u128)?;
    BURNED_TOTAL.save(deps.storage, &0u128)?;
    ADMIN.set(deps, Some(info.sender.clone()))?;

    Ok(Response::default())
}



#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {

    let mut curve_type = CURVE_TYPE.load(deps.storage)?;
    let token_info = TOKEN_INFO.load(deps.storage)?;

    let mut state = CURVE_STATE.load(deps.storage)?;

    let staked_total = STAKE_TOTAL.load(deps.storage)?;
    let balance_total = deps.querier.query_balance(&env.contract.address, DENOM)?;

    let paid = cw_utils::may_pay(&info, DENOM)?;
    let balance = balance_total.amount.checked_sub(paid)?;

    let reserve: Uint128 = balance.checked_add(staked_total.into())?.try_into()?;

    curve_type = updated_curve_slope(deps.storage, &curve_type, state.decimals.clone(), &reserve, &token_info.total_supply)?;
    let curve_fn = curve_type.to_curve_fn();


    match msg {
        ExecuteMsg::Buy { validator } => {
            execute_buy(deps, env, info,  &mut state, curve_fn, validator)
        }
        ExecuteMsg::Sell { amount, validator } => {
            execute_sell(deps, env, info, &mut state, curve_fn, amount, validator)
        },

        // "sponsored burn"
        ExecuteMsg::Burn { amount } => {
            BURNED_TOTAL.update(deps.storage, |total| -> StdResult<_> {
                Ok::<_, StdError>(Uint128::new(total).checked_add(amount)?.u128())
            })?;
            execute_burn(deps, env, info, amount).map_err(Into::into)
        },
        ExecuteMsg::BurnFrom { owner, amount } => {
            BURNED_TOTAL.update(deps.storage, |total| -> StdResult<_> {
                Ok::<_, StdError>(Uint128::new(total).checked_add(amount)?.u128())
            })?;
            execute_burn_from(deps, env, info, owner, amount).map_err(Into::into)
        }
        _ => {
            let bonding_msg = to_bonding_msg(&msg).ok_or_else(|| ContractError::NestedCallFor {})?;
            do_execute(deps, env, info, bonding_msg, curve_fn).map_err(Into::into)
        }
    }
}


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    let curve_type = CURVE_TYPE.load(deps.storage)?;
    let curve_fn = curve_type.to_curve_fn();
    do_query(deps, env, msg, curve_fn)
}






fn execute_buy(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    state: &mut CurveState,
    curve_fn: CurveFn,
    validator: Option<String>,
) -> Result<Response, ContractError> {
    let cfg = CONFIG.load(deps.storage)?;
    let pay: Uint128 = cw_utils::must_pay(&info, &state.reserve_denom)?.try_into()?;

    let com_amount: Uint128 = Decimal::from_ratio(pay, 1u32).checked_mul(cfg.com_rate)?.to_uint_floor();
    let buy_amount = pay.checked_sub(com_amount)?;

    let amount = cosmwasm_std::Coin {
        denom: state.reserve_denom.clone(),
        amount: buy_amount.into(),
    };
    let info = MessageInfo { 
        sender: info.sender.clone(),
        funds: vec![amount.clone()] 
    };

    let validator = validator.unwrap_or(
        VALIDATORS.load(deps.storage)?
            .first().ok_or(ContractError::Unauthorized {})?
            .clone()
    );

    STAKE_TOTAL.update(deps.storage, |total| -> StdResult<_> {
        Ok::<_, StdError>(Uint128::new(total).checked_add(buy_amount)?.u128())
    })?;


    let payment : Uint128 = must_pay(&info, DENOM)?.try_into()?;

    // calculate how many tokens can be purchased with this and mint them
    let curve = curve_fn(state.clone().decimals);
    state.reserve += payment;
    let new_supply = curve.supply(state.reserve);
    let minted = new_supply
        .checked_sub(state.supply)
        .map_err(|_|StdError::from(OverflowError { operation: OverflowOperation::Sub}))?;
    state.supply = new_supply;
    CURVE_STATE.save(deps.storage, &state)?;

    // call into cw20-base to mint the token, call as self as no one else is allowed
    let sub_info = MessageInfo {
        sender: env.contract.address.clone(),
        funds: vec![],
    };

    execute_mint(deps, env, sub_info, info.sender.to_string(), minted)?;

    let res = Response::new()
        .add_message(StakingMsg::Delegate { validator, amount })
        .add_attributes([
            ("action", "buy"),
            ("supply", &state.supply.to_string()),
            ("reserve", &state.reserve.to_string()),
            ("minted", &minted.to_string()),
        ]);


    Ok(if !com_amount.is_zero() {
        let to_address = cfg.com_recipient.to_string();
        res
            .add_attribute("commission", com_amount.to_string())
            .add_attribute("commission_recipient", &to_address)
            .add_message(BankMsg::Send {
                to_address,
                amount: vec![cosmwasm_std::Coin {
                    denom: state.reserve_denom.clone(),
                    amount: com_amount.into(),
                }],
            })
    } else {
        res
    })
}





fn execute_sell(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    state: &mut CurveState,
    curve_fn: CurveFn,
    amount: Uint256,
    validator: Option<String>,
) -> Result<Response, ContractError> {
    let contract_addr = env.contract.address.to_string();
    let sender = info.sender.to_string();

    let denom = state.reserve_denom.clone();

    let amount128: Uint128 = amount.try_into()?;
    
    let curve = curve_fn(state.clone().decimals);
    state.supply = state
        .supply
        .checked_sub(amount128)
        .map_err(|_|StdError::from(OverflowError { operation: OverflowOperation::Sub }))?;

    let new_reserve = curve.reserve(state.supply);
    let released : Uint256 = state
        .reserve
        .checked_sub(new_reserve)
        .map_err(|_|StdError::from(OverflowError { operation: OverflowOperation::Sub }))?
        .into();

    state.reserve = new_reserve;
    CURVE_STATE.save(deps.storage, &state)?;

    let balance = deps.querier.query_balance(&env.contract.address, &denom)?.amount;

    let to_send_now = std::cmp::min(released, balance);
    let to_liquid_unbond = released.checked_sub(to_send_now).unwrap_or_default();

    let mut messages: Vec<CosmosMsg> = Vec::with_capacity(2);


    if !to_send_now.is_zero() {
        messages.push(
            BankMsg::Send {
                to_address: info.sender.to_string(),
                amount: vec![Coin {
                    denom: state.reserve_denom.clone(),
                    amount: to_send_now.clone(),
                }],
            }
            .into(),
        );
    }

    if !to_liquid_unbond.is_zero() {

        STAKE_TOTAL.update(deps.storage, |total| -> StdResult<_> {
            Ok::<_, StdError>(Uint128::new(total).checked_sub(to_liquid_unbond.try_into().unwrap_or_default())?.u128())
        })?;
    
        let delegation = deps.querier.query_all_delegations(&contract_addr)?
            .into_iter()
            .find(|d| d.amount.amount >= to_liquid_unbond && validator.as_ref().map_or(true, |v| d.validator == *v))
            .ok_or(ContractError::Unauthorized  {})?;
    
        messages.push(common::MsgTokenizeShares {
            delegator_address: contract_addr,
            validator_address: delegation.validator.to_string(),
            amount: Some(common::Coin { denom, amount: to_liquid_unbond.to_string() }),
            tokenized_share_owner: sender,
        }.to_cosmos_msg());
    }

    execute_burn(deps, env, info, amount128)?;
    
    Ok(Response::new()
        .add_messages(messages)
        .add_attributes([
            ("action", "liquid_unbond"),
            ("supply", &state.supply.to_string()),
            ("reserve", &state.reserve.to_string()),
            ("released", &released.to_string()),
            ("normal_unbond", &to_send_now.to_string()),
            ("liquid_unbond", &to_liquid_unbond.to_string()),
        ]))
}