ADDRESS=erd1qqqqqqqqqqqqqpgq03yjtslvm63d3whhunycqgyg923866c7ah0s9vht8j

DEPLOY_TRANSACTION=$(mxpy data load --key=deployTransaction-mainnet)
PROXY=https://gateway.xoxno.com

deploy() {
    echo ${PROJECT}
    mxpy --verbose contract deploy --metadata-payable --bytecode="/Users/mihaieremia/GitHub/rs-ticketing/ticketing/output/ticketing.wasm" --recall-nonce  --gas-limit=500000000 \
    --arguments 0x96 \
    --send --recall-nonce --ledger --ledger-account-index=0 --ledger-address-index=0 --proxy=${PROXY} --chain=1 || return
}

upgrade() {
    mxpy --verbose contract upgrade ${ADDRESS} --metadata-payable --bytecode="/Users/mihaieremia/GitHub/rs-ticketing/ticketing/output/ticketing.wasm" \
    --recall-nonce --recall-nonce --ledger --ledger-account-index=0 --ledger-address-index=0 \
    --gas-limit=150000000 --send --outfile="upgrade.json" --proxy=${PROXY} --chain=1 || return
}