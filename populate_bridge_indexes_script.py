import json
import requests
from erdpy.accounts import Address, Account
from erdpy.proxy import ElrondProxy
from erdpy.transactions import Transaction, BunchOfTransactions
from erdpy import config


def int_to_hex(number: int) -> str:
    hex_nr = hex(number)[2:]
    if len(hex_nr) % 2 != 0:
        hex_nr = "0" + hex_nr
    return hex_nr


TOKEN_IDENTIFIER = "TILE-9d6c87"

with open("tiles.json") as f:
    tiles = json.load(f)

proxy = ElrondProxy("https://gateway.elrond.com")
sender = Account(pem_file="PEM_FILE_PATH")
sender.sync_nonce()
tx = Transaction()
tx.nonce = sender.nonce
tx.sender = sender.address.bech32()
tx.value = str(int(0.00 * pow(10, 18)))
tx.receiver = "CONTRACT_ADDRESS"
tx.gasPrice = 1000000000
tx.chainID = "1"  # 1 for main, T for test, D for dev
tx.data = "populateBridgeIndexes" + "@" + TOKEN_IDENTIFIER.encode().hex()
for tile in tiles:
    tx.data += "@" + int_to_hex(tile["nonce"]) + "@" + int_to_hex(int(tile["index"]))
tx.gasLimit = 300000000
tx.version = config.get_tx_version()
tx.sign(sender)
print(tx.send(proxy))
