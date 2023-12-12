import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import {
  sendAndConfirmTransaction,
  Connection,
  Keypair,
  SystemProgram,
  Transaction,
  PublicKey,
} from "@solana/web3.js";
import {
  createAssociatedTokenAccount,
  mintTo,
  ExtensionType,
  createInitializeMintInstruction,
  getMintLen,
  TOKEN_2022_PROGRAM_ID,
  createInitializeTransferFeeConfigInstruction,
} from "@solana/spl-token";
import {
  AttestationQueueAccount,
  FunctionAccount,
  SwitchboardProgram,
} from "@switchboard-xyz/solana.js";
import { IPubkeys } from "./pdas";
import { L2 } from "../target/types/l2";

export const createTransferFeeMint = async (
  connection: Connection,
  payer: Keypair,
  feeBasisPoints: number,
  decimals: number,
  maxFee: bigint,
  mintAuthority: PublicKey,
  transferFeeConfigAuthority: PublicKey,
  withdrawWithheldAuthority: PublicKey
): Promise<PublicKey> => {
  const mintKeypair = Keypair.generate();
  const mint = mintKeypair.publicKey;

  const extensions = [ExtensionType.TransferFeeConfig];

  const mintLen = getMintLen(extensions);

  const mintLamports = await connection.getMinimumBalanceForRentExemption(
    mintLen
  );
  const mintTransaction = new Transaction().add(
    SystemProgram.createAccount({
      fromPubkey: payer.publicKey,
      newAccountPubkey: mint,
      space: mintLen,
      lamports: mintLamports,
      programId: TOKEN_2022_PROGRAM_ID,
    }),
    createInitializeTransferFeeConfigInstruction(
      mint,
      transferFeeConfigAuthority,
      withdrawWithheldAuthority,
      feeBasisPoints,
      maxFee,
      TOKEN_2022_PROGRAM_ID
    ),
    createInitializeMintInstruction(
      mint,
      decimals,
      mintAuthority,
      null,
      TOKEN_2022_PROGRAM_ID
    )
  );
  await sendAndConfirmTransaction(
    connection,
    mintTransaction,
    [payer, mintKeypair],
    undefined
  );
  return mint;
};

// export async function loadSwitchboard(
//   provider: anchor.AnchorProvider,
//   MRENCLAVE: RawBuffer,
//   recentSlot?: number
// ): Promise<[BootstrappedAttestationQueue, FunctionAccount]> {
//   const switchboardProgram = await SwitchboardProgram.fromProvider(provider);
  // const switchboard = await AttestationQueueAccount.bootstrapNewQueue(
//     switchboardProgram
//   );

//   const [switchboardFunction] =
//     await switchboard.attestationQueue.account.createFunction({
//       name: "random_transfer_fee",
//       containerRegistry: "dockerhub",
//       container: "energeticlee/example",
//       version: "latest",
//       mrEnclave: parseRawBuffer(MRENCLAVE, 32),
//       recentSlot,
//     });

//   return [switchboard, switchboardFunction];
// }
export async function loadSwitchboard(
  provider: anchor.AnchorProvider
): Promise<[AttestationQueueAccount, FunctionAccount]> {
  const switchboardProgram = await SwitchboardProgram.fromProvider(provider);

  const switchboard = await AttestationQueueAccount.load(
    switchboardProgram,
    "CkvizjVnm2zA5Wuwan34NhVT3zFc7vqUyGnA6tuEF5aE"
  );

  const [functionAccount] = await FunctionAccount.load(
    switchboardProgram,
    "4JwNWzqoYVULs1ohHW72mFrCFz1vKmJkTuP72ZVUP2eX"
  );

  return [switchboard[0], functionAccount];
}

export const signAndSendTx = async (
  connection: anchor.web3.Connection,
  tx: anchor.web3.Transaction,
  payer: anchor.web3.Keypair
) => {
  tx.recentBlockhash = (
    await connection.getLatestBlockhash("singleGossip")
  ).blockhash;
  tx.feePayer = payer.publicKey;
  tx.sign(payer);
  const rawTransaction = tx.serialize();
  const txSig = await connection.sendRawTransaction(rawTransaction);

  const latestBlockHash = await connection.getLatestBlockhash();

  await connection.confirmTransaction({
    blockhash: latestBlockHash.blockhash,
    lastValidBlockHeight: latestBlockHash.lastValidBlockHeight,
    signature: txSig,
  });

  return txSig;
};

export const envSetup = async (program: Program<L2>, pubkeys: IPubkeys) => {
  try {
    // AIRDROP SOL
    console.log("TEST 1");
    const tx = new anchor.web3.Transaction();
    // await program.provider.connection.confirmTransaction(
    //   await program.provider.connection.requestAirdrop(
    //     pubkeys.globalOwner.publicKey,
    //     1e9
    //   )
    // );

    console.log("TEST 2");
    const arr = Object.values(pubkeys).slice(1);

    for (let i = 1; i < arr.length; i++) {
      let ix = anchor.web3.SystemProgram.transfer({
        fromPubkey: arr[0].publicKey,
        toPubkey: arr[i].publicKey,
        lamports: 0.1 * anchor.web3.LAMPORTS_PER_SOL,
      });
      tx.add(ix);
    }

    console.log("TEST 3");
    await signAndSendTx(program.provider.connection, tx, pubkeys.globalOwner);

    console.log("TEST 4");
    // TODO: CREATE TOKEN22 MINT, set GLOBAL to AUTHORITY
    const TRANSFER_FEE = 100_00; // 100%
    const mint = await createTransferFeeMint(
      program.provider.connection,
      pubkeys.user1,
      TRANSFER_FEE,
      9,
      BigInt(0),
      pubkeys.user1.publicKey, // MINT AUTHORITY
      pubkeys.globalPda,
      pubkeys.globalPda
    );

    console.log("TEST 5");
    // MINT 10,000 TO ALL pubkeys
    for (let i = 0; i < arr.length; i++) {
      const ata = await createAssociatedTokenAccount(
        program.provider.connection,
        pubkeys.globalOwner,
        mint,
        arr[i].publicKey,
        undefined,
        TOKEN_2022_PROGRAM_ID
      );

      console.log("TEST 6");
      // MINT 10,000
      await mintTo(
        program.provider.connection,
        pubkeys.globalOwner,
        mint,
        ata,
        pubkeys.user1,
        10_000_00,
        undefined,
        undefined,
        TOKEN_2022_PROGRAM_ID
      );
    }

    console.log("TEST 7");
    // RETURN MINT
    return mint;
  } catch (error) {
    console.log("error: ", error);
    process.exit(1);
  }
};
