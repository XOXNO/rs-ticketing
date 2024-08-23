ADDRESS=erd1qqqqqqqqqqqqqpgq2np2khle66ea5nml5h2d98mkhguh2thwah0savzpnh

DEPLOY_TRANSACTION=$(mxpy data load --key=deployTransaction-devnet)
PROXY=https://devnet-gateway.xoxno.com

deploy() {
    echo ${PROJECT}
    mxpy --verbose contract deploy --metadata-payable --bytecode="/Users/mihaieremia/GitHub/rs-ticketing/ticketing/output/ticketing.wasm" --recall-nonce  --gas-limit=500000000 \
    --arguments 0x96 \
    --send --recall-nonce --ledger --ledger-account-index=0 --ledger-address-index=0 --proxy=${PROXY} --chain="D" || return
}

upgrade() {
    mxpy --verbose contract upgrade ${ADDRESS} --metadata-payable --bytecode="/Users/mihaieremia/GitHub/rs-ticketing/ticketing/output/ticketing.wasm" \
    --recall-nonce --recall-nonce --ledger --ledger-account-index=0 --ledger-address-index=0 \
    --gas-limit=125000000 --send --outfile="upgrade.json" --proxy=${PROXY} --chain="D" || return
}