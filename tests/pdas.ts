import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { L2 } from "../target/types/l2";
import { PublicKey } from "@solana/web3.js";
import {
  createAssociatedTokenAccount,
  createMint,
  mintTo,
} from "@solana/spl-token";
import {
  BootstrappedAttestationQueue,
  FunctionAccount,
} from "@switchboard-xyz/solana.js";

export interface IPubkeys {
  globalPda: anchor.web3.PublicKey;
  globalOwner: anchor.web3.Keypair;
  user1: anchor.web3.Keypair;
}
export interface ISwitchboardObj {
  switchboard: BootstrappedAttestationQueue;
  switchboardFunction: FunctionAccount;
  switchboardRequestKeypair: anchor.web3.Keypair;
}

export const getGlobalPda = (program: Program<L2>) => {
  const [globalPda, _globalPdaBump] = PublicKey.findProgramAddressSync(
    [Buffer.from("global")],
    program.programId
  );
  return globalPda;
};
