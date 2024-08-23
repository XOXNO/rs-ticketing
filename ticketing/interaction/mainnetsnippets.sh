ADDRESS=

DEPLOY_TRANSACTION=$(mxpy data load --key=deployTransaction-devnet)
PROXY=https://gateway.xoxno.com

deploy() {
    echo ${PROJECT}
    mxpy --verbose contract deploy --metadata-payable --bytecode="/Users/mihaieremia/GitHub/rs-ticketing/ticketing/output/minter.wasm" --recall-nonce  --gas-limit=100000000 \
    --send --recall-nonce --ledger --ledger-account-index=0 --ledger-address-index=0 --proxy=${PROXY} --chain=1 || return
}

upgrade() {
    mxpy --verbose contract upgrade ${ADDRESS} --metadata-payable --bytecode="/Users/mihaieremia/GitHub/rs-ticketing/ticketing/output/minter.wasm" \
    --recall-nonce --recall-nonce --ledger --ledger-account-index=0 --ledger-address-index=0 \
    --gas-limit=125000000 --send --outfile="upgrade.json" --proxy=${PROXY} --chain=1 || return
}