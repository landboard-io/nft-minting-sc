const buildNftPayLoad = (nft: any) => {
  const hexid = Buffer.from(nft.collection).toString('hex');
  let hexnonce = parseInt(nft.nonce).toString(16);
  if (hexnonce.length % 2 !== 0) {
    hexnonce = '0' + hexnonce;
  }
  return '@' + hexid + '@' + hexnonce + '@01';
}


const BridgeExample = () => {

  const [bridgeNfts, setBridgeNfts] = useState([]);

  useEffect(() => {
    if (isLoggedIn) {
      axios
        .get(
          `${api}/accounts/${address}/nfts?search=${tokenIdentifier}&size=100`
        )
        .then((res) => {
          setBridgeNfts(res.data);
        });
    }
  }, []);


  const sendBridgeTransaction = async () => {
    if (selectedWalletNfts.length > 0) {
      let hexamount = bridgeNfts.length.toString(16);
      if (hexamount.length % 2 !== 0) {
        hexamount = '0' + hexamount;
      }
      let data =
        'MultiESDTNFTTransfer' +
        '@' +
        new Address(stakeContractAddress).hex() +
        '@' +
        hexamount;
      bridgeNfts.forEach((nft) => {
        data += buildNftPayLoad(nft);
      });
      data += '@' + Buffer.from('bridgeNfts').toString('hex');
      const bridgeNftTransaction = new Transaction({
        value: 0,
        data: new TransactionPayload(data),
        receiver: new Address(contractAddress),
        gasLimit: 20000000 * (bridgeNfts.length + 1),
        chainID: '1'
      });
      await refreshAccount();
      const { sessionId, error } = await sendTransactions({
        transactions: bridgeNftTransaction,
        transactionsDisplayInfo: {
          processingMessage: 'Bridging tiles to V2...',
          errorMessage: 'Error occured during tile bridging',
          successMessage: 'Tiles bridged successfully'
        },
        redirectAfterSign: false
      });
      if (sessionId != null) {
        setTransactionSessionId(sessionId);
      }
    }
  };
}