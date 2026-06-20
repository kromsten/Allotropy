#!/bin/bash


FROM=${FROM:-t}
ADDR=$(gaiad keys show $FROM -a)
NODE=${NODE:-http://localhost:26657}
CHAIN_ID=${CHAIN_ID:-gaia_9001-1}


run_gaiad() {
    gaiad "$@" --from $FROM -y --gas-prices 0.1uatom --gas 600000 --node $NODE --chain-id $CHAIN_ID
}
