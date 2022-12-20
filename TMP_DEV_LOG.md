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

CODE ID: 4792


## SWAPROUTER
Code ID: 4484
Contract: osmo1lktz0qj8cfs5mm20qlm9s9d9p5j4l5437u8663umy3mps46weggqfmtllk
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

`osmosisd tx wasm execute osmo1fm349tq2lnwkmzcgwzkvwmvvrhskhk2cepfugxs5cnfsx3663xysdzh6qg '{"swap": {"input_coin":{"denom":"uosmo","amount":"637870"},"output_denom":"uion","slippage":{"max_slippage_percentage":"99.999999999"}}}' --from main --amount 637870uosmo`
#### thats one percent slippage^


### Swap output

code: 0
codespace: ""
data: 0A91010A242F636F736D7761736D2E7761736D2E76312E4D736745786563757465436F6E747261637412690A677B226F726967696E616C5F73656E646572223A226F736D6F317A6E747136686175387935353571763372617A707539786E7270796A64747439347566616879222C22746F6B656E5F6F75745F64656E6F6D223A2275696F6E222C22616D6F756E74223A2236227D
events:
- attributes:
  - index: true
    key: ZmVl
    value: null
  type: tx
- attributes:
  - index: true
    key: YWNjX3NlcQ==
    value: b3NtbzF6bnRxNmhhdTh5NTU1cXYzcmF6cHU5eG5ycHlqZHR0OTR1ZmFoeS80Nw==
  type: tx
- attributes:
  - index: true
    key: c2lnbmF0dXJl
    value: N0dLYnZ5RGorcmYrT1U0WGU4QUV3Q3BHQWxjbzg0SFlIZmdpVlo2M0tKNUMrY3RGdFppdHNuNjg1bjUyc1V3aDBUNmVTS3JwTFd6L3IvYjFEOHVzR1E9PQ==
  type: tx
- attributes:
  - index: true
    key: YWN0aW9u
    value: L2Nvc213YXNtLndhc20udjEuTXNnRXhlY3V0ZUNvbnRyYWN0
  type: message
- attributes:
  - index: true
    key: bW9kdWxl
    value: d2FzbQ==
  - index: true
    key: c2VuZGVy
    value: b3NtbzF6bnRxNmhhdTh5NTU1cXYzcmF6cHU5eG5ycHlqZHR0OTR1ZmFoeQ==
  type: message
- attributes:
  - index: true
    key: c3BlbmRlcg==
    value: b3NtbzF6bnRxNmhhdTh5NTU1cXYzcmF6cHU5eG5ycHlqZHR0OTR1ZmFoeQ==
  - index: true
    key: YW1vdW50
    value: NjM3ODcwdW9zbW8=
  type: coin_spent
- attributes:
  - index: true
    key: cmVjZWl2ZXI=
    value: b3NtbzE0MzJjM2R4MmU4NnYzN3ZmdTNlbWZmdjVma2duaGp6ZTA3NXZwcXE3N2Y5ZHpzdWptdXNxMHN5M3lj
  - index: true
    key: YW1vdW50
    value: NjM3ODcwdW9zbW8=
  type: coin_received
- attributes:
  - index: true
    key: cmVjaXBpZW50
    value: b3NtbzE0MzJjM2R4MmU4NnYzN3ZmdTNlbWZmdjVma2duaGp6ZTA3NXZwcXE3N2Y5ZHpzdWptdXNxMHN5M3lj
  - index: true
    key: c2VuZGVy
    value: b3NtbzF6bnRxNmhhdTh5NTU1cXYzcmF6cHU5eG5ycHlqZHR0OTR1ZmFoeQ==
  - index: true
    key: YW1vdW50
    value: NjM3ODcwdW9zbW8=
  type: transfer
- attributes:
  - index: true
    key: X2NvbnRyYWN0X2FkZHJlc3M=
    value: b3NtbzE0MzJjM2R4MmU4NnYzN3ZmdTNlbWZmdjVma2duaGp6ZTA3NXZwcXE3N2Y5ZHpzdWptdXNxMHN5M3lj
  type: execute
- attributes:
  - index: true
    key: X2NvbnRyYWN0X2FkZHJlc3M=
    value: b3NtbzE0MzJjM2R4MmU4NnYzN3ZmdTNlbWZmdjVma2duaGp6ZTA3NXZwcXE3N2Y5ZHpzdWptdXNxMHN5M3lj
  - index: true
    key: YWN0aW9u
    value: dHJhZGVfd2l0aF9zbGlwcGFnZV9saW1pdA==
  type: wasm
- attributes:
  - index: true
    key: c3BlbmRlcg==
    value: b3NtbzE0MzJjM2R4MmU4NnYzN3ZmdTNlbWZmdjVma2duaGp6ZTA3NXZwcXE3N2Y5ZHpzdWptdXNxMHN5M3lj
  - index: true
    key: YW1vdW50
    value: NjM3ODcwdW9zbW8=
  type: coin_spent
- attributes:
  - index: true
    key: cmVjZWl2ZXI=
    value: b3NtbzE1MDBoeTc1a3JzOWU4dDUwYWF2NmZhaGs4c3hoYWpuOWN0cDQwcXd2dm44dGNwcmtrNndzenVuNGE1
  - index: true
    key: YW1vdW50
    value: NjM3ODcwdW9zbW8=
  type: coin_received
- attributes:
  - index: true
    key: cmVjaXBpZW50
    value: b3NtbzE1MDBoeTc1a3JzOWU4dDUwYWF2NmZhaGs4c3hoYWpuOWN0cDQwcXd2dm44dGNwcmtrNndzenVuNGE1
  - index: true
    key: c2VuZGVy
    value: b3NtbzE0MzJjM2R4MmU4NnYzN3ZmdTNlbWZmdjVma2duaGp6ZTA3NXZwcXE3N2Y5ZHpzdWptdXNxMHN5M3lj
  - index: true
    key: YW1vdW50
    value: NjM3ODcwdW9zbW8=
  type: transfer
- attributes:
  - index: true
    key: c3BlbmRlcg==
    value: b3NtbzE1MDBoeTc1a3JzOWU4dDUwYWF2NmZhaGs4c3hoYWpuOWN0cDQwcXd2dm44dGNwcmtrNndzenVuNGE1
  - index: true
    key: YW1vdW50
    value: NnVpb24=
  type: coin_spent
- attributes:
  - index: true
    key: cmVjZWl2ZXI=
    value: b3NtbzE0MzJjM2R4MmU4NnYzN3ZmdTNlbWZmdjVma2duaGp6ZTA3NXZwcXE3N2Y5ZHpzdWptdXNxMHN5M3lj
  - index: true
    key: YW1vdW50
    value: NnVpb24=
  type: coin_received
- attributes:
  - index: true
    key: cmVjaXBpZW50
    value: b3NtbzE0MzJjM2R4MmU4NnYzN3ZmdTNlbWZmdjVma2duaGp6ZTA3NXZwcXE3N2Y5ZHpzdWptdXNxMHN5M3lj
  - index: true
    key: c2VuZGVy
    value: b3NtbzE1MDBoeTc1a3JzOWU4dDUwYWF2NmZhaGs4c3hoYWpuOWN0cDQwcXd2dm44dGNwcmtrNndzenVuNGE1
  - index: true
    key: YW1vdW50
    value: NnVpb24=
  type: transfer
- attributes:
  - index: true
    key: bW9kdWxl
    value: Z2FtbQ==
  - index: true
    key: c2VuZGVy
    value: b3NtbzE0MzJjM2R4MmU4NnYzN3ZmdTNlbWZmdjVma2duaGp6ZTA3NXZwcXE3N2Y5ZHpzdWptdXNxMHN5M3lj
  - index: true
    key: cG9vbF9pZA==
    value: Mg==
  - index: true
    key: dG9rZW5zX2lu
    value: NjM3ODcwdW9zbW8=
  - index: true
    key: dG9rZW5zX291dA==
    value: NnVpb24=
  type: token_swapped
- attributes:
  - index: true
    key: X2NvbnRyYWN0X2FkZHJlc3M=
    value: b3NtbzE0MzJjM2R4MmU4NnYzN3ZmdTNlbWZmdjVma2duaGp6ZTA3NXZwcXE3N2Y5ZHpzdWptdXNxMHN5M3lj
  type: reply
- attributes:
  - index: true
    key: X2NvbnRyYWN0X2FkZHJlc3M=
    value: b3NtbzE0MzJjM2R4MmU4NnYzN3ZmdTNlbWZmdjVma2duaGp6ZTA3NXZwcXE3N2Y5ZHpzdWptdXNxMHN5M3lj
  - index: true
    key: dG9rZW5fb3V0X2Ftb3VudA==
    value: Ng==
  type: wasm
- attributes:
  - index: true
    key: c3BlbmRlcg==
    value: b3NtbzE0MzJjM2R4MmU4NnYzN3ZmdTNlbWZmdjVma2duaGp6ZTA3NXZwcXE3N2Y5ZHpzdWptdXNxMHN5M3lj
  - index: true
    key: YW1vdW50
    value: NnVpb24=
  type: coin_spent
- attributes:
  - index: true
    key: cmVjZWl2ZXI=
    value: b3NtbzF6bnRxNmhhdTh5NTU1cXYzcmF6cHU5eG5ycHlqZHR0OTR1ZmFoeQ==
  - index: true
    key: YW1vdW50
    value: NnVpb24=
  type: coin_received
- attributes:
  - index: true
    key: cmVjaXBpZW50
    value: b3NtbzF6bnRxNmhhdTh5NTU1cXYzcmF6cHU5eG5ycHlqZHR0OTR1ZmFoeQ==
  - index: true
    key: c2VuZGVy
    value: b3NtbzE0MzJjM2R4MmU4NnYzN3ZmdTNlbWZmdjVma2duaGp6ZTA3NXZwcXE3N2Y5ZHpzdWptdXNxMHN5M3lj
  - index: true
    key: YW1vdW50
    value: NnVpb24=
  type: transfer
gas_used: "211089"
gas_wanted: "350000"
height: "8134578"
info: ""
logs:
- events:
  - attributes:
    - key: receiver
      value: osmo1432c3dx2e86v37vfu3emffv5fkgnhjze075vpqq77f9dzsujmusq0sy3yc
    - key: amount
      value: 637870uosmo
    - key: receiver
      value: osmo1500hy75krs9e8t50aav6fahk8sxhajn9ctp40qwvvn8tcprkk6wszun4a5
    - key: amount
      value: 637870uosmo
    - key: receiver
      value: osmo1432c3dx2e86v37vfu3emffv5fkgnhjze075vpqq77f9dzsujmusq0sy3yc
    - key: amount
      value: 6uion
    - key: receiver
      value: osmo1zntq6hau8y555qv3razpu9xnrpyjdtt94ufahy
    - key: amount
      value: 6uion
    type: coin_received
  - attributes:
    - key: spender
      value: osmo1zntq6hau8y555qv3razpu9xnrpyjdtt94ufahy
    - key: amount
      value: 637870uosmo
    - key: spender
      value: osmo1432c3dx2e86v37vfu3emffv5fkgnhjze075vpqq77f9dzsujmusq0sy3yc
    - key: amount
      value: 637870uosmo
    - key: spender
      value: osmo1500hy75krs9e8t50aav6fahk8sxhajn9ctp40qwvvn8tcprkk6wszun4a5
    - key: amount
      value: 6uion
    - key: spender
      value: osmo1432c3dx2e86v37vfu3emffv5fkgnhjze075vpqq77f9dzsujmusq0sy3yc
    - key: amount
      value: 6uion
    type: coin_spent
  - attributes:
    - key: _contract_address
      value: osmo1432c3dx2e86v37vfu3emffv5fkgnhjze075vpqq77f9dzsujmusq0sy3yc
    type: execute
  - attributes:
    - key: action
      value: /cosmwasm.wasm.v1.MsgExecuteContract
    - key: module
      value: wasm
    - key: sender
      value: osmo1zntq6hau8y555qv3razpu9xnrpyjdtt94ufahy
    type: message
  - attributes:
    - key: _contract_address
      value: osmo1432c3dx2e86v37vfu3emffv5fkgnhjze075vpqq77f9dzsujmusq0sy3yc
    type: reply
  - attributes:
    - key: module
      value: gamm
    - key: sender
      value: osmo1432c3dx2e86v37vfu3emffv5fkgnhjze075vpqq77f9dzsujmusq0sy3yc
    - key: pool_id
      value: "2"
    - key: tokens_in
      value: 637870uosmo
    - key: tokens_out
      value: 6uion
    type: token_swapped
  - attributes:
    - key: recipient
      value: osmo1432c3dx2e86v37vfu3emffv5fkgnhjze075vpqq77f9dzsujmusq0sy3yc
    - key: sender
      value: osmo1zntq6hau8y555qv3razpu9xnrpyjdtt94ufahy
    - key: amount
      value: 637870uosmo
    - key: recipient
      value: osmo1500hy75krs9e8t50aav6fahk8sxhajn9ctp40qwvvn8tcprkk6wszun4a5
    - key: sender
      value: osmo1432c3dx2e86v37vfu3emffv5fkgnhjze075vpqq77f9dzsujmusq0sy3yc
    - key: amount
      value: 637870uosmo
    - key: recipient
      value: osmo1432c3dx2e86v37vfu3emffv5fkgnhjze075vpqq77f9dzsujmusq0sy3yc
    - key: sender
      value: osmo1500hy75krs9e8t50aav6fahk8sxhajn9ctp40qwvvn8tcprkk6wszun4a5
    - key: amount
      value: 6uion
    - key: recipient
      value: osmo1zntq6hau8y555qv3razpu9xnrpyjdtt94ufahy
    - key: sender
      value: osmo1432c3dx2e86v37vfu3emffv5fkgnhjze075vpqq77f9dzsujmusq0sy3yc
    - key: amount
      value: 6uion
    type: transfer
  - attributes:
    - key: _contract_address
      value: osmo1432c3dx2e86v37vfu3emffv5fkgnhjze075vpqq77f9dzsujmusq0sy3yc
    - key: action
      value: trade_with_slippage_limit
    - key: _contract_address
      value: osmo1432c3dx2e86v37vfu3emffv5fkgnhjze075vpqq77f9dzsujmusq0sy3yc
    - key: token_out_amount
      value: "6"
    type: wasm
  log: ""
  msg_index: 0
raw_log: '[{"events":[{"type":"coin_received","attributes":[{"key":"receiver","value":"osmo1432c3dx2e86v37vfu3emffv5fkgnhjze075vpqq77f9dzsujmusq0sy3yc"},{"key":"amount","value":"637870uosmo"},{"key":"receiver","value":"osmo1500hy75krs9e8t50aav6fahk8sxhajn9ctp40qwvvn8tcprkk6wszun4a5"},{"key":"amount","value":"637870uosmo"},{"key":"receiver","value":"osmo1432c3dx2e86v37vfu3emffv5fkgnhjze075vpqq77f9dzsujmusq0sy3yc"},{"key":"amount","value":"6uion"},{"key":"receiver","value":"osmo1zntq6hau8y555qv3razpu9xnrpyjdtt94ufahy"},{"key":"amount","value":"6uion"}]},{"type":"coin_spent","attributes":[{"key":"spender","value":"osmo1zntq6hau8y555qv3razpu9xnrpyjdtt94ufahy"},{"key":"amount","value":"637870uosmo"},{"key":"spender","value":"osmo1432c3dx2e86v37vfu3emffv5fkgnhjze075vpqq77f9dzsujmusq0sy3yc"},{"key":"amount","value":"637870uosmo"},{"key":"spender","value":"osmo1500hy75krs9e8t50aav6fahk8sxhajn9ctp40qwvvn8tcprkk6wszun4a5"},{"key":"amount","value":"6uion"},{"key":"spender","value":"osmo1432c3dx2e86v37vfu3emffv5fkgnhjze075vpqq77f9dzsujmusq0sy3yc"},{"key":"amount","value":"6uion"}]},{"type":"execute","attributes":[{"key":"_contract_address","value":"osmo1432c3dx2e86v37vfu3emffv5fkgnhjze075vpqq77f9dzsujmusq0sy3yc"}]},{"type":"message","attributes":[{"key":"action","value":"/cosmwasm.wasm.v1.MsgExecuteContract"},{"key":"module","value":"wasm"},{"key":"sender","value":"osmo1zntq6hau8y555qv3razpu9xnrpyjdtt94ufahy"}]},{"type":"reply","attributes":[{"key":"_contract_address","value":"osmo1432c3dx2e86v37vfu3emffv5fkgnhjze075vpqq77f9dzsujmusq0sy3yc"}]},{"type":"token_swapped","attributes":[{"key":"module","value":"gamm"},{"key":"sender","value":"osmo1432c3dx2e86v37vfu3emffv5fkgnhjze075vpqq77f9dzsujmusq0sy3yc"},{"key":"pool_id","value":"2"},{"key":"tokens_in","value":"637870uosmo"},{"key":"tokens_out","value":"6uion"}]},{"type":"transfer","attributes":[{"key":"recipient","value":"osmo1432c3dx2e86v37vfu3emffv5fkgnhjze075vpqq77f9dzsujmusq0sy3yc"},{"key":"sender","value":"osmo1zntq6hau8y555qv3razpu9xnrpyjdtt94ufahy"},{"key":"amount","value":"637870uosmo"},{"key":"recipient","value":"osmo1500hy75krs9e8t50aav6fahk8sxhajn9ctp40qwvvn8tcprkk6wszun4a5"},{"key":"sender","value":"osmo1432c3dx2e86v37vfu3emffv5fkgnhjze075vpqq77f9dzsujmusq0sy3yc"},{"key":"amount","value":"637870uosmo"},{"key":"recipient","value":"osmo1432c3dx2e86v37vfu3emffv5fkgnhjze075vpqq77f9dzsujmusq0sy3yc"},{"key":"sender","value":"osmo1500hy75krs9e8t50aav6fahk8sxhajn9ctp40qwvvn8tcprkk6wszun4a5"},{"key":"amount","value":"6uion"},{"key":"recipient","value":"osmo1zntq6hau8y555qv3razpu9xnrpyjdtt94ufahy"},{"key":"sender","value":"osmo1432c3dx2e86v37vfu3emffv5fkgnhjze075vpqq77f9dzsujmusq0sy3yc"},{"key":"amount","value":"6uion"}]},{"type":"wasm","attributes":[{"key":"_contract_address","value":"osmo1432c3dx2e86v37vfu3emffv5fkgnhjze075vpqq77f9dzsujmusq0sy3yc"},{"key":"action","value":"trade_with_slippage_limit"},{"key":"_contract_address","value":"osmo1432c3dx2e86v37vfu3emffv5fkgnhjze075vpqq77f9dzsujmusq0sy3yc"},{"key":"token_out_amount","value":"6"}]}]}]'
timestamp: "2022-12-13T16:53:43Z"
tx:
  '@type': /cosmos.tx.v1beta1.Tx
  auth_info:
    fee:
      amount: []
      gas_limit: "350000"
      granter: ""
      payer: ""
    signer_infos:
    - mode_info:
        single:
          mode: SIGN_MODE_DIRECT
      public_key:
        '@type': /cosmos.crypto.secp256k1.PubKey
        key: AvnAZYftukL5YQXi8ABr+tUIcmKiCKKvgmhs72It+E+c
      sequence: "47"
  body:
    extension_options: []
    memo: ""
    messages:
    - '@type': /cosmwasm.wasm.v1.MsgExecuteContract
      contract: osmo1432c3dx2e86v37vfu3emffv5fkgnhjze075vpqq77f9dzsujmusq0sy3yc
      funds:
      - amount: "637870"
        denom: uosmo
      msg:
        swap:
          input_coin:
            amount: "637870"
            denom: uosmo
          output_denom: uion
          slippage:
            max_slippage_percentage: "0.99999999999"
      sender: osmo1zntq6hau8y555qv3razpu9xnrpyjdtt94ufahy
    non_critical_extension_options: []
    timeout_height: "0"
  signatures:
  - 7GKbvyDj+rf+OU4Xe8AEwCpGAlco84HYHfgiVZ63KJ5C+ctFtZitsn685n52sUwh0T6eSKrpLWz/r/b1D8usGQ==
txhash: 299D6E3F2D37D4CCADC56B4E77E4249AFF670EA23F20FD70CC04BE2660A602BA