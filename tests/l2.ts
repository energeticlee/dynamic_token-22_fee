import * as anchor from "@coral-xyz/anchor";
import { getGlobalPda } from "./pdas";
import { setupTestEnv } from "./script";
import {
  AttestationQueueAccount,
  BootstrappedAttestationQueue,
  FunctionAccount,
  FunctionRequestAccount,
  attestationTypes,
} from "@switchboard-xyz/solana.js";
import { parseRawMrEnclave } from "@switchboard-xyz/common";
import { envSetup, loadSwitchboard } from "./utils";
import fs from "fs";
import { L2 } from "../target/types/l2";
import { SECRET_KEY } from "./delete";

describe("l2", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  let program = new anchor.Program(
    JSON.parse(
      fs
        .readFileSync(
          "./target/idl/l2.json", // your idl
          "utf8"
        )
        .toString()
    ),
    new anchor.web3.PublicKey("auULn3TunUFz5mvM1VSLUT184oAApgnEsLmqZrVyUAP"),
    provider
  ) as anchor.Program<L2>;

  const TRANSFER_FEE = 100_00; // 100%
  const globalOwner = anchor.web3.Keypair.generate();
  const user1 = anchor.web3.Keypair.generate();
  const user2 = anchor.web3.Keypair.generate();
  let mint: anchor.web3.PublicKey;

  const globalPda = getGlobalPda(program);
  const pubkeys = {
    globalPda,
    globalOwner,
    user1,
    user2,
  };

  const MRENCLAVE = parseRawMrEnclave(
    "0xe89b9949da4248f80264f6d0da95dcf14a203c4b44c7670719298e2cadec5ca2",
    true
  );

  let switchboard: AttestationQueueAccount;
  let switchboardFunction: FunctionAccount;

  const switchboardRequestKeypair = anchor.web3.Keypair.generate();
  const switchboardObj = {
    switchboard,
    switchboardFunction,
    switchboardRequestKeypair,
  };
  it("Is initialized!", async () => {
    // [switchboard, switchboardFunction] = await loadSwitchboard(
    //   provider,
    //   MRENCLAVE,
    //   await provider.connection.getSlot()
    // );
    [switchboard, switchboardFunction] = await loadSwitchboard(provider);

    switchboardObj.switchboard = switchboard;
    switchboardObj.switchboardFunction = switchboardFunction;

    // INIT GLOBAL STATE
    mint = await envSetup(program, pubkeys);

    // CREATE GLOBAL
    // TODO: FINISH TEST
    await setupTestEnv(program, mint, pubkeys, switchboardObj);
  });

  const delay = async (ms: number): Promise<void> => {
    return new Promise((resolve) => {
      setTimeout(resolve, ms);
    });
  };

  it("Wait and check for trigger!", async () => {
    const slotBefore = await program.provider.connection.getSlot();
    const globalPda = getGlobalPda(program);
    const globalDataPre = await program.account.global.fetch(globalPda);
    console.log("WAIT", slotBefore);
    console.log("hourToNextUpdate", globalDataPre.currentTransferFeeBp);
    console.log("hourToNextUpdate", globalDataPre.hourToNextUpdate);
    console.log("UPDATE IN", +globalDataPre.nextUpdateSlot);
    await delay(10_000);
    const slotAfter = await program.provider.connection.getSlot();
    console.log("CLEAR", slotAfter);
    const globalData = await program.account.global.fetch(globalPda);
    console.log("globalData", globalData.currentTransferFeeBp);
    console.log("globalData", globalData.hourToNextUpdate);
    console.log("globalData", +globalData.nextUpdateSlot);

    // // First, generate a new keypair to sign our instruction
    // // Normally this happens within the enclave
    // const enclaveSigner = anchor.web3.Keypair.generate();

    // // Load the Switchboard account states
    // const [_sbRequestAccount, sbRequestState] =
    //   await FunctionRequestAccount.load(
    //     switchboard.program,
    //     switchboardRequestKeypair.publicKey
    //   );
    // const sbFunctionState = await switchboardFunction.loadData();

    // // We need a wrapped SOL TokenAccount to receive the oracle reward from the fn request escrow
    // const rewardAddress =
    //   await switchboard.program.mint.getOrCreateAssociatedUser(user1.publicKey);

    // // Next, generate the function_request_verify ixn that we must call before running
    // // any of our emitted instructions.
    // const fnRequestVerifyIxn = attestationTypes.functionRequestVerify(
    //   switchboard.program,
    //   {
    //     params: {
    //       observedTime: new anchor.BN(Math.floor(Date.now() / 1000)),
    //       errorCode: 0,
    //       mrEnclave: Array.from(MRENCLAVE),
    //       requestSlot: sbRequestState.activeRequest.requestSlot,
    //       containerParamsHash: sbRequestState.containerParamsHash,
    //     },
    //   },
    //   {
    //     request: switchboardRequestKeypair.publicKey,
    //     functionEnclaveSigner: enclaveSigner.publicKey,
    //     escrow: sbRequestState.escrow,
    //     function: switchboardFunction.publicKey,
    //     functionEscrow: sbFunctionState.escrowTokenWallet,
    //     verifierQuote: switchboard.verifier.publicKey,
    //     verifierEnclaveSigner: switchboard.verifier.signer.publicKey,
    //     verifierPermission: switchboard.verifier.permissionAccount.publicKey,
    //     state: switchboard.program.attestationProgramState.publicKey,
    //     attestationQueue: switchboard.attestationQueue.publicKey,
    //     receiver: rewardAddress,
    //     tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
    //   }
    // );

    // const tx = await program.methods
    //   .settle(1)
    //   .accounts({
    //     user: userPubkey,
    //     switchboardFunction: switchboardFunction.publicKey,
    //     switchboardRequest: switchboardRequestKeypair.publicKey,
    //     enclaveSigner: enclaveSigner.publicKey,
    //   })
    //   .preInstructions([fnRequestVerifyIxn])
    //   .signers([enclaveSigner, switchboard.verifier.signer])
    //   .rpc();
    // console.log(`[TX] settle: ${tx}`);

    // const userState = await program.account.userState.fetch(userPubkey);
    // if (userState.guess === userState.result) {
    //   console.log(`[RESULT] user won!`);
    // } else {
    //   console.log(`[RESULT] user lost :(`);
    // }
  });
  // TODO: SET TIMER LOW AND TEST BURN
  // TRADE TO TEST BURN
  // TEST CALLBACK UPDATE 1
  // TRADE TO TEST BURN
  // TEST CALLBACK UPDATE 2
  // TRADE TO TEST BURN
  // TEST CLAIM WITHDRAW
});
