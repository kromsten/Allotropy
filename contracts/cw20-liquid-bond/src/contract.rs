use std::str::FromStr;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use cosmwasm_std::{
    BankMsg, Binary, Coin, CosmosMsg, Decimal, Deps, DepsMut, Env, MessageInfo, Response, StakingMsg, StdError, StdResult, Uint128, Uint256, ensure
};
use cw2::set_contract_version;
use cw20_base::allowances::execute_burn_from;
use cw20_base::contract::execute_burn;
use cw20_bonding::curves::DecimalPlaces;
use cw_utils::nonpayable;

use crate::error::ContractError;
use crate::msg::{InstantiateMsg, ExecuteMsg};
use crate::state::{ADMIN, BURNED_TOTAL, CONFIG, CURVE_TYPE, Config, VALIDATORS};
use crate::utils::to_bonding_msg;
use cw20_base::state::{TOKEN_INFO, TokenInfo, MinterData};
use cw20_bonding::state::{CURVE_STATE, CurveState};

// entrypoints of the contract
use cw20_bonding::contract::{do_execute, do_query};
use cw20_bonding::msg::{QueryMsg};


// version info for migration info
pub const CONTRACT_NAME: &str = "crates.io:cw20-liquid-bond";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");



#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    ensure!(
        msg.commission_rate >= Decimal::zero() && msg.commission_rate <= Decimal::percent(100),
        ContractError::InvalidCommissionRate {}
    );

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
    
    let places = DecimalPlaces::new(msg.decimals, msg.reserve_decimals);
    let supply = CurveState::new(msg.reserve_denom, places);
    
    TOKEN_INFO.save(deps.storage, &data)?;
    CURVE_STATE.save(deps.storage, &supply)?;
    CURVE_TYPE.save(deps.storage, &msg.curve_type)?;

    let config = Config {
        com_rate: msg.commission_rate,
        com_recipient: deps.api.addr_validate(&msg.commission_recipient)?,
    };
    CONFIG.save(deps.storage, &config)?;
    VALIDATORS.save(deps.storage, &msg.validators)?;

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

    let curve_type = CURVE_TYPE.load(deps.storage)?;
    let curve_fn = curve_type.to_curve_fn();

    let bonding_msg = to_bonding_msg(&msg).ok_or_else(|| ContractError::NestedCallFor {})?;

    let res = match msg {
        ExecuteMsg::Buy { validator } => {
            let cfg = CONFIG.load(deps.storage)?;
            let state = CURVE_STATE.load(deps.storage)?;
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

            let res = do_execute(deps, env, info, bonding_msg, curve_fn)?
                .add_message(StakingMsg::Delegate { validator, amount });


            if !com_amount.is_zero() {
                let to_address = cfg.com_recipient.to_string();
                res
                    .add_attribute("commission", com_amount)
                    .add_attribute("commission_recipient", &to_address)
                    .add_message(BankMsg::Send {
                        to_address,
                        amount: vec![cosmwasm_std::Coin {
                            denom: state.reserve_denom,
                            amount: com_amount.into(),
                        }],
                    })
            } else {
                res
            }
        },
        ExecuteMsg::Sell { amount, validator } => {

            let state = CURVE_STATE.load(deps.storage)?;
            let querier = deps.querier.clone();
            let contract_addr = env.contract.address.to_string();
            let sender = info.sender.to_string();

            let to_address = info.sender.to_string();
            let denom = state.reserve_denom.clone();
            let balance = deps.querier.query_balance(&env.contract.address, &denom)?.amount;
            let rem = amount.checked_sub(balance).unwrap_or_else(|_| Uint256::zero());

            let mut messages: Vec<CosmosMsg> = Vec::with_capacity(2);

            let res =  do_execute(deps, env, info, bonding_msg, curve_fn)?;
            let amount : Uint256 = res.attributes.iter().
                find(|a| a.key == "released")
                .map(|a| Uint128::from_str(&a.value).unwrap())
                .ok_or(ContractError::Unauthorized {})?
                .into();

            let normal = if rem.is_zero() {
                messages.push(BankMsg::Send { to_address: to_address, amount: vec![Coin { denom, amount }] }.into());
                amount
            } else {
                
                messages.push(BankMsg::Send {
                    to_address: to_address.clone(),
                    amount: vec![Coin { denom: denom.clone(), amount: balance }],
                }.into());

                let delegation = querier.query_all_delegations(&contract_addr)?
                    .into_iter()
                    .find(|d| d.amount.amount >= rem && validator.as_ref().map_or(true, |v| d.validator == *v))
                    .ok_or(ContractError::Unauthorized  {})?;

                messages.push(common::MsgTokenizeShares {
                    delegator_address: contract_addr,
                    validator_address: delegation.validator.to_string(),
                    amount: Some(common::Coin { denom, amount: rem.to_string() }),
                    tokenized_share_owner: sender,
                }.to_cosmos_msg());

                balance
            };
            

            Response::new()
                .add_messages(messages)
                .add_attributes(res.attributes)
                .add_attributes([
                    ("action", "liquid_unbond"),
                    ("normal_unbond", &normal.to_string()),
                    ("liquid_unbond", &rem.to_string()),
                ])
        },

        // "sponsored burn"
        ExecuteMsg::Burn { amount } => {
            BURNED_TOTAL.update(deps.storage, |total| -> StdResult<_> {
                Ok::<_, StdError>(Uint128::new(total).checked_add(amount)?.u128())
            })?;
            execute_burn(deps, env, info, amount)?
        },

        ExecuteMsg::BurnFrom { owner, amount } => {
            BURNED_TOTAL.update(deps.storage, |total| -> StdResult<_> {
                Ok::<_, StdError>(Uint128::new(total).checked_add(amount)?.u128())
            })?;
            execute_burn_from(deps, env, info, owner, amount)?
        }

        _ => {
            do_execute(deps, env, info, bonding_msg, curve_fn)?
        }
       
    };

    Ok(res)
}


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    let curve_type = CURVE_TYPE.load(deps.storage)?;
    let curve_fn = curve_type.to_curve_fn();
    do_query(deps, env, msg, curve_fn)
}