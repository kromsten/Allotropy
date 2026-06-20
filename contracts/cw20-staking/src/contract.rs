#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use std::convert::TryInto;
use cosmwasm_std::{
    coin, to_json_binary, Addr, BankMsg, Binary, Decimal, Deps, DepsMut, DistributionMsg, Env,
    MessageInfo, QuerierWrapper, Response, StakingMsg, StdResult, Uint128, WasmMsg,
};

use cw2::set_contract_version;
use cw20_base::allowances::{
    execute_burn_from, execute_decrease_allowance, execute_increase_allowance, execute_send_from,
    execute_transfer_from, query_allowance,
};
use cw20_base::contract::{
    execute_burn, execute_mint, execute_send, execute_transfer, query_balance, query_token_info,
};
use cw20_base::state::{MinterData, TokenInfo, TOKEN_INFO};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, InvestmentResponse, QueryMsg};
use crate::state::{InvestmentInfo, Supply, CLAIMS, INVESTMENT, TOTAL_SUPPLY};

const FALLBACK_RATIO: Decimal = Decimal::one();

// version info for migration info
pub const CONTRACT_NAME: &str = "crates.io:cw20-staking";
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    #[cfg(not(feature = "library"))]
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // ensure the validator is registered
    let vals = deps.querier.query_all_validators()?;
    if !vals.iter().any(|v| v.address == msg.validator) {
        return Err(ContractError::NotInValidatorSet {
            validator: msg.validator,
        });
    }

    // store token info using cw20-base format
    let data = TokenInfo {
        name: msg.name,
        symbol: msg.symbol,
        decimals: msg.decimals,
        total_supply: Uint128::zero(),
        // set self as minter, so we can properly execute mint and burn
        mint: Some(MinterData {
            minter: env.contract.address,
            cap: None,
        }),
    };
    TOKEN_INFO.save(deps.storage, &data)?;

    let denom = deps.querier.query_bonded_denom()?;
    let invest = InvestmentInfo {
        owner: info.sender,
        exit_tax: msg.exit_tax,
        unbonding_period: msg.unbonding_period,
        bond_denom: denom,
        validator: msg.validator,
        min_withdrawal: msg.min_withdrawal,
    };
    INVESTMENT.save(deps.storage, &invest)?;

    // set supply to 0
    let supply = Supply::default();
    TOTAL_SUPPLY.save(deps.storage, &supply)?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Bond {} => bond(deps, env, info),
        ExecuteMsg::Unbond { amount } => unbond(deps, env, info, amount),
        ExecuteMsg::Claim {} => claim(deps, env, info),
        ExecuteMsg::Reinvest {} => reinvest(deps, env, info),
        ExecuteMsg::_BondAllTokens {} => _bond_all_tokens(deps, env, info),

        // these all come from cw20-base to implement the cw20 standard
        ExecuteMsg::Transfer { recipient, amount } => {
            Ok(execute_transfer(deps, env, info, recipient, amount)?)
        }
        ExecuteMsg::Burn { amount } => Ok(execute_burn(deps, env, info, amount)?),
        ExecuteMsg::Send {
            contract,
            amount,
            msg,
        } => Ok(execute_send(deps, env, info, contract, amount, msg)?),
        ExecuteMsg::IncreaseAllowance {
            spender,
            amount,
            expires,
        } => Ok(execute_increase_allowance(
            deps, env, info, spender, amount, expires,
        )?),
        ExecuteMsg::DecreaseAllowance {
            spender,
            amount,
            expires,
        } => Ok(execute_decrease_allowance(
            deps, env, info, spender, amount, expires,
        )?),
        ExecuteMsg::TransferFrom {
            owner,
            recipient,
            amount,
        } => Ok(execute_transfer_from(
            deps, env, info, owner, recipient, amount,
        )?),
        ExecuteMsg::BurnFrom { owner, amount } => {
            Ok(execute_burn_from(deps, env, info, owner, amount)?)
        }
        ExecuteMsg::SendFrom {
            owner,
            contract,
            amount,
            msg,
        } => Ok(execute_send_from(
            deps, env, info, owner, contract, amount, msg,
        )?),
    }
}

// get_bonded returns the total amount of delegations from contract
// it ensures they are all the same denom
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

pub fn bond(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    // ensure we have the proper denom
    let invest = INVESTMENT.load(deps.storage)?;
    // payment finds the proper coin (or throws an error)
    let payment = info
        .funds
        .iter()
        .find(|x| x.denom == invest.bond_denom)
        .ok_or_else(|| ContractError::EmptyBalance {
            denom: invest.bond_denom.clone(),
        })?;
    let payment_amount: Uint128 = payment.amount.try_into()?;

    // bonded is the total number of tokens we have delegated from this address
    let bonded = get_bonded(&deps.querier, &env.contract.address)?;

    // calculate to_mint and update total supply
    let mut supply = TOTAL_SUPPLY.load(deps.storage)?;
    // TODO: this is just a safety assertion - do we keep it, or remove caching?
    // in the end supply is just there to cache the (expected) results of get_bonded() so we don't
    // have expensive queries everywhere
    assert_bonds(&supply, bonded)?;
    let to_mint = if supply.issued.is_zero() || bonded.is_zero() {
        payment_amount.mul_floor(FALLBACK_RATIO)
    } else {
        payment_amount.multiply_ratio(supply.issued, bonded)
    };
    supply.bonded = bonded + payment_amount;
    supply.issued += to_mint;
    TOTAL_SUPPLY.save(deps.storage, &supply)?;

    // call into cw20-base to mint the token, call as self as no one else is allowed
    let sub_info = MessageInfo {
        sender: env.contract.address.clone(),
        funds: vec![],
    };
    execute_mint(deps, env, sub_info, info.sender.to_string(), to_mint)?;

    // bond them to the validator
    let res = Response::new()
        .add_message(StakingMsg::Delegate {
            validator: invest.validator,
            amount: payment.clone(),
        })
        .add_attribute("action", "bond")
        .add_attribute("from", info.sender)
        .add_attribute("bonded", payment.amount)
        .add_attribute("minted", to_mint);
    Ok(res)
}



pub fn unbond(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let invest = INVESTMENT.load(deps.storage)?;
    // ensure it is big enough to care
    if amount < invest.min_withdrawal {
        return Err(ContractError::UnbondTooSmall {
            min_bonded: invest.min_withdrawal,
            denom: invest.bond_denom,
        });
    }
    // calculate tax and remainer to unbond
    let tax = amount.mul_floor(invest.exit_tax);

    // burn from the original caller
    execute_burn(deps.branch(), env.clone(), info.clone(), amount)?;
    if tax > Uint128::zero() {
        let sub_info = MessageInfo {
            sender: env.contract.address.clone(),
            funds: vec![],
        };
        // call into cw20-base to mint tokens to owner, call as self as no one else is allowed
        execute_mint(
            deps.branch(),
            env.clone(),
            sub_info,
            invest.owner.to_string(),
            tax,
        )?;
    }

    // re-calculate bonded to ensure we have real values
    // bonded is the total number of tokens we have delegated from this address
    let bonded = get_bonded(&deps.querier, &env.contract.address)?;

    // calculate how many native tokens this is worth and update supply
    let remainder = amount.checked_sub(tax)?;
    let mut supply = TOTAL_SUPPLY.load(deps.storage)?;
    // TODO: this is just a safety assertion - do we keep it, or remove caching?
    // in the end supply is just there to cache the (expected) results of get_bonded() so we don't
    // have expensive queries everywhere
    assert_bonds(&supply, bonded)?;
    let unbond = remainder.multiply_ratio(bonded, supply.issued);
    supply.bonded = bonded.checked_sub(unbond)?;
    supply.issued = supply
        .issued
        .checked_sub(remainder)?;
    supply.claims += unbond;
    TOTAL_SUPPLY.save(deps.storage, &supply)?;

    CLAIMS.create_claim(
        deps.storage,
        &info.sender,
        unbond,
        invest.unbonding_period.after(&env.block),
    )?;

    // unbond them
    let res = Response::new()
        .add_message(StakingMsg::Undelegate {
            validator: invest.validator,
            amount: coin(unbond.u128(), &invest.bond_denom),
        })
        .add_attribute("action", "unbond")
        .add_attribute("to", info.sender)
        .add_attribute("unbonded", unbond)
        .add_attribute("burnt", amount);
    Ok(res)
}

pub fn claim(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    // find how many tokens the contract has
    let invest = INVESTMENT.load(deps.storage)?;
    let mut balance = deps
        .querier
        .query_balance(&env.contract.address, &invest.bond_denom)?;
    let balance_amount: Uint128 = balance.amount.try_into()?;
    if balance_amount < invest.min_withdrawal {
        return Err(ContractError::BalanceTooSmall {});
    }

    // check how much to send - min(balance, claims[sender]), and reduce the claim
    // Ensure we have enough balance to cover this and only send some claims if that is all we can cover
    let to_send =
        CLAIMS.claim_tokens(deps.storage, &info.sender, &env.block, Some(balance_amount))?;
    if to_send == Uint128::zero() {
        return Err(ContractError::NothingToClaim {});
    }

    // update total supply (lower claim)
    TOTAL_SUPPLY.update(deps.storage, |mut supply| -> Result<_, ContractError> {
        supply.claims = supply.claims.checked_sub(to_send)?;
        Ok(supply)
    })?;

    // transfer tokens to the sender
    balance.amount = to_send.into();
    let res = Response::new()
        .add_message(BankMsg::Send {
            to_address: info.sender.to_string(),
            amount: vec![balance],
        })
        .add_attribute("action", "claim")
        .add_attribute("from", info.sender)
        .add_attribute("amount", to_send);
    Ok(res)
}

/// reinvest will withdraw all pending rewards,
/// then issue a callback to itself via _bond_all_tokens
/// to reinvest the new earnings (and anything else that accumulated)
pub fn reinvest(deps: DepsMut, env: Env, _info: MessageInfo) -> Result<Response, ContractError> {
    let contract_addr = env.contract.address;
    let invest = INVESTMENT.load(deps.storage)?;
    let msg = to_json_binary(&ExecuteMsg::_BondAllTokens {})?;

    // and bond them to the validator
    let res = Response::new()
        .add_message(DistributionMsg::WithdrawDelegatorReward {
            validator: invest.validator,
        })
        .add_message(WasmMsg::Execute {
            contract_addr: contract_addr.to_string(),
            msg,
            funds: vec![],
        });
    Ok(res)
}

pub fn _bond_all_tokens(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    // this is just meant as a call-back to ourself
    if info.sender != env.contract.address {
        return Err(ContractError::Unauthorized {});
    }

    // find how many tokens we have to bond
    let invest = INVESTMENT.load(deps.storage)?;
    let mut balance = deps
        .querier
        .query_balance(&env.contract.address, &invest.bond_denom)?;

    // we deduct pending claims from our account balance before reinvesting.
    // if there is not enough funds, we just return a no-op
    let mut balance_amount: Uint128 = balance.amount.try_into()?;
    match TOTAL_SUPPLY.update(deps.storage, |mut supply| -> Result<_, ContractError> {
        balance_amount = balance_amount.checked_sub(supply.claims)?;
        // this just triggers the "no op" case if we don't have min_withdrawal left to reinvest
        balance_amount.checked_sub(invest.min_withdrawal)?;
        supply.bonded += balance_amount;
        Ok(supply)
    }) {
        Ok(_) => {}
        // if it is below the minimum, we do a no-op (do not revert other state from withdrawal)
        Err(ContractError::Overflow { .. }) => return Ok(Response::default()),
        Err(e) => return Err(e),
    }

    balance.amount = balance_amount.into();

    // and bond them to the validator
    let res = Response::new()
        .add_message(StakingMsg::Delegate {
            validator: invest.validator,
            amount: balance.clone(),
        })
        .add_attribute("action", "reinvest")
        .add_attribute("bonded", balance.amount);
    Ok(res)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        // custom queries
        QueryMsg::Claims { address } => {
            to_json_binary(&CLAIMS.query_claims(deps, &deps.api.addr_validate(&address)?)?)
        }
        QueryMsg::Investment {} => to_json_binary(&query_investment(deps)?),
        // inherited from cw20-base
        QueryMsg::TokenInfo {} => to_json_binary(&query_token_info(deps)?),
        QueryMsg::Balance { address } => to_json_binary(&query_balance(deps, address)?),
        QueryMsg::Allowance { owner, spender } => {
            to_json_binary(&query_allowance(deps, owner, spender)?)
        }
    }
}

pub fn query_investment(deps: Deps) -> StdResult<InvestmentResponse> {
    let invest = INVESTMENT.load(deps.storage)?;
    let supply = TOTAL_SUPPLY.load(deps.storage)?;

    let res = InvestmentResponse {
        owner: invest.owner.to_string(),
        exit_tax: invest.exit_tax,
        validator: invest.validator,
        min_withdrawal: invest.min_withdrawal,
        token_supply: supply.issued,
        staked_tokens: coin(supply.bonded.u128(), &invest.bond_denom),
        nominal_value: if supply.issued.is_zero() {
            FALLBACK_RATIO
        } else {
            Decimal::from_ratio(supply.bonded, supply.issued)
        },
    };
    Ok(res)
}
