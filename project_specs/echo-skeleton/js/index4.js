const {
  Connection,
  sendAndConfirmTransaction,
  Keypair,
  Transaction,
  SystemProgram,
  PublicKey,
  TransactionInstruction,
} = require("@solana/web3.js");

const {
  createMint,
  createAssociatedTokenAccount,
  mintToChecked,
  TOKEN_PROGRAM_ID,
} = require("@solana/spl-token");

const BN = require("bn.js");
const { serialize } = require("borsh");

const {
  Payload,
  InstructionVariant,
  InitializeVendingMachineEcho,
  VendingMachineEchoPayload,
} = require("./instructions");
const { waitFor } = require("./util");

const main = async () => {
  const programId = new PublicKey(
    "6noSzHzFuyU4VYafb8kX28iXZWaMxKkfAX5wDKrF16nT"
  );

  const connection = new Connection("https://api.devnet.solana.com/");

  const feePayer = new Keypair();

  console.log("Requesting Airdrop of 1 SOL...");
  await connection.requestAirdrop(feePayer.publicKey, 2e9);
  console.log("Airdrop received");

  const price = 123;
  let utf8Encode = new TextEncoder();

  const tokenMint = await createMint(
    connection,
    feePayer,
    feePayer.publicKey,
    feePayer.publicKey,
    9,
    undefined,
    {
      skipPreflight: true,
    }
  );

  console.log("tokenMint", tokenMint.toBase58());

  let ata = await createAssociatedTokenAccount(
    connection, // connection
    feePayer, // fee payer
    tokenMint, // mint
    feePayer.publicKey, // owner
    {
      skipPreflight: true,
    }
  );

  console.log("ata", ata.toBase58());

  let txhash = await mintToChecked(
    connection, // connection
    feePayer, // fee payer
    tokenMint, // mint
    ata, // receiver (sholud be a token account)
    feePayer, // mint authority
    1e12, // amount. if your decimals is 8, you mint 10^8 for 1 token.
    9, // decimals
    [],
    {
      skipPreflight: true,
    }
  );

  console.log(
    `mint tx https://explorer.solana.com/tx/${txhash}?cluster=devnet`
  );

  const [vendingMachineBuffer, bumpSeed] = await PublicKey.findProgramAddress(
    [
      utf8Encode.encode("authority"),
      tokenMint.toBuffer(),
      new Uint8Array(new BN(price).toArray("le", 8)),
    ],
    programId
  );
  console.log(vendingMachineBuffer);

  const initPayload = Buffer.from(
    serialize(
      InitializeVendingMachineEcho,
      new Payload({
        id: InstructionVariant.InitializeVendingMachineEcho,
        price,
        buffer_size: 20,
      })
    )
  );

  // init authorized buffer account
  let initIx = new TransactionInstruction({
    keys: [
      {
        pubkey: vendingMachineBuffer,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: tokenMint,
        isSigner: false,
        isWritable: false,
      },
      {
        pubkey: feePayer.publicKey,
        isSigner: true,
        isWritable: false,
      },
      {
        pubkey: SystemProgram.programId,
        isSigner: false,
        isWritable: false,
      },
    ],
    programId: programId,
    data: initPayload,
  });

  let tx = new Transaction();
  tx.add(initIx);

  let txid = await sendAndConfirmTransaction(connection, tx, [feePayer], {
    skipPreflight: true,
    preflightCommitment: "confirmed",
    confirmation: "confirmed",
  });
  console.log(`init tx https://explorer.solana.com/tx/${txid}?cluster=devnet`);

  data = (await connection.getAccountInfo(vendingMachineBuffer)).data;
  console.log("Vending Buffer:", data, data.toJSON());

  const echoData = Buffer.from(
    serialize(
      VendingMachineEchoPayload,
      new Payload({
        id: InstructionVariant.VendingMachineEcho,
        data: "0123",
      })
    )
  );

  let echoIx = new TransactionInstruction({
    keys: [
      {
        pubkey: vendingMachineBuffer,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: feePayer.publicKey,
        isSigner: true,
        isWritable: false,
      },
      {
        pubkey: ata,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: tokenMint,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: TOKEN_PROGRAM_ID,
        isSigner: false,
        isWritable: false,
      },
    ],
    programId: programId,
    data: echoData,
  });

  let tx2 = new Transaction();
  tx2.add(echoIx);

  let txid2 = await sendAndConfirmTransaction(connection, tx2, [feePayer], {
    skipPreflight: true,
    preflightCommitment: "confirmed",
    confirmation: "confirmed",
  });
  console.log(
    `vend echo: https://explorer.solana.com/tx/${txid2}?cluster=devnet`
  );

  // await waitFor(6000);
  data = (await connection.getAccountInfo(vendingMachineBuffer)).data;
  console.log("Vending Buffer after echo:", data, data.toJSON());

  // echo value to authorized buffer using unauthorized signer
  // const badSigner = new Keypair();
  // console.log(`Attempting to write to buffer as ${badSigner.publicKey}`);

  // console.log("Requesting Airdrop of 1 SOL...");
  // await connection.requestAirdrop(badSigner.publicKey, 2e9);
  // console.log("Airdrop received");

  // const echoData2 = Buffer.from(
  //   serialize(
  //     AuthorizedEchoPayload,
  //     new Payload({
  //       id: InstructionVariant.AuthorizedEcho,
  //       data: "4321",
  //     })
  //   )
  // );

  // let echoIx2 = new TransactionInstruction({
  //   keys: [
  //     {
  //       pubkey: authorizedBuffer,
  //       isSigner: false,
  //       isWritable: true,
  //     },
  //     {
  //       pubkey: feePayer.publicKey,
  //       isSigner: false,
  //       isWritable: false,
  //     },
  //   ],
  //   programId: programId,
  //   data: echoData2,
  // });

  // let tx3 = new Transaction();
  // tx3.add(echoIx2);

  // let txid3 = await sendAndConfirmTransaction(connection, tx3, [badSigner], {
  //   skipPreflight: true,
  //   preflightCommitment: "confirmed",
  //   confirmation: "confirmed",
  // });
  // console.log(`https://explorer.solana.com/tx/${txid3}?cluster=devnet`);
};

main()
  .then(() => {
    console.log("Success");
  })
  .catch((e) => {
    console.error(e);
  });
