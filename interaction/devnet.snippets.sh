PROXY=https://devnet-gateway.elrond.com
CHAIN_ID="D"

WALLET="./wallets/test-wallet.pem"

#######################################################
ADDRESS=$(erdpy data load --key=address-devnet)
DEVNET_ADDRESS=$(erdpy data load --key=address-devnet-2)
#######################################################

COLLECTION_NAME="LandboardTile"
COLLECTION_NAME_HEX="0x$(echo -n ${COLLECTION_NAME} | xxd -p -u | tr -d '\n')"
COLLECTION_TIKER="TILE"
COLLECTION_TIKER_HEX="0x$(echo -n ${COLLECTION_TIKER} | xxd -p -u | tr -d '\n')"

TOTAL_NUMBER_OF_NFTS=500

CID="QmUbuMPWK8U9Kvn4HHZJ7uyGpesH1heNiQMaWc1M9itX7Q"
CID_HEX="0x$(echo -n ${CID} | xxd -p -u | tr -d '\n')"

MINT_EGLD_VALUE=1000000000000000000 # 1 EGLD

NUMBER_OF_NFTS_TO_MINT=05
REF_PERCENT=02
DISCOUNT_PERCENT=05


# RANDOM MINT

PAYMENT_TOKEN_ID="LAND-40f26f"
PAYMENT_TOKEN_ID_HEX="$(echo -n ${PAYMENT_TOKEN_ID} | xxd -p -u | tr -d '\n')"
PAYMENT_TOKEN_AMOUNT=533000000000000000000 # 533 LAND

# PAYMENT_TOKEN_ID="LKLAND-6cf78e"
# PAYMENT_TOKEN_ID_HEX="$(echo -n ${PAYMENT_TOKEN_ID} | xxd -p -u | tr -d '\n')"
# PAYMENT_TOKEN_AMOUNT=710000000000000000000 # 533 LAND

# PAYMENT_TOKEN_ID="LKLAND-c617f7"
# PAYMENT_TOKEN_ID_HEX="$(echo -n ${PAYMENT_TOKEN_ID} | xxd -p -u | tr -d '\n')"
# PAYMENT_TOKEN_AMOUNT=710000000000000000000 # 533 LAND


#SPECIFIC MINT

# PAYMENT_TOKEN_ID="LAND-40f26f"
# PAYMENT_TOKEN_ID_HEX="$(echo -n ${PAYMENT_TOKEN_ID} | xxd -p -u | tr -d '\n')"
# PAYMENT_TOKEN_AMOUNT=800000000000000000000 # 533 LAND

# PAYMENT_TOKEN_ID="LKLAND-6cf78e"
# PAYMENT_TOKEN_ID_HEX="$(echo -n ${PAYMENT_TOKEN_ID} | xxd -p -u | tr -d '\n')"
# PAYMENT_TOKEN_AMOUNT=1080000000000000000000 # 533 LAND

# PAYMENT_TOKEN_ID="LKLAND-c617f7"
# PAYMENT_TOKEN_ID_HEX="$(echo -n ${PAYMENT_TOKEN_ID} | xxd -p -u | tr -d '\n')"
# PAYMENT_TOKEN_AMOUNT=1080000000000000000000 # 533 LAND


MINT_RANDOM="$(echo -n mintRandomNft | xxd -p -u | tr -d '\n')"
MINT_SPECIFIC="$(echo -n mintSpecificNft | xxd -p -u | tr -d '\n')"
MAX_PER_TX=32



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
upgrade() {
    erdpy --verbose contract upgrade ${DEVNET_ADDRESS} --project=${PROJECT} --recall-nonce --pem=${WALLET} --gas-limit=100000000 --send --outfile="upgrade.json" --proxy=${PROXY} --chain=${CHAIN_ID} --metadata-payable
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

setRefPercent() {
    erdpy --verbose contract call ${ADDRESS} --send --proxy=${PROXY} --chain=${CHAIN_ID} --recall-nonce --pem=${WALLET} \
    --gas-limit=100000000 \
    --function="setRefPercent" \
    --arguments ${REF_PERCENT}
}

setDiscountPercent() {
    erdpy --verbose contract call ${ADDRESS} --send --proxy=${PROXY} --chain=${CHAIN_ID} --recall-nonce --pem=${WALLET} \
    --gas-limit=100000000 \
    --function="setDiscountPercent" \
    --arguments ${DISCOUNT_PERCENT}
}

populateIndexes() {
    erdpy --verbose contract call ${ADDRESS} --send --proxy=${PROXY} --chain=${CHAIN_ID} --recall-nonce --pem=${WALLET} \
    --gas-limit=600000000 \
    --function="populateIndexes" \
    --arguments ${TOTAL_NUMBER_OF_NFTS}
    sleep 1
    
}

setCid() {
    erdpy --verbose contract call ${ADDRESS} --send --proxy=${PROXY} --chain=${CHAIN_ID} --recall-nonce --pem=${WALLET} \
    --gas-limit=100000000 \
    --function="setCid" \
    --arguments ${CID_HEX}
}


depopulateIndexes() {
   
    erdpy --verbose contract call ${ADDRESS} --send --proxy=${PROXY} --chain=${CHAIN_ID} --recall-nonce --pem=${WALLET} \
    --gas-limit=600000000 \
    --function="depopulateIndexes" 
 
}

setPrice() {
    erdpy --verbose contract call ${ADDRESS} --send --proxy=${PROXY} --chain=${CHAIN_ID} --recall-nonce --pem=${WALLET} \
    --gas-limit=100000000 \
    --function="setPrice" \
    --arguments 0x${PAYMENT_TOKEN_ID_HEX} ${PAYMENT_TOKEN_AMOUNT}
}

setSpecificPrice() {
    erdpy --verbose contract call ${ADDRESS} --send --proxy=${PROXY} --chain=${CHAIN_ID} --recall-nonce --pem=${WALLET} \
    --gas-limit=100000000 \
    --function="setSpecificPrice" \
    --arguments 0x${PAYMENT_TOKEN_ID_HEX} ${PAYMENT_TOKEN_AMOUNT}
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
    erdpy --verbose tx new --receiver ${ADDRESS} --send --proxy=${PROXY} --chain=${CHAIN_ID} --recall-nonce --pem=${WALLET} \
    --data="ESDTTransfer@${PAYMENT_TOKEN_ID_HEX}@016345785d8a0000@${MINT_RANDOM}" \
    --gas-limit=100000000 
}

mintSpecificNft() {
    erdpy --verbose tx new --receiver ${ADDRESS} --send --proxy=${PROXY} --chain=${CHAIN_ID} --recall-nonce --pem=${WALLET} \
    --gas-limit=100000000 \
    --data="ESDTTransfer@${PAYMENT_TOKEN_ID_HEX}@016345785d8a0000@${MINT_SPECIFIC}@01103E" 
}

getSftPrice() {
    erdpy --verbose contract query ${ADDRESS} --proxy=${PROXY} --function="getSftPrice" --arguments ${CALLER_ADDRESS_HEX} 1
}

