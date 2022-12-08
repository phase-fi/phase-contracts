# Describing get_upcoming_swaps query and how to test

## Upload contract
quick reminder to upload code using these commands:
`cd contracts/pf-dca && RUSTFLAGS='-C link-arg=-s' cargo wasm && cd ../..`
`osmosisd tx wasm store target/wasm32-unknown-unknown/release/pf_dca.wasm --from main -y --output json -b block  --node https://rpc.testnet.osmosis.zone:443 --chain-id osmo-test-4 --gas-prices 10uosmo --gas auto --gas-adjustment 1.3`

## Instantiate contract and create task
Croncat instantiation now works. You need to enter this command to instantiate
`osmosisd tx wasm instantiate REPLACE_WITH_CODE_ID '{"amount_per_trade":"1","cron":"1 * * * * *","destinations":[{"denom":"juno","weight":"1"}],"num_trades":"5","platform_fee":"0","platform_wallet":"osmo1pwyl8cg7wskhvsl8hwmu20j0j2qcf52k2vph8w","strategy_type":"linear"}' --node https://rpc.testnet.osmosis.zone:443 --chain-id osmo-test-4 --from REPLACE_WITH_YOUR_WALLET --label "phase finance boi" --gas-prices 0.025uosmo --gas auto --gas-adjustment 1.3 -b block -y --no-admin --amount 54330uosmo`

You may need to change the amount (54330uosmo) to a higher number if you change the above message to more trades, or a higher amount per trade, or changes are made to the perform_dca function, etc. It depends on how much croncat charges per execution. It is pretty interesting to play around with params, croncat will give great error messages if too little is given. 

*everything is hardcoded right now in estimate_croncat_funding, lots to do there.*

## After successful instantiation you can query upcoming swaps
Currently if you are querying more than 5 minutes after the croncat instantiation, you will get an error, since our dca will have already ended (5 trades, one per minute).

*Actually, this is not implemented yet, so idt you will get an error, we can implement by adding the correct boundary to the croncat create_task call*

`osmosisd query wasm  contract-state smart REPLACE_WITH_YOUR_DEPLOYED_CONTRACT_ADDRESS '{"get_upcoming_swap":{}}' --node https://rpc.testnet.osmosis.zone:443 --chain-id osmo-test-4`



gas_base_fee + gas_action_fee*actiions.length


## CW_DEX_ROUTER

### Init route
`osmosisd tx wasm execute osmo1fm349tq2lnwkmzcgwzkvwmvvrhskhk2cepfugxs5cnfsx3663xysdzh6qg '{"set_route": {"input_denom":"uosmo","output_denom":"uion","pool_route":[{"pool_id":"2","token_out_denom":"uion"}]}}' --from main`

```
{
    "set_route": {
        "input_denom": "uosmo",
        "output_denom": "uion",
        "pool_route": [
            {
                "pool_id": "2",
                "token_out_denom": "uion"
            }
        ]
    }
}
```

### Swap

`osmosisd tx wasm execute osmo1fm349tq2lnwkmzcgwzkvwmvvrhskhk2cepfugxs5cnfsx3663xysdzh6qg '{"swap": {"input_coin":{"denom":"uosmo","amount":"637870"},"output_denom":"uion","slipage":{"max_slipage_percentage":"0.9"}}}' --from main --amount 637870uosmo`
#### thats one percent slippage^