use std::convert::TryInto;

use cosmwasm_std::{
    Addr, BankMsg, Coin, CosmosMsg, DepsMut, DistributionMsg, Env, MessageInfo, QuerierWrapper, Response, StakingMsg, StdResult, Storage, Uint128, Uint256, coin
};
use cw20_base::{contract::execute_burn, state::{BALANCES, TOKEN_INFO}};
use cw20_staking::state::{INVESTMENT, InvestmentInfo, Supply, TOTAL_SUPPLY};
use crate::{contract::{assert_bonds, get_bonded}, msg::ContractError};

// Primitives for cw20-liquid-staking




pub fn unbond_logic(
    storage: &mut dyn Storage,
    addr: &Addr,
    amount: Uint128,
    bonded: Uint128,
    invest: &InvestmentInfo,
) -> Result<Uint256, ContractError> {
    let supply = TOTAL_SUPPLY.load(storage)?;

    // Validate amount
    assert_bonds(&supply, bonded)?;
    if amount.is_zero() || amount < invest.min_withdrawal {
        return Err(ContractError::UnbondTooSmall {
            min_bonded: invest.min_withdrawal,
            denom: invest.bond_denom.clone(),
        });
    }

    // Update balances and total supply
    BALANCES.update(storage, addr, |balance| {
        balance.unwrap_or_default().checked_sub(amount).map_err(|_| ContractError::Unauthorized {})
    })?;

    TOKEN_INFO.update(storage, |mut info| -> StdResult<_> {
        info.total_supply = info.total_supply.checked_sub(amount)?;
        Ok(info)
    })?;

    // Calculate unbond amount and update supply
    let unbond = amount.multiply_ratio(bonded, supply.issued);

    TOTAL_SUPPLY.save(storage, &Supply { 
        issued: supply.issued.checked_sub(amount)?,
        bonded: bonded.checked_sub(unbond)?,
        claims: supply.claims,
    })?;

    Ok(unbond.into())
}




pub fn liquid_unbond(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint256,
) -> Result<Response, ContractError> {
    let invest = INVESTMENT.load(deps.storage)?;
    let bonded = get_bonded(&deps.querier, &env.contract.address)?;

    let amount = unbond_logic(deps.storage, &info.sender, amount.try_into()?, bonded, &invest)?;
    let denom = invest.bond_denom.clone();

    let delegation = deps.querier.query_all_delegations(&env.contract.address)?
        .into_iter()
        .find(|d| d.amount.amount >= amount)
        .ok_or(ContractError::NothingToClaim {})?;

    let amount = amount.to_string();
    let res = Response::new()
        .add_attributes([("action", "liquid_unbond"), ("amount", &amount)]);

    Ok(res
        .add_message(common::MsgTokenizeShares {
            delegator_address: env.contract.address.to_string(),
            validator_address: delegation.validator.to_string(),
            amount: Some(common::Coin { denom, amount: amount }),
            tokenized_share_owner: info.sender.to_string(),
        }.to_cosmos_msg()
    ))
}

