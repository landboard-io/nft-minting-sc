import json
import requests
from erdpy.accounts import Address, Account
from erdpy.proxy import ElrondProxy
from erdpy.transactions import Transaction, BunchOfTransactions
from erdpy import config
from mrbp.sendTransactions import TOKEN_IDENTIFIER


def int_to_hex(number: int) -> str:
    hex_nr = hex(number)[2:]
    if len(hex_nr) % 2 != 0:
        hex_nr = "0" + hex_nr
    return hex_nr


TOKEN_IDENTIFIER = "TILE-9d6c87"
resp = requests.get(
    f"https://api.elrond.com/collections/{TOKEN_IDENTIFIER}/nfts?size=10000"
).json()
tiles = [
    {"nonce": tile["nonce"], "index": tile["name"].split("#")[-1]} for tile in resp
]
output_file = open(f"landboard/tiles.json", "a")
output_file.truncate(0)
output_file.write(json.dumps(tiles, sort_keys=False, indent=4))
output_file.close


proxy = ElrondProxy("https://gateway.elrond.com")
sender = Account(pem_file="PEM_FILE_PATH")
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
print(tx.send())
