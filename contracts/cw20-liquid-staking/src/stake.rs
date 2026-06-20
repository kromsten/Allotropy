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
    supply: &mut Supply,
    invest: &InvestmentInfo,
) -> Result<Uint256, ContractError> {
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

    let tax = amount.mul_floor(invest.exit_tax);
    let remainder = amount.checked_sub(tax)?;

    // Handle exit tax
    if !tax.is_zero() {
        let mut config = TOKEN_INFO.load(storage)?;
        if config.mint.as_ref().map(|m| &m.minter) != Some(addr) {
            return Err(ContractError::Unauthorized {});
        }
        
        let new_total = config.total_supply + tax;
        if let Some(limit) = config.get_cap() {
            if new_total > limit {
                return Err(ContractError::CannotExceedCap {});
            }
        }
        config.total_supply = new_total;
        TOKEN_INFO.save(storage, &config)?;

        BALANCES.update(storage, &invest.owner, |balance| 
            Ok::<_, ContractError>(balance.unwrap_or_default() + tax)
        )?;
    }

    // Calculate unbond amount and update supply
    let unbond = remainder.multiply_ratio(bonded, supply.issued);
    supply.bonded = bonded.checked_sub(unbond)?;
    supply.issued = supply.issued.checked_sub(remainder)?;
    TOTAL_SUPPLY.save(storage, &supply)?;

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
    let mut supply = TOTAL_SUPPLY.load(deps.storage)?;

    let amount = unbond_logic(deps.storage, &info.sender, amount.try_into()?, bonded, &mut supply, &invest)?;

    let mut messages: Vec<CosmosMsg> = vec![];

    let to_address = info.sender.to_string();
    let denom = invest.bond_denom.clone();
    let balance = deps.querier.query_balance(&env.contract.address, &denom)?.amount;
    let remainder = amount.checked_sub(balance).unwrap_or_else(|_| Uint256::zero());

    let normal = if remainder.is_zero() {
        messages.push(BankMsg::Send { to_address: to_address, amount: vec![Coin { denom,amount }] }.into());
        amount
    } else {
        
        messages.push(BankMsg::Send {
            to_address: to_address.clone(),
            amount: vec![Coin { denom: denom.clone(), amount: balance }],
        }.into());

        let delegation = deps.querier.query_all_delegations(&env.contract.address)?
            .into_iter()
            .find(|d| d.amount.amount >= remainder)
            .ok_or(ContractError::NothingToClaim {})?;

        messages.push(common::MsgTokenizeShares {
            delegator_address: env.contract.address.to_string(),
            validator_address: delegation.validator.to_string(),
            amount: Some(common::Coin {
                denom: delegation.amount.denom,
                amount: remainder.to_string(),
            }),
            tokenized_share_owner: info.sender.to_string(),
        }.to_cosmos_msg());

        balance
    };

    Ok(Response::new()
        .add_messages(messages)
        .add_attributes([
            ("action", "liquid_unbond"),
            ("normal_unbond", &normal.to_string()),
            ("liquid_unbond", &remainder.to_string()),
        ]))
}

