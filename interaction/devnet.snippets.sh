PROXY=https://devnet-gateway.elrond.com
CHAIN_ID="D"

WALLET="./wallets/users/heidi.pem"

#######################################################
ADDRESS=$(erdpy data load --key=address-devnet)
#######################################################

COLLECTION_NAME="LandboardTile"
COLLECTION_NAME_HEX="0x$(echo -n ${COLLECTION_NAME} | xxd -p -u | tr -d '\n')"
COLLECTION_TIKER="LBTILE"
COLLECTION_TIKER_HEX="0x$(echo -n ${COLLECTION_TIKER} | xxd -p -u | tr -d '\n')"

TOTAL_NUMBER_OF_NFTS=100

CID="CID_URL"
CID_HEX="0x$(echo -n ${CID} | xxd -p -u | tr -d '\n')"

MINT_EGLD_VALUE=1000000000000000000 # 1 EGLD

NUMBER_OF_NFTS_TO_MINT=10

PAYMENT_TOKEN_ID="EGLD"
PAYMENT_TOKEN_ID_HEX="0x$(echo -n ${PAYMENT_TOKEN_ID} | xxd -p -u | tr -d '\n')"
PAYMENT_TOKEN_AMOUNT=100000000000000000 # 0.1 EGLD

MAX_PER_TX=20

deploy() {
    erdpy --verbose contract deploy \
    --project=${PROJECT} --pem=${WALLET} --recall-nonce --send --proxy=${PROXY} --chain=${CHAIN_ID} \
    --outfile="deploy-devnet.interaction.json" \
    --metadata-payable --gas-limit=100000000

    ADDRESS=$(erdpy data parse --file="deploy-devnet.interaction.json" --expression="data['contractAddress']")
    erdpy data store --key=address-devnet --value=${ADDRESS}

    echo ""
    echo "Smart contract address: ${ADDRESS}"
}

issueToken() {
    erdpy --verbose contract call ${ADDRESS} --send --proxy=${PROXY} --chain=${CHAIN_ID} --recall-nonce --pem=${WALLET} \
    --gas-limit=100000000 \
    --function="issueToken" \
    --value 50000000000000000 \
    --arguments ${COLLECTION_NAME_HEX} ${COLLECTION_TIKER_HEX}
}

setLocalRoles() {
    erdpy --verbose contract call ${ADDRESS} --send --proxy=${PROXY} --chain=${CHAIN_ID} --recall-nonce --pem=${WALLET} \
    --gas-limit=100000000 \
    --function="setLocalRoles"
}

populateIndexes() {
    erdpy --verbose contract call ${ADDRESS} --send --proxy=${PROXY} --chain=${CHAIN_ID} --recall-nonce --pem=${WALLET} \
    --gas-limit=100000000 \
    --function="populateIndexes" \
    --arguments ${TOTAL_NUMBER_OF_NFTS}
}

setCid() {
    erdpy --verbose contract call ${ADDRESS} --send --proxy=${PROXY} --chain=${CHAIN_ID} --recall-nonce --pem=${WALLET} \
    --gas-limit=100000000 \
    --function="setCid" \
    --arguments ${CID_HEX}
}

setPrice() {
    erdpy --verbose contract call ${ADDRESS} --send --proxy=${PROXY} --chain=${CHAIN_ID} --recall-nonce --pem=${WALLET} \
    --gas-limit=100000000 \
    --function="setPrice" \
    --arguments ${PAYMENT_TOKEN_ID_HEX} ${PAYMENT_TOKEN_AMOUNT}
}

setMaxPerTx() {
    erdpy --verbose contract call ${ADDRESS} --send --proxy=${PROXY} --chain=${CHAIN_ID} --recall-nonce --pem=${WALLET} \
    --gas-limit=100000000 \
    --function="setMaxPerTx" \
    --arguments ${MAX_PER_TX}
}

pause() {
    erdpy --verbose contract call ${ADDRESS} --send --proxy=${PROXY} --chain=${CHAIN_ID} --recall-nonce --pem=${WALLET} \
    --gas-limit=100000000 \
    --function="pause"
}

mintRandomNft() {
    erdpy --verbose contract call ${ADDRESS} --send --proxy=${PROXY} --chain=${CHAIN_ID} --recall-nonce --pem=${WALLET} \
    --gas-limit=100000000 \
    --function="mintRandomNft" \
    --value ${MINT_EGLD_VALUE}
}

mintSpecificNft() {
    erdpy --verbose contract call ${ADDRESS} --send --proxy=${PROXY} --chain=${CHAIN_ID} --recall-nonce --pem=${WALLET} \
    --gas-limit=100000000 \
    --function="mintSpecificNft" \
    --value ${MINT_EGLD_VALUE}  \
    --arguments ${NUMBER_OF_NFTS_TO_MINT}
}