FROM=${FROM:-t}
ADDR=$(gaiad keys show $FROM -a)
NODE=${NODE:-http://localhost:26657}
CHAIN_ID=${CHAIN_ID:-gaia_9001-1}

# Function to run xiond commands with common args
# gas value can be removed to be lowered down individually
run_gaiad() {
    gaiad "$@" --from $FROM -y --gas-prices 0.1uatom --gas 5000000 --node $NODE --chain-id $CHAIN_ID
}


# ID: 1; Gas used 2,937,050
echo "📁 Storing example.wasm..."
run_xiond tx wasm store artifacts/example
sleep 1.5

