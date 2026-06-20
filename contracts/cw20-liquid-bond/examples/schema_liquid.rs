use cosmwasm_schema::write_api;
use cw20_bonding::msg::{ExecuteMsg, QueryMsg};
use cw20_liquid_bond::msg::InstantiateMsg;

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        execute: ExecuteMsg,
        query: QueryMsg,
    }
}
