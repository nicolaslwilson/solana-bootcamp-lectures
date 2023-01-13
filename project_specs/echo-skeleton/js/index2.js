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
  // const echo = "foo";

  const connection = new Connection("https://api.devnet.solana.com/");

  const feePayer = new Keypair();

  console.log("Requesting Airdrop of 1 SOL...");
  await connection.requestAirdrop(feePayer.publicKey, 2e9);
  console.log("Airdrop received");

  const idx = Buffer.from(new Uint8Array([1]));
  let utf8Encode = new TextEncoder();
  let bufferSeed = Uint8Array.from(Array(8).fill(2));

  const [authorizedBuffer, bumpSeed] = await PublicKey.findProgramAddress(
    [utf8Encode.encode("authority"), feePayer.publicKey.toBuffer(), bufferSeed],
    programId
  );
  const bufferSize = Buffer.from(new Uint8Array(new BN(17).toArray("le", 8)));

  let echoIx = new TransactionInstruction({
    keys: [
      {
        pubkey: authorizedBuffer,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: feePayer.publicKey,
        isSigner: true,
        isWritable: true,
      },
      {
        pubkey: SystemProgram.programId,
        isSigner: false,
        isWritable: false,
      },
    ],
    programId: programId,
    data: Buffer.concat([idx, bufferSeed, bufferSize]),
  });

  let tx = new Transaction();
  tx.add(echoIx);

  let txid = await sendAndConfirmTransaction(connection, tx, [feePayer], {
    skipPreflight: true,
    preflightCommitment: "confirmed",
    confirmation: "confirmed",
  });
  console.log(`https://explorer.solana.com/tx/${txid}?cluster=devnet`);

  data = (await connection.getAccountInfo(authorizedBuffer)).data;
  console.log("Echo Buffer Text:", data, data.toJSON());

  // this will error
  // let tx2 = new Transaction();
  // tx2.add(echoIx);

  // let txid2 = await sendAndConfirmTransaction(connection, tx2, [feePayer], {
  //   skipPreflight: true,
  //   preflightCommitment: "confirmed",
  //   confirmation: "confirmed",
  // });
  // console.log(`https://explorer.solana.com/tx/${txid2}?cluster=devnet`);
};

main()
  .then(() => {
    console.log("Success");
  })
  .catch((e) => {
    console.error(e);
  });
