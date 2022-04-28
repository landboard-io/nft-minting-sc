import json
import logging, sys
from erdpy import config
from erdpy.transactions import Transaction, BunchOfTransactions
from erdpy.accounts import Address, Account
from erdpy.proxy import ElrondProxy

logger = logging.getLogger("transactions")


def initialize_proxy(proxy_url="https://gateway.elrond.com"):
    return ElrondProxy(proxy_url)


def initialize_wallet_from_pem_file(proxy, pem_file_path="wallets/wallet"):
    sender = Account(pem_file=pem_file_path)
    sender.sync_nonce(proxy)
    return sender


def int_to_hex(number: int) -> str:
    hex_nr = hex(number)[2:]
    if len(hex_nr) % 2 != 0:
        hex_nr = "0" + hex_nr
    return hex_nr


def main():

    txs = BunchOfTransactions()
    proxy = initialize_proxy("https://devnet-gateway.elrond.com")
    sender = initialize_wallet_from_pem_file(proxy, "wallet.pem")

    with open("new_data.json") as snapshot:
        data = json.load(snapshot)

    for key, value in data.items():
        tx = Transaction()
        tx.nonce = sender.nonce
        tx.sender = sender.address.bech32()
        tx.value = str(int(0.00 * pow(10, 18)))
        tx.receiver = "CONTRACT ADDRESS HERE"
        tx.gasPrice = 1000000000
        tx.gasLimit = 10000000 * value  # 80000000
        tx.data = f"giveaway@{int_to_hex(value)}@{Address(key).hex()}"
        tx.chainID = "D"  # 1 for main, T for test, D for dev
        tx.version = config.get_tx_version()
        tx.sign(sender)
        sender.nonce += 1
        txs.add_prepared(tx)
    [num_sent, hashes] = txs.send(proxy)
    print(hashes)


if __name__ == "__main__":
    try:
        main()
    except Exception as err:
        logger.critical(err)
        sys.exit(1)
