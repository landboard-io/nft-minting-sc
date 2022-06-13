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
resp = requests.get(
    f"https://api.elrond.com/collections/{TOKEN_IDENTIFIER}/nfts?size=10000"
).json()
tiles = [
    {"nonce": tile["nonce"], "index": tile["name"].split("#")[-1]} for tile in resp
]
output_file = open(f"tiles.json", "a")
output_file.truncate(0)
output_file.write(json.dumps(tiles, sort_keys=False, indent=4))
output_file.close
