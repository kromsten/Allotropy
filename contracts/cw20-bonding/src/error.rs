use cosmwasm_std::StdError;
use cw_utils::PaymentError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Base(#[from] cw20_base::ContractError),

    #[error("{0}")]
    Overflow(#[from] cosmwasm_std::ConversionOverflowError),

    #[error("{0}")]
    Payment(#[from] PaymentError),

    #[error("Unauthorized")]
    Unauthorized {},
}

#[cfg(not(target_arch = "wasm32"))]
impl PartialEq for ContractError {
    fn eq(&self, other: &Self) -> bool {
        self.to_string() == other.to_string()
    }
}
