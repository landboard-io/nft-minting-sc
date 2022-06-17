from erdpy.accounts import Account
from erdpy.proxy import ElrondProxy
from erdpy.transactions import Transaction, BunchOfTransactions
from erdpy import config

proxy = ElrondProxy("https://devnet-gateway.elrond.com")
sender = Account(pem_file="./wallets/test-wallet2.pem")
sender.sync_nonce(proxy)

txs = BunchOfTransactions()

for _ in range(0, 500):
    tx = Transaction()
    tx.nonce = sender.nonce
    tx.sender = sender.address.bech32()
    tx.value = str(int(0.0 * pow(10, 18)))
    tx.receiver = "erd1qqqqqqqqqqqqqpgq97pumna4lp7082djz6msrnzvchytzaav23qs0d4z5u"
    tx.gasPrice = 1000000000
    tx.gasLimit = 600000000  # 80000000
    tx.data = "depopulateIndexes"
    tx.chainID = "D"  # 1 for main, T for test, D for dev
    tx.version = config.get_tx_version()
    tx.sign(sender)
    txs.add_prepared(tx)
    sender.nonce += 1

[num_sent, hashes] = txs.send(proxy)
print(hashes)
