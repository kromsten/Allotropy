use crate::contract::{execute, instantiate};
use crate::msg::{ExecuteMsg, InstantiateMsg};
use crate::state::CONFIG;
use cosmwasm_std::testing::{mock_dependencies, mock_env};
use cosmwasm_std::{coins, Addr, BankMsg, Decimal, Uint128, MessageInfo};

#[test]
fn test_instantiation_and_buy_commission() {
    let mut deps = mock_dependencies();
    let env = mock_env();

    let owner = deps.api.addr_make("creator").to_string();
    let treasury = deps.api.addr_make("treasury").to_string();
    let buyer = deps.api.addr_make("buyer").to_string();

    // 1. Test Instantiation with valid commission config
    let instantiate_msg = InstantiateMsg {
        name: "Liquid Bond Token".to_string(),
        symbol: "LBT".to_string(),
        decimals: 6,
        reserve_denom: "ushell".to_string(),
        reserve_decimals: 6,
        curve_type: cw20_bonding::msg::CurveType::Constant {
            value: Uint128::new(100), // 1.0 ratio
            scale: 2,
        },
        commission_rate: Decimal::percent(5), // 5% fee commission
        commission_recipient: treasury.clone(),
    };

    let info = MessageInfo {
        sender: Addr::unchecked(&owner),
        funds: vec![],
    };
    let res = instantiate(deps.as_mut(), env.clone(), info, instantiate_msg).unwrap();
    assert_eq!(0, res.messages.len());

    // Verify Config is stored correctly
    let config = CONFIG.load(&deps.storage).unwrap();
    assert_eq!(config.commission_rate, Decimal::percent(5));
    assert_eq!(config.commission_recipient, Addr::unchecked(&treasury));

    // 2. Test Buy Operation with commission
    // Buyer sends 1000 ushell
    let funds = coins(1000, "ushell");
    let info = MessageInfo {
        sender: Addr::unchecked(&buyer),
        funds,
    };
    let execute_msg = ExecuteMsg::Buy {};

    let res = execute(deps.as_mut(), env.clone(), info, execute_msg).unwrap();

    // We expect:
    // - One message: BankMsg::Send 50 ushell to treasury (5% of 1000)
    assert_eq!(res.messages.len(), 1);

    // Verify the target bank message
    let expected_bank_msg = BankMsg::Send {
        to_address: treasury.clone(),
        amount: coins(50, "ushell"),
    };
    assert_eq!(res.messages[0].msg, expected_bank_msg.into());

    // Verify commission attributes are saved
    assert_eq!(
        res.attributes.iter().find(|a| a.key == "commission").unwrap().value,
        "50"
    );
    assert_eq!(
        res.attributes.iter().find(|a| a.key == "commission_recipient").unwrap().value,
        treasury
    );

    // Verify buyer got 950 LBT (1000 - 50 commission = 950, constant ratio is 1 LBT = 1 ushell)
    // Let's verify balance by querying or looking at BALANCES in cw20-base
    let balance = cw20_base::state::BALANCES.load(&deps.storage, &Addr::unchecked(&buyer)).unwrap();
    assert_eq!(balance, Uint128::new(950));

    // Verify reserve in CurveState is 950 ushell
    let curve_state = cw20_bonding::state::CURVE_STATE.load(&deps.storage).unwrap();
    assert_eq!(curve_state.reserve, Uint128::new(950));
    assert_eq!(curve_state.supply, Uint128::new(950));
}

#[test]
fn test_instantiation_invalid_commission() {
    let mut deps = mock_dependencies();
    let env = mock_env();

    let owner = deps.api.addr_make("creator").to_string();
    let treasury = deps.api.addr_make("treasury").to_string();

    let instantiate_msg = InstantiateMsg {
        name: "Liquid Bond Token".to_string(),
        symbol: "LBT".to_string(),
        decimals: 6,
        reserve_denom: "ushell".to_string(),
        reserve_decimals: 6,
        curve_type: cw20_bonding::msg::CurveType::Constant {
            value: Uint128::new(100),
            scale: 2,
        },
        commission_rate: Decimal::percent(105), // Invalid > 100%
        commission_recipient: treasury.to_string(),
    };

    let info = MessageInfo {
        sender: Addr::unchecked(&owner),
        funds: vec![],
    };
    let err = instantiate(deps.as_mut(), env, info, instantiate_msg).unwrap_err();
    match err {
        crate::error::ContractError::InvalidCommissionRate {} => {}
        _ => panic!("Expected InvalidCommissionRate error"),
    }
}