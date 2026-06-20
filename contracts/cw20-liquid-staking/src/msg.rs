use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Coin, Decimal, Uint128};
pub use cw_controllers::ClaimsResponse;

pub type  InstantiateMsg = cw20_staking::msg::InstantiateMsg;
pub type ContractError = cw20_staking::ContractError;
pub type ExecuteMsg = cw20_staking::msg::ExecuteMsg;
pub type QueryMsg = cw20_staking::msg::QueryMsg;



#[cw_serde]
pub struct InvestmentResponse {
    pub token_supply: Uint128,
    pub staked_tokens: Coin,
    // ratio of staked_tokens / token_supply (or how many native tokens that one derivative token is nominally worth)
    pub nominal_value: Decimal,

    /// owner created the contract and takes a cut
    pub owner: String,
    /// this is how much the owner takes as a cut when someone unbonds
    pub exit_tax: Decimal,
    /// All tokens are bonded to this validator
    pub validator: String,
    /// This is the minimum amount we will pull out to reinvest, as well as a minimum
    /// that can be unbonded (to avoid needless staking tx)
    pub min_withdrawal: Uint128,
}
