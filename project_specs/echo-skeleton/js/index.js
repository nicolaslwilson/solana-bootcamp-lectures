const {
  Connection,
  sendAndConfirmTransaction,
  Keypair,
  Transaction,
  SystemProgram,
  PublicKey,
  TransactionInstruction,
} = require("@solana/web3.js");

const BN = require("bn.js");

const main = async () => {
  // var args = process.argv.slice(2);
  // const programId = new PublicKey(args[0]);
  const programId = new PublicKey(
    "6noSzHzFuyU4VYafb8kX28iXZWaMxKkfAX5wDKrF16nT"
  );
  // const echo = args[1];
  const echo = "foo";

  const connection = new Connection("https://api.devnet.solana.com/");

  const feePayer = new Keypair();
  const echoBuffer = new Keypair();

  console.log("Requesting Airdrop of 1 SOL...");
  await connection.requestAirdrop(feePayer.publicKey, 2e9);
  console.log("Airdrop received");

  let createIx = SystemProgram.createAccount({
    fromPubkey: feePayer.publicKey,
    newAccountPubkey: echoBuffer.publicKey,
    /** Amount of lamports to transfer to the created account */
    lamports: await connection.getMinimumBalanceForRentExemption(
      echo.length + 1
    ),
    /** Amount of space in bytes to allocate to the created account */
    space: echo.length + 1,
    /** Public key of the program to assign as the owner of the created account */
    programId: programId,
  });

  const idx = Buffer.from(new Uint8Array([0]));
  const messageLen = Buffer.from(
    new Uint8Array(new BN(echo.length).toArray("le", 4))
  );
  const message = Buffer.from(echo, "ascii");

  let echoIx = new TransactionInstruction({
    keys: [
      {
        pubkey: echoBuffer.publicKey,
        isSigner: false,
        isWritable: true,
      },
    ],
    programId: programId,
    data: Buffer.concat([idx, messageLen, message]),
  });

  let tx = new Transaction();
  tx.add(createIx).add(echoIx);

  let txid = await sendAndConfirmTransaction(
    connection,
    tx,
    [feePayer, echoBuffer],
    {
      skipPreflight: true,
      preflightCommitment: "confirmed",
      confirmation: "confirmed",
    }
  );
  console.log(`https://explorer.solana.com/tx/${txid}?cluster=devnet`);

  data = (await connection.getAccountInfo(echoBuffer.publicKey)).data;
  console.log("Echo Buffer Text:", data.toString());

  let tx2 = new Transaction();
  tx2.add(echoIx);

  let txid2 = await sendAndConfirmTransaction(
    connection,
    tx2,
    [feePayer, echoBuffer],
    {
      skipPreflight: true,
      preflightCommitment: "confirmed",
      confirmation: "confirmed",
    }
  );
  console.log(`https://explorer.solana.com/tx/${txid2}?cluster=devnet`);
};

main()
  .then(() => {
    console.log("Success");
  })
  .catch((e) => {
    console.error(e);
  });
