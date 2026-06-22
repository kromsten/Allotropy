use cosmwasm_std::{StdResult, Uint128};
use cw20_bonding::{curves::{DecimalPlaces, square_root}, msg::{CurveType, ExecuteMsg as BondingExecuteMsg}};
use rust_decimal::Decimal;
use crate::{msg::ExecuteMsg, state::CURVE_TYPE};


/// Convert our local ExecuteMsg to the bonding contract's ExecuteMsg
pub fn to_bonding_msg(msg: &ExecuteMsg) -> Option<BondingExecuteMsg> {
    match msg {
        ExecuteMsg::Buy { .. } => Some(BondingExecuteMsg::Buy {}),
        ExecuteMsg::Sell { amount, .. } => Some(BondingExecuteMsg::Burn { amount: amount.clone().try_into().ok()? }),
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
    }
}


pub fn updated_curve_slope(
    storage: &mut dyn cosmwasm_std::Storage,
    curve_type: &CurveType,
    normalize: DecimalPlaces,
    reserve: &Uint128,
    supply: &Uint128
 ) -> StdResult<CurveType> {
    if supply.is_zero() {
        return Ok(curve_type.clone());
    }

    let supply = normalize.from_supply(*supply);
    let reserve = normalize.from_reserve(*reserve);

    Ok(match curve_type.clone() {
        CurveType::SquareRoot { scale, ..} => {
            let denom = supply * square_root(supply); // x^(3/2) = x * sqrt(x)
            let slope = Decimal::new(15, 1) * reserve / denom;
            let ctype = CurveType::SquareRoot { slope: normalize.to_reserve(slope), scale };
            CURVE_TYPE.save(storage, &ctype)?;
            ctype
        },
        CurveType::Linear { scale, .. } => {
            let nom = Decimal::new(2, 0) * reserve;
            let slope = nom / (supply * supply);
            let ctype = CurveType::Linear { slope: normalize.to_reserve(slope), scale };
            CURVE_TYPE.save(storage, &ctype)?;
            ctype
        },
        CurveType::Constant { scale, .. } => {
            let slope = reserve / supply;
            let ctype = CurveType::Constant { value: normalize.to_reserve(slope), scale };
            CURVE_TYPE.save(storage, &ctype )?;
            ctype
        }
    })
}