use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Binary, Decimal, Uint128};
use cw20::Expiration;


#[cw_serde]
pub struct InstantiateMsg {
    /// name of the supply token
    pub name: String,
    /// symbol / ticker of the supply token
    pub symbol: String,
    /// number of decimal places of the supply token, needed for proper curve math.
    /// If it is eg. BTC, where a balance of 10^8 means 1 BTC, then use 8 here.
    pub decimals: u8,

    /// this is the reserve token denom (only support native for now)
    pub reserve_denom: String,
    /// number of decimal places for the reserve token, needed for proper curve math.
    /// Same format as decimals above, eg. if it is uatom, where 1 unit is 10^-6 ATOM, use 6 here
    pub reserve_decimals: u8,

    /// enum to store the curve parameters used for this contract
    /// if you want to add a custom Curve, you should make a new contract that imports this one.
    /// write a custom `instantiate`, and then dispatch `your::execute` -> `cw20_bonding::do_execute`
    /// with your custom curve as a parameter (and same with `query` -> `do_query`)
    pub curve_type: cw20_bonding::msg::CurveType,

    /// Commission rate (e.g. 0.05 for 5%)
    pub commission_rate: Decimal,
    /// Address to receive the commission fees
    pub commission_recipient: String,
}


#[cw_serde]
pub enum ExecuteMsg {
    /// Buy will attempt to purchase as many supply tokens as possible.
    /// You must send only reserve tokens in that message
    Buy {},

    /// Implements CW20. Transfer is a base message to move tokens to another account without triggering actions
    Transfer { recipient: String, amount: Uint128 },
    /// Implements CW20. Burn is a base message to destroy tokens forever
    Burn { amount: Uint128 },
    /// Implements CW20.  Send is a base message to transfer tokens to a contract and trigger an action
    /// on the receiving contract.
    Send {
        contract: String,
        amount: Uint128,
        msg: Binary,
    },
    /// Implements CW20 "approval" extension. Allows spender to access an additional amount tokens
    /// from the owner's (env.sender) account. If expires is Some(), overwrites current allowance
    /// expiration with this one.
    IncreaseAllowance {
        spender: String,
        amount: Uint128,
        expires: Option<Expiration>,
    },
    /// Implements CW20 "approval" extension. Lowers the spender's access of tokens
    /// from the owner's (env.sender) account by amount. If expires is Some(), overwrites current
    /// allowance expiration with this one.
    DecreaseAllowance {
        spender: String,
        amount: Uint128,
        expires: Option<Expiration>,
    },
    /// Implements CW20 "approval" extension. Transfers amount tokens from owner -> recipient
    /// if `env.sender` has sufficient pre-approval.
    TransferFrom {
        owner: String,
        recipient: String,
        amount: Uint128,
    },
    /// Implements CW20 "approval" extension. Sends amount tokens from owner -> contract
    /// if `env.sender` has sufficient pre-approval.
    SendFrom {
        owner: String,
        contract: String,
        amount: Uint128,
        msg: Binary,
    },
    /// Implements CW20 "approval" extension. Destroys tokens forever
    BurnFrom { owner: String, amount: Uint128 },
    
    /// Execute a message on behalf of another user (admin only)
    CallFor {
        /// Address of the user on whose behalf to execute
        sender: String,
        /// The message to execute
        msg: Box<ExecuteMsg>,
    },
}