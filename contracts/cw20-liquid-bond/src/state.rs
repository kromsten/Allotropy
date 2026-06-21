use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Decimal};
use cw_storage_plus::Item;
use cw_controllers::Admin;
use cw20_bonding::msg::CurveType;


#[cw_serde]
pub struct Config {
    pub com_rate: Decimal,
    pub com_recipient: Addr,
}

pub const CONFIG: Item<Config> = Item::new("config");

/// CURVE_TYPE storage item using the CurveType enum from cw20-bonding
/// Uses the same storage key as the original for compatibility during migration
pub const CURVE_TYPE: Item<CurveType> = Item::new("curve_type");

/// Contract admin controller
pub const ADMIN: Admin = Admin::new("admin");


pub const VALIDATORS: Item<Vec<String>> = Item::new("vs");


pub const BURNED_TOTAL: Item<u128> = Item::new("burned_total");