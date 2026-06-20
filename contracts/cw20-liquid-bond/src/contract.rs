#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use cosmwasm_std::{
    ensure, Binary, Decimal, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult, Uint128, BankMsg
};
use cw2::set_contract_version;
use cw20_bonding::curves::DecimalPlaces;
use cw_utils::nonpayable;

use crate::error::ContractError;
use crate::msg::{InstantiateMsg, ExecuteMsg};
use crate::state::{ADMIN, CURVE_TYPE, Config, CONFIG};
use cw20_base::state::{TOKEN_INFO, TokenInfo, MinterData};
use cw20_bonding::state::{CURVE_STATE, CurveState};

// entrypoints of the contract
use cw20_bonding::contract::{do_execute, do_query};
use cw20_bonding::msg::{ExecuteMsg as BondingExecuteMsg, QueryMsg};


// version info for migration info
pub const CONTRACT_NAME: &str = "crates.io:cw20-liquid-bond";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");


/// Convert our local ExecuteMsg to the bonding contract's ExecuteMsg
fn to_bonding_msg(msg: &ExecuteMsg) -> Option<BondingExecuteMsg> {
    match msg {
        ExecuteMsg::Buy {} => Some(BondingExecuteMsg::Buy {}),
        ExecuteMsg::Transfer { recipient, amount } => Some(BondingExecuteMsg::Transfer { 
            recipient: recipient.clone(), 
            amount: *amount 
        }),
        ExecuteMsg::Burn { amount } => Some(BondingExecuteMsg::Burn { amount: *amount }),
        ExecuteMsg::Send { contract, amount, msg } => Some(BondingExecuteMsg::Send { 
            contract: contract.clone(), 
            amount: *amount, 
            msg: msg.clone() 
        }),
        ExecuteMsg::IncreaseAllowance { spender, amount, expires } => {
            Some(BondingExecuteMsg::IncreaseAllowance { 
                spender: spender.clone(), 
                amount: *amount, 
                expires: *expires 
            })
        },
        ExecuteMsg::DecreaseAllowance { spender, amount, expires } => {
            Some(BondingExecuteMsg::DecreaseAllowance { 
                spender: spender.clone(), 
                amount: *amount, 
                expires: *expires 
            })
        },
        ExecuteMsg::TransferFrom { owner, recipient, amount } => {
            Some(BondingExecuteMsg::TransferFrom { 
                owner: owner.clone(), 
                recipient: recipient.clone(), 
                amount: *amount 
            })
        },
        ExecuteMsg::SendFrom { owner, contract, amount, msg } => {
            Some(BondingExecuteMsg::SendFrom { 
                owner: owner.clone(), 
                contract: contract.clone(), 
                amount: *amount, 
                msg: msg.clone() 
            })
        },
        ExecuteMsg::BurnFrom { owner, amount } => {
            Some(BondingExecuteMsg::BurnFrom { 
                owner: owner.clone(), 
                amount: *amount 
            })
        },
        ExecuteMsg::CallFor { .. } => None,
    }
}


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
        commission_rate: msg.commission_rate,
        commission_recipient: deps.api.addr_validate(&msg.commission_recipient)?,
    };
    CONFIG.save(deps.storage, &config)?;

    ADMIN.set(deps, Some(info.sender.clone()))?;

    Ok(Response::default())
}



#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    mut info: MessageInfo,
    mut msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    // Handle CallFor admin delegation first
    if let ExecuteMsg::CallFor { sender, msg: inner_msg } = msg {
        ADMIN.assert_admin(deps.as_ref(), &info.sender)?;
        info.sender = deps.api.addr_validate(&sender)?;
        msg = *inner_msg;
    }

    let curve_type = CURVE_TYPE.load(deps.storage)?;
    let curve_fn = curve_type.to_curve_fn();

    let res = if let ExecuteMsg::Buy {} = msg {
        let config = CONFIG.load(deps.storage)?;
        let state = CURVE_STATE.load(deps.storage)?;
        let payment: Uint128 = cw_utils::must_pay(&info, &state.reserve_denom)?
            .try_into()
            .map_err(|_| StdError::msg("amount conversion overflow"))?;

        let commission_amount: Uint128 = Decimal::from_ratio(payment, 1u32)
            .checked_mul(config.commission_rate)?
            .to_uint_floor();
        let buy_amount = payment.checked_sub(commission_amount)?;

        let mut modified_info = info.clone();
        modified_info.funds = vec![cosmwasm_std::Coin {
            denom: state.reserve_denom.clone(),
            amount: buy_amount.into(),
        }];

        let bonding_msg = to_bonding_msg(&msg).ok_or_else(|| ContractError::NestedCallFor {})?;
        let mut res = do_execute(deps, env, modified_info, bonding_msg, curve_fn)?;

        if !commission_amount.is_zero() {
            res = res.add_message(BankMsg::Send {
                to_address: config.commission_recipient.to_string(),
                amount: vec![cosmwasm_std::Coin {
                    denom: state.reserve_denom,
                    amount: commission_amount.into(),
                }],
            });
            res = res.add_attribute("commission", commission_amount)
                     .add_attribute("commission_recipient", config.commission_recipient.to_string());
        }
        res
    } else if let ExecuteMsg::CallFor { .. } = msg {
        // Prevent nested CallFor
        return Err(ContractError::NestedCallFor {});
    } else {
        let bonding_msg = to_bonding_msg(&msg).ok_or_else(|| ContractError::NestedCallFor {})?;
        do_execute(deps, env, info, bonding_msg, curve_fn)?
    };

    Ok(res)
}


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    let curve_type = CURVE_TYPE.load(deps.storage)?;
    let curve_fn = curve_type.to_curve_fn();
    do_query(deps, env, msg, curve_fn)
}