use cw20_bonding::msg::ExecuteMsg as BondingExecuteMsg;
use crate::msg::ExecuteMsg;


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
        ExecuteMsg::CallFor { .. } => None,
    }
}
