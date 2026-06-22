# Allotropy

> The same ATOM, in different forms.

**Allotropy** is a CosmWasm smart contract that brings **liquid bonding curves** to the Cosmos ecosystem. It combines the capital-efficient price discovery of bonding curves with Cosmos-native liquid staking, allowing users to buy and sell a liquid representation of staked ATOM without the 21-day unbonding period.



## Key Features

- **Dynamic Bonding Curve** — Price is determined algorithmically by supply (supports multiple curve types via `cw20-bonding`)
- **Instant or Liquid Exit** — On sell, users receive native ATOM if available, otherwise tokenized shares via liquid staking

## How It Works

### Buy Flow

1. User sends `uatom` to the contract
2. Commission is taken (if configured)
3. Remaining ATOM is staked to a validator via `StakingMsg::Delegate`
4. New tokens are minted according to the current bonding curve
5. User can see his balance using cw20 interface (or as a native token in a future tokenfactory implementation) 

### Sell Flow (Liquid Unbond)

1. User calls `Sell` with the amount of tokens to sell
2. Tokens are burned from the user's balance and removed from the total supply
3. The bonding curve calculates how much native ATOM should be released
4. The contract attempt to find regulat free ATOM that might have got accumulated from staking rewards or other sources like deliberate deposits by governing entity
5. If there is not enough free ATOM, the contract will ask the chain to issue tokenised shares directly to the user
6. The user can keep the tokenized shares and claim the rewards or initiate the unbonding process releasing tokens directky to the user (after 21 days) without any intermediaries.



## Why Bonding Curves + Liquid Staking?

Traditional bonding curves give instant liquidity and fair price discovery but usually require the reserve to sit idle. Allotropy solves this by **staking the reserve**, while still allowing users to exit instantly through Cosmos liquid staking primitives.

This creates a powerful new primitive: **liquid staked bonding curve tokens**.




