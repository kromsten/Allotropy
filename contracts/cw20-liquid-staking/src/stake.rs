use cosmwasm_std::{
    Addr, BankMsg, Coin, DepsMut, DistributionMsg, Env, MessageInfo, Response, StakingMsg, Uint128, Uint256, coin, CosmosMsg
};
use cw20_staking::state::INVESTMENT;
use crate::{msg::ContractError};

// Primitives for cw20-liquid-staking


pub fn liquid_unbond(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint256,
) -> Result<Response, ContractError> {
    let invest = INVESTMENT.load(deps.storage)?;
    
    let mut messages: Vec<CosmosMsg> = vec![];
    let to_send = if amount > Uint256::zero() {
        // Calculate the actual amount we can send if we have it natively
        let balance = deps.querier.query_balance(&env.contract.address, &invest.bond_denom)?.amount;
        let withdrawable = std::cmp::min(amount, Uint256::from(balance));
        
        if withdrawable > Uint256::zero() {
            messages.push(BankMsg::Send {
                to_address: info.sender.to_string(),
                amount: vec![Coin { denom: invest.bond_denom.clone(), amount: withdrawable }],
            }.into());
        }
        withdrawable
    } else {
        Uint256::zero()
    };
    
    let remainder = amount.checked_sub(to_send).unwrap_or_else(|_| Uint256::zero());
    if !remainder.is_zero() {
        let delegation = deps.querier.query_all_delegations(&env.contract.address)?
            .into_iter()
            .find(|d| d.amount.amount >= remainder)
            .ok_or(ContractError::NothingToClaim {})?;
        
        let msg = common::MsgTokenizeShares {
            delegator_address: env.contract.address.to_string(),
            validator_address: delegation.validator.to_string(),
            amount: Some(common::Coin {
                denom: delegation.amount.denom.clone(),
                amount: remainder.to_string(),
            }),
            tokenized_share_owner: info.sender.to_string(),
        };
        messages.push(msg.to_cosmos_msg());
    }

    
    let res = Response::new()
        .add_messages(messages)
        .add_attribute("action", "liquid_unbond")
        .add_attribute("withdrawn", to_send)
        .add_attribute("remainder_to_handle", remainder);

    Ok(res)
}
