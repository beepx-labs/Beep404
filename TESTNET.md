
## Build
```
docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/workspace-optimizer-arm64:0.15.0
```

## Deploy
```

injectived tx wasm store ./artifacts/cw404-aarch64.wasm \
--from="inj12qy3algm6e0zdpv8zxvauzquumuvd39ccdcdjt" \
--chain-id="injective-888" \
--yes --gas-prices=500000000inj --gas=20000000 \
--node=https://testnet.sentry.tm.injective.network:443


```

## Codes
Testnet
CW404 - 7200

Mainnet




## Init

### Core Init
```
export INJ_ADDRESS=inj12qy3algm6e0zdpv8zxvauzquumuvd39ccdcdjt
export CODE_ID=7200
export INIT='{"name":"404", "symbol": "404", "decimals": 18, "total_native_supply": "10000"}'
injectived tx wasm instantiate $CODE_ID $INIT --label="Test" --from=$(echo $INJ_ADDRESS) --chain-id="injective-888" --yes --gas-prices=500000000inj --gas=20000000 --admin=$(echo $INJ_ADDRESS) --node=https://testnet.sentry.tm.injective.network:443
```

### Migrate
```sh
export CONTRACT=inj1634vsflevg040sx4yhr3pva5t0wwlqzdlkk5v2
export INJ_ADDRESS=inj12qy3algm6e0zdpv8zxvauzquumuvd39ccdcdjt
export CONFIG='{}'
export CODE_ID=7264
injectived tx wasm migrate $CONTRACT $CODE_ID "$CONFIG" --from=$(echo $INJ_ADDRESS) \
--chain-id="injective-888" \
--yes --fees=1000000000000000inj --gas=2000000 \
--node="https://testnet.sentry.tm.injective.network:443" \
--output json
```

### Factory Settings
```sh
# send some native tokens as the factory contract will check for fund existence
injectived tx bank send inj12qy3algm6e0zdpv8zxvauzquumuvd39ccdcdjt inj1pc2vxcmnyzawnwkf03n2ggvt997avtuwagqngk 1peggy0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599 --node=https://testnet.sentry.tm.injective.network:443 --chain-id="injective-888" --gas-prices=500000000inj



# init native token
export CONTRACT=inj1634vsflevg040sx4yhr3pva5t0wwlqzdlkk5v2
export INJ_ADDRESS=inj12qy3algm6e0zdpv8zxvauzquumuvd39ccdcdjt
CONFIG='{"set_whitelist":{"target": "inj12qy3algm6e0zdpv8zxvauzquumuvd39ccdcdjt", "state": true}}'
CONFIG='{"set_whitelist":{"target": "inj1gc99d6upv5q93cy33xahalsh2jxledu3hq8f65", "state": true}}'
injectived tx wasm execute $CONTRACT "$CONFIG" --from=$(echo $INJ_ADDRESS) \
--chain-id="injective-888" \
--yes --fees=1000000000000000inj --gas=2000000 \
--node="https://testnet.sentry.tm.injective.network:443" \
--output json
```

### Staking Init
```sh


# sends reward tokens to contract
export CONTRACT=inj16l3txcsmjcs6nrc3s0se0388r39j8wn73n45cy
export CONFIG='{"send":{"contract": "inj18j3tn5hrf3uex5lw2egp5epl6xuwnmu2rt2k0z", "amount": "10000000000000000000000", "msg": ""}}'
injectived tx wasm execute $CONTRACT "$CONFIG" --from=$(echo $INJ_ADDRESS) \
--chain-id="injective-888" \
--yes --fees=1000000000000000inj --gas=2000000 \
--node="https://testnet.sentry.tm.injective.network:443" \
--output json

```

### Create New Pair
```sh
# Allowance for adding liq
export CONTRACT=inj1634vsflevg040sx4yhr3pva5t0wwlqzdlkk5v2
export INJ_ADDRESS=inj12qy3algm6e0zdpv8zxvauzquumuvd39ccdcdjt
export CONFIG='{"increase_allowance":{"spender": "inj1gc99d6upv5q93cy33xahalsh2jxledu3hq8f65", "amount": "100000000000000000000000000"}}'
injectived tx wasm execute $CONTRACT "$CONFIG" --from=$(echo $INJ_ADDRESS) \
--chain-id="injective-888" \
--yes --fees=1000000000000000inj --gas=2000000 \
--node="https://testnet.sentry.tm.injective.network:443" \
--output json

# Denom-CW20 pair
export CONTRACT=inj14mxpetzg9sur0g6m39zu9m9n2ajxvlx4ytlgq3
export INJ_ADDRESS=inj12qy3algm6e0zdpv8zxvauzquumuvd39ccdcdjt
export CONFIG='{"create_pair":{"assets":[{"info":{"token":{"contract_addr":"inj1634vsflevg040sx4yhr3pva5t0wwlqzdlkk5v2"}},"amount":"0"},{"info":{"native_token":{"denom":"inj"}},"amount":"0"}]}}'
injectived tx wasm execute $CONTRACT "$CONFIG" --from=$(echo $INJ_ADDRESS) \
--chain-id="injective-888" \
--yes --fees=1000000000000000inj --gas=2000000 \
--node="https://testnet.sentry.tm.injective.network:443" \
--output json


### Provide Liquidity
```sh
export CONTRACT=inj1gc99d6upv5q93cy33xahalsh2jxledu3hq8f65
export INJ_ADDRESS=inj12qy3algm6e0zdpv8zxvauzquumuvd39ccdcdjt
export CONFIG='{"provide_liquidity":{"assets":[{"info":{"token":{"contract_addr":"inj1634vsflevg040sx4yhr3pva5t0wwlqzdlkk5v2"}},"amount":"1000000000000000000000"},{"info":{"native_token":{"denom":"inj"}},"amount":"1000000000000000000"}]}}'
injectived tx wasm execute $CONTRACT "$CONFIG" --from=$(echo $INJ_ADDRESS) \
--chain-id="injective-888" \
--yes --fees=1000000000000000inj --gas=2000000 \
--node="https://testnet.sentry.tm.injective.network:443" \
--amount="1000000000000000000inj" \
--output json
```

### Swap
```sh
export CONTRACT=inj1gc99d6upv5q93cy33xahalsh2jxledu3hq8f65
export INJ_ADDRESS=inj12qy3algm6e0zdpv8zxvauzquumuvd39ccdcdjt
export CONFIG='{"swap":{"offer_asset":{"info":{"native_token":{"denom":"inj"}},"amount":"100000000000000000000"}}}'
injectived tx wasm execute $CONTRACT "$CONFIG" --from=$(echo $INJ_ADDRESS) \
--chain-id="injective-888" \
--yes --fees=1000000000000000inj --gas=2000000 \
--node="https://testnet.sentry.tm.injective.network:443" \
--amount="100000000000000000000inj" \
--output json
```

### Addresses
Testnet
Factory - inj14mxpetzg9sur0g6m39zu9m9n2ajxvlx4ytlgq3
Router - inj18em4c6qvzc6kymk98p5j7dxdm4at6d9qy8wxqv
Staking - 
Multicall - inj1q54g9rhprqd5v87we3rrjhp2etpxx2w6qys6ku


Sushi_INJ contract - inj1gc99d6upv5q93cy33xahalsh2jxledu3hq8f65


DojoToken - inj16l3txcsmjcs6nrc3s0se0388r39j8wn73n45cy
TestToken - inj1e8ppkcdttmvqywcx84rjqf0l2x9gcutlmft4l0
TestToken - inj16ncdyyhwl7gz6q6vvz447fyn70myc22vw45zvt
Launchpad - inj1h6m6erreyp72fthqj5esxa25nre4arg65g5us9

Mainnet

Factory - inj1pc2vxcmnyzawnwkf03n2ggvt997avtuwagqngk
PointToken - inj1l73x8hh6du0h8upp65r7ltzpj5twadtp5490n0

#### Get Config
```
injectived query wasm contract-state smart inj14mxpetzg9sur0g6m39zu9m9n2ajxvlx4ytlgq3 '{"config": {}}' --node=https://testnet.sentry.tm.injective.network:443


injectived query wasm contract-state smart inj14mxpetzg9sur0g6m39zu9m9n2ajxvlx4ytlgq3 '{"pairs": {}}' --node=https://testnet.sentry.tm.injective.network:443

injectived query wasm contract-state smart inj14mxpetzg9sur0g6m39zu9m9n2ajxvlx4ytlgq3 '{"pair": {"asset_infos": [{"token": { "contract_addr": "inj1375v9e5awxf340cgv2pzh4seu074lxw0d092gd" }}, {"native_token": {"denom": "inj"}} ] }}' --node=https://testnet.sentry.tm.injective.network:443

injectived query wasm contract-state smart inj14mxpetzg9sur0g6m39zu9m9n2ajxvlx4ytlgq3 '{"pair": {"asset_infos": [{"native_token": { "denom": "peggy0x87aB3B4C8661e07D6372361211B96ed4Dc36B1B5" }}, {"native_token": {"denom": "inj"}} ] }}' --node=https://testnet.sentry.tm.injective.network:443

injectived query wasm contract-state smart inj14mxpetzg9sur0g6m39zu9m9n2ajxvlx4ytlgq3 '{"native_token_decimals": {"denom": "peggy0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2"}}' --node=https://testnet.sentry.tm.injective.network:443

injectived query wasm contract-state smart inj1l73x8hh6du0h8upp65r7ltzpj5twadtp5490n0 '{"minter": {}}' --node=https://testnet.sentry.tm.injective.network:443

injectived query wasm contract-state smart inj1vdx02lw5ay5wpu03s6rpnkadc5chyyxx7khrlf '{"get_user": {"address":"inj1x26aln79hxrfm8c6v00208wlumrc0l6qtrtn8y"}}' --node=https://testnet.sentry.tm.injective.network:443

injectived query wasm contract-state smart inj1634vsflevg040sx4yhr3pva5t0wwlqzdlkk5v2 '{"balance": {"address": "inj12qy3algm6e0zdpv8zxvauzquumuvd39ccdcdjt"}}' --node=https://testnet.sentry.tm.injective.network:443

injectived query wasm contract-state smart inj1634vsflevg040sx4yhr3pva5t0wwlqzdlkk5v2 '{"owner_of": {"token_id": "3"}}' --node=https://testnet.sentry.tm.injective.network:443

injectived query wasm contract-state smart inj1634vsflevg040sx4yhr3pva5t0wwlqzdlkk5v2 '{"num_tokens": {}}' --node=https://testnet.sentry.tm.injective.network:443
```

### Launchpad
```sh
export CONTRACT=inj1vdx02lw5ay5wpu03s6rpnkadc5chyyxx7khrlf
export INJ_ADDRESS=inj12qy3algm6e0zdpv8zxvauzquumuvd39ccdcdjt
export CONFIG='{"update_config": {"offering_token": "inj16ncdyyhwl7gz6q6vvz447fyn70myc22vw45zvt"}}'
injectived tx wasm execute $CONTRACT "$CONFIG" --from=$(echo $INJ_ADDRESS) \
--chain-id="injective-888" \
--yes --fees=1000000000000000inj --gas=2000000 \
--node=https://testnet.sentry.tm.injective.network:443 \
--output json

export CONTRACT=inj1vdx02lw5ay5wpu03s6rpnkadc5chyyxx7khrlf
export INJ_ADDRESS=inj12qy3algm6e0zdpv8zxvauzquumuvd39ccdcdjt
export CONFIG='{"flip_allow_claim": {}}'
injectived tx wasm execute $CONTRACT "$CONFIG" --from=$(echo $INJ_ADDRESS) \
--chain-id="injective-888" \
--yes --fees=1000000000000000inj --gas=2000000 \
--node=https://testnet.sentry.tm.injective.network:443 \
--output json

export CONTRACT=inj1vdx02lw5ay5wpu03s6rpnkadc5chyyxx7khrlf
export INJ_ADDRESS=inj12qy3algm6e0zdpv8zxvauzquumuvd39ccdcdjt
export CONFIG='{"final_withdraw": {"raise_amount":"1000000000000000000", "offer_amount": "1000"}}'
injectived tx wasm execute $CONTRACT "$CONFIG" --from=$(echo $INJ_ADDRESS) \
--chain-id="injective-888" \
--yes --fees=1000000000000000inj --gas=2000000 \
--node=https://testnet.sentry.tm.injective.network:443 \
--output json
```

injectived query wasm contract-state smart inj1vdx02lw5ay5wpu03s6rpnkadc5chyyxx7khrlf '{"get_user_amount": {"address":"inj1x26aln79hxrfm8c6v00208wlumrc0l6qtrtn8y"}}' --node=https://testnet.sentry.tm.injective.network:443


