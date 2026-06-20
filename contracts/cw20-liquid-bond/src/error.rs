use cosmwasm_std::{OverflowError, StdError};
use cw_utils::PaymentError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Base(#[from] cw20_base::ContractError),

    #[error("{0}")]
    Bonding(#[from] cw20_bonding::ContractError),

    #[error("{0}")]
    Overflow(#[from] OverflowError),

    #[error("{0}")]
    Payment(#[from] PaymentError),

    #[error("{0}")]
    Admin(#[from] cw_controllers::AdminError),

    #[error("The reserve denom must match the staking denom")]
    DenomMismatch {},

    #[error("Commission rate must be between 0 and 1")]
    InvalidCommissionRate {},

    #[error("BiasedCurve nesting is not allowed")]
    BiasedCurveNesting {},

    #[error("Balance is not big enough. Found: {0}, Required: {1}")]
    NoBalance(String, u128),

    #[error("Not enough funds in the reserve. Expected: {0}, Found: {1}")]
    NoReserve(u128, u128),

    #[error("Needed to issue {0} tokens with the current reserve can only issue {1}")]
    NoSupply(u128, u128),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Nested CallFor is not allowed")]
    NestedCallFor {},
}