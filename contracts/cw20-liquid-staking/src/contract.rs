#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use std::convert::TryInto;
use cosmwasm_std::{
    Addr, Binary, Deps, DepsMut, Env, MessageInfo, QuerierWrapper, Response, StdResult, Uint128,
};
use cw2::set_contract_version;
use cw20_staking::state::{INVESTMENT, TOTAL_SUPPLY, Supply};

use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, ContractError};
use crate::proto::{MsgTokenizeShares, Coin};

const CONTRACT_NAME: &str = "crates.io:cw20-liquid-staking";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let res = cw20_staking::contract::instantiate(deps.branch(), env, info, msg);
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    res
}

fn get_bonded(querier: &QuerierWrapper, contract: &Addr) -> Result<Uint128, ContractError> {
    let bonds = querier.query_all_delegations(contract)?;
    if bonds.is_empty() {
        return Ok(Uint128::zero());
    }
    let denom = bonds[0].amount.denom.as_str();
    bonds.iter().fold(Ok(Uint128::zero()), |racc, d| {
        let acc = racc?;
        if d.amount.denom.as_str() != denom {
            Err(ContractError::DifferentBondDenom {
                denom1: denom.into(),
                denom2: d.amount.denom.to_string(),
            })
        } else {
            let amount: Uint128 = d.amount.amount.try_into()?;
            Ok(acc + amount)
        }
    })
}

fn assert_bonds(supply: &Supply, bonded: Uint128) -> Result<(), ContractError> {
    if supply.bonded != bonded {
        Err(ContractError::BondedMismatch {
            stored: supply.bonded,
            queried: bonded,
        })
    } else {
        Ok(())
    }
}

pub fn execute_unbond(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let invest = INVESTMENT.load(deps.storage)?;
    if amount < invest.min_withdrawal {
        return Err(ContractError::UnbondTooSmall {
            min_bonded: invest.min_withdrawal,
            denom: invest.bond_denom,
        });
    }

    let tax = amount.mul_floor(invest.exit_tax);

    cw20_base::contract::execute_burn(deps.branch(), env.clone(), info.clone(), amount)?;
    if tax > Uint128::zero() {
        let sub_info = MessageInfo {
            sender: env.contract.address.clone(),
            funds: vec![],
        };
        cw20_base::contract::execute_mint(
            deps.branch(),
            env.clone(),
            sub_info,
            invest.owner.to_string(),
            tax,
        )?;
    }

    let bonded = get_bonded(&deps.querier, &env.contract.address)?;
    let remainder = amount.checked_sub(tax)?;
    let mut supply = TOTAL_SUPPLY.load(deps.storage)?;
    assert_bonds(&supply, bonded)?;

    let unbond = remainder.multiply_ratio(bonded, supply.issued);
    supply.bonded = bonded.checked_sub(unbond)?;
    supply.issued = supply.issued.checked_sub(remainder)?;
    TOTAL_SUPPLY.save(deps.storage, &supply)?;

    let tokenize_msg = MsgTokenizeShares {
        delegator_address: env.contract.address.to_string(),
        validator_address: invest.validator.clone(),
        amount: Some(Coin {
            denom: invest.bond_denom.clone(),
            amount: unbond.to_string(),
        }),
        tokenized_share_owner: info.sender.to_string(),
    };

    let cosmos_msg = tokenize_msg.to_cosmos_msg();

    let res = Response::new()
        .add_message(cosmos_msg)
        .add_attribute("action", "unbond")
        .add_attribute("to", info.sender.to_string())
        .add_attribute("unbonded", unbond)
        .add_attribute("burnt", amount);
    Ok(res)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Unbond { amount } => execute_unbond(deps, env, info, amount),
        _ => cw20_staking::contract::execute(deps, env, info, msg),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    cw20_staking::contract::query(deps, env, msg)
}

