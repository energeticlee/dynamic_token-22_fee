import * as anchor from "@coral-xyz/anchor";
import { PublicKey } from "@solana/web3.js";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  NATIVE_MINT,
  TOKEN_2022_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
  createAssociatedTokenAccount,
  createMint,
  mintTo,
} from "@solana/spl-token";
import { IPubkeys, ISwitchboardObj, getGlobalPda } from "./pdas";
import { assert } from "chai";
import { L2 } from "../target/types/l2";

export const setupTestEnv = async (
  program: anchor.Program<L2>,
  mint: PublicKey,
  pubkeys: IPubkeys,
  sbObj: ISwitchboardObj
) => {
  // Create global
  const HOUR_TO_NEXT_UPDATE = 48;
  try {
    await program.methods
      .initGlobal(HOUR_TO_NEXT_UPDATE)
      .accounts({
        global: pubkeys.globalPda,
        mint: mint, // ADD TOKEN22 MINT
        payer: pubkeys.globalOwner.publicKey,
        switchboard: sbObj.switchboard.program.attestationProgramId,
        switchboardState:
          sbObj.switchboard.program.attestationProgramState.publicKey,
        switchboardAttestationQueue:
          sbObj.switchboard.attestationQueue.publicKey,
        switchboardFunction: sbObj.switchboardFunction.publicKey,
        switchboardRequest: sbObj.switchboardRequestKeypair.publicKey,
        switchboardRequestEscrow: anchor.utils.token.associatedAddress({
          mint: sbObj.switchboard.program.mint.address,
          owner: sbObj.switchboardRequestKeypair.publicKey,
        }),
        switchboardMint: NATIVE_MINT,
        tokenProgram: TOKEN_PROGRAM_ID,
        tokenProgram22: TOKEN_2022_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      })
      .signers([pubkeys.globalOwner, sbObj.switchboardRequestKeypair])
      .rpc();
  } catch (error) {
    console.log("ERROR", error);
  }
  const globalPda = getGlobalPda(program);
  const globalData = await program.account.global.fetch(globalPda);

  assert.equal(+globalData.hourToNextUpdate, HOUR_TO_NEXT_UPDATE);
  assert.equal(+globalData.currentTransferFeeBp, 100_00);
  assert.equal(globalData.mint.toString(), mint.toString());

  assert.equal(
    globalData.switchboardFunction.toString(),
    sbObj.switchboardFunction.publicKey.toString()
  );
  assert.equal(
    globalData.attestationProgramState.toString(),
    sbObj.switchboard.program.attestationProgramState.publicKey.toString()
  );
  assert.equal(
    globalData.attestationQueue.toString(),
    sbObj.switchboard.attestationQueue.publicKey.toString()
  );
  assert.equal(globalData.switchboardRequest, null);
};

export const triggerUpdate = async (
  program: anchor.Program<L2>,
  mint: PublicKey,
  pubkeys: IPubkeys
) => {
  // Create global
  await program.methods
    .triggerUpdate(123)
    .accounts({
      global: pubkeys.globalPda,
      mint: mint, // ADD TOKEN22 MINT
      enclaveSigner: pubkeys.globalOwner.publicKey,
      switchboardMint: NATIVE_MINT,
      tokenProgram: TOKEN_PROGRAM_ID,
      tokenProgram22: TOKEN_2022_PROGRAM_ID,
      associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
    })
    .signers([pubkeys.globalOwner])
    .rpc();

  const globalPda = getGlobalPda(program);
  const globalData = await program.account.global.fetch(globalPda);

  assert.equal(+globalData.currentTransferFeeBp, 0);
  assert.equal(+globalData.hourToNextUpdate, 0);
  assert.equal(globalData.mint.toString(), "0.toString()");
  assert.equal(+globalData.nextUpdateSlot, 0);
};
