const sendMintRandomNftTransaction = async (numberOfTokensToMint: number) => {
  const mintRandomNftTransaction = {
    value: tokenPrice * numberOfTokensToMint,
    data: 'mintRandomNft',
    receiver: contractAddress,
    gasLimit: 10000000 * numberOfTokensToMint
  };
  await refreshAccount();

  const { sessionId /*, error*/ } = await sendTransactions({
    transactions: mintRandomNftTransaction,
    transactionsDisplayInfo: {
      processingMessage: 'Message to display while transaction is processing',
      errorMessage: 'Message to display if transaction failed',
      successMessage: 'Message to display if transaction succeeded'
    },
    redirectAfterSign: false
  });
  if (sessionId != null) {
    setTransactionSessionId(sessionId);
  }
};

const sendMintSpecificNftTransaction = async (nonceOfMintedNft: number) => {
  //nonceOfMintedNft should be in hexa
  //nonceOfMintedNft should have an even number of characters
  //(pad with a 0 at the beginning if it does not)
  const mintRandomNftTransaction = {
    value: tokenPrice,
    data: `mintSpecificNft@${nonceOfMintedNft}`,
    receiver: contractAddress,
    gasLimit: 10000000
  };
  await refreshAccount();

  const { sessionId /*, error*/ } = await sendTransactions({
    transactions: mintRandomNftTransaction,
    transactionsDisplayInfo: {
      processingMessage: 'Message to display while transaction is processing',
      errorMessage: 'Message to display if transaction failed',
      successMessage: 'Message to display if transaction succeeded'
    },
    redirectAfterSign: false
  });
  if (sessionId != null) {
    setTransactionSessionId(sessionId);
  }
};