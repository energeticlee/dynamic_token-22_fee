import * as anchor from "@coral-xyz/anchor";
import { getGlobalPda } from "./pdas";
import { setupTestEnv } from "./script";
import {
  BootstrappedAttestationQueue,
  FunctionAccount,
} from "@switchboard-xyz/solana.js";
import { parseRawMrEnclave } from "@switchboard-xyz/common";
import { createTransferFeeMint, envSetup, loadSwitchboard } from "./utils";
import fs from "fs";
import { L2 } from "../target/types/l2";

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
    new anchor.web3.PublicKey("F9drLzcivmLJNaSMtiVtmc2aewgVjkbjg5DREqjndS7A"),
    provider
  ) as anchor.Program<L2>;

  const TRANSFER_FEE = 100_00; // 100%

  const globalOwner = anchor.web3.Keypair.generate();
  const user1 = anchor.web3.Keypair.generate();
  const user2 = anchor.web3.Keypair.generate();
  const user3 = anchor.web3.Keypair.generate();
  const bozo = anchor.web3.Keypair.generate();
  let mint: anchor.web3.PublicKey;

  const globalPda = getGlobalPda(program);
  const pubkeys = {
    globalPda,
    globalOwner,
    user1,
    user2,
  };

  const MRENCLAVE = parseRawMrEnclave(
    "3140b0ad38b60a5229fa4ca553fafc5ba1307335638f3adb58afac7657280362",
    true
  );

  let switchboard: BootstrappedAttestationQueue;
  let switchboardFunction: FunctionAccount;
  const switchboardRequestKeypair = anchor.web3.Keypair.generate();
  const switchboardObj = {
    switchboard,
    switchboardFunction,
    switchboardRequestKeypair,
  };
  it("Is initialized!", async () => {
    [switchboard, switchboardFunction] = await loadSwitchboard(
      program.provider as anchor.AnchorProvider,
      MRENCLAVE,
      await provider.connection.getSlot()
    );
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
    console.log("UPDATE IN", +globalDataPre.nextUpdateSlot);
    console.log("WAIT", slotBefore);
    await delay(10_000);
    const slotAfter = await program.provider.connection.getSlot();
    console.log("CLEAR", slotAfter);
    const globalData = await program.account.global.fetch(globalPda);
    console.log("globalData", globalData.currentTransferFeeBp);
  });
  // TODO: SET TIMER LOW AND TEST BURN
  // TRADE TO TEST BURN
  // TEST CALLBACK UPDATE 1
  // TRADE TO TEST BURN
  // TEST CALLBACK UPDATE 2
  // TRADE TO TEST BURN
  // TEST CLAIM WITHDRAW
});
