import * as anchor from '@coral-xyz/anchor';
import { BN, Program } from '@coral-xyz/anchor';
import { Escrow } from '../target/types/escrow';
import { LAMPORTS_PER_SOL, Keypair, PublicKey } from '@solana/web3.js';
import {
  TOKEN_2022_PROGRAM_ID,
  type TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAssociatedTokenAddressSync,
  createAssociatedTokenAccount,
  mintToChecked,
  createMint,
} from '@solana/spl-token';
import {
  confirmTransaction,
  createAccountsMintsAndTokenAccounts,
  makeKeypairs,
} from '@solana-developers/helpers';
import { randomBytes } from 'node:crypto';
import { assert } from 'chai';

const programId = new PublicKey('D1WxxPdrGKZym4rBRHz6A18JPqPVRUeHKnvBbj1b7oac');
const TOKEN_PROGRAM: typeof TOKEN_2022_PROGRAM_ID | typeof TOKEN_PROGRAM_ID =
  TOKEN_2022_PROGRAM_ID;

describe('escrow', () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const connection = provider.connection;
  const signer = provider.wallet as anchor.Wallet;

  const program = anchor.workspace.Escrow as Program<Escrow>;

  let alice: anchor.web3.Keypair;
  let bob: anchor.web3.Keypair;

  [alice, bob] = makeKeypairs(4);

  let maker;
  let taker;
  let tokenMintAkey;
  let makerTokenAccountA;
  let takerTokenAccountA;
  let tokenMintBkey;
  let makerTokenAccountB;
  let takerTokenAccountB;

  before(
    'Creates Alice and Bob accounts, 2 token mints, and associated token accounts for both tokens for both users',
    async () => {
      //airdrop some SOL to both alice and bob
      await provider.connection.requestAirdrop(
        alice.publicKey,
        2 * LAMPORTS_PER_SOL
      );
      await provider.connection.requestAirdrop(
        bob.publicKey,
        2 * LAMPORTS_PER_SOL
      );

      //create token mints
      let mintPubkeyA = await createMint(
        connection, // connection
        signer.payer, // fee payer
        alice.publicKey, // mint authority
        alice.publicKey, // freeze authority (you can use `null` to disable it. when you disable it, you can't turn it on again)
        6 // decimals
      );
      console.log(`mint: ${mintPubkeyA.toBase58()}`);

      let mintPubkeyB = await createMint(
        connection, // connection
        signer.payer, // fee payer
        alice.publicKey, // mint authority
        alice.publicKey, // freeze authority (you can use `null` to disable it. when you disable it, you can't turn it on again)
        6 // decimals
      );

      // create associated token accounts for both alice and bob
      let makerATAA = await createAssociatedTokenAccount(
        connection, // connection
        signer.payer, // fee payer
        mintPubkeyA, // mint
        alice.publicKey // owner,
      );
      console.log(`maker ATAA: ${makerATAA.toBase58()}`);

      let makerATAB = await createAssociatedTokenAccount(
        connection, // connection
        signer.payer, // fee payer
        mintPubkeyB, // mint
        alice.publicKey // owner,
      );
      console.log(`maker ATAB: ${makerATAB.toBase58()}`);

      let takerATAA = await createAssociatedTokenAccount(
        connection, // connection
        signer.payer, // fee payer
        mintPubkeyA, // mint
        bob.publicKey // owner,
      );
      console.log(`taker ATAA: ${takerATAA.toBase58()}`);

      let takerATAB = await createAssociatedTokenAccount(
        connection, // connection
        signer.payer, // fee payer
        mintPubkeyB, // mint
        bob.publicKey // owner,
      );
      console.log(`taker ATAB: ${takerATAB.toBase58()}`);

      // mint tokens to both alice and bob
      let txhash = await mintToChecked(
        connection, // connection
        signer.payer, // fee payer
        mintPubkeyA, // mint
        makerATAA, // receiver (should be a token account)
        alice, // mint authority
        10000 * 10 ** 6, // amount. if your decimals is 8, you mint 10^8 for 1 token.
        6 // decimals
      );
      console.log(`txhash: ${txhash}`);

      let txhash2 = await mintToChecked(
        connection, // connection
        signer.payer, // fee payer
        mintPubkeyB, // mint
        takerATAB, // receiver (should be a token account)
        alice, // mint authority
        10000 * 10 ** 6, // amount. if your decimals is 8, you mint 10^8 for 1 token.
        6 // decimals
      );
      console.log(`txhash: ${txhash2}`);

      //set variables to be used in the tests
      maker = alice;
      taker = bob;
      tokenMintAkey = mintPubkeyA;
      tokenMintBkey = mintPubkeyB;
      makerTokenAccountA = makerATAA;
      makerTokenAccountB = makerATAB;
      takerTokenAccountA = takerATAA;
      takerTokenAccountB = takerATAB;
    }
  );

  it('Alice makes an offer for token B and deposits token A', async () => {
    // Pick a random seed for the offer we'll make
    const seed = new BN(randomBytes(8));

    let amount = new BN(1_000_000);
    let deposit = new BN(500_000);

    // Then determine the account addresses we'll use for the escrow and the vault
    const escrow = PublicKey.findProgramAddressSync(
      [
        Buffer.from('escrow'),
        maker.publicKey.toBuffer(),
        seed.toArrayLike(Buffer, 'le', 8),
      ],
      program.programId
    )[0];

    const vault = PublicKey.findProgramAddressSync(
      [
        Buffer.from('vault'),
        escrow.toBuffer(),
        seed.toArrayLike(Buffer, 'le', 8),
      ],
      program.programId
    )[0];

    try {
      const makeIx = await program.methods
        .make(seed, amount, deposit)
        .accounts({
          maker: maker.publicKey,
          tokenMintA: tokenMintAkey,
          tokenMintB: tokenMintBkey,
          tokenProgram: TOKEN_PROGRAM,
        })
        .instruction();

      const blockhashContext = await connection.getLatestBlockhash();

      const tx = new anchor.web3.Transaction({
        feePayer: alice.publicKey,
        blockhash: blockhashContext.blockhash,
        lastValidBlockHeight: blockhashContext.lastValidBlockHeight,
      }).add(makeIx);

      const signature = await anchor.web3.sendAndConfirmTransaction(
        connection,
        tx,
        [alice]
      );

      console.log(`Signature: ${signature}`);

      /*

      //different way to send the transaction


      const transactionSignature = await program.methods
        .make(seed, amount, deposit)
        .accounts({
          maker: maker.publicKey,
          tokenMintA: tokenMintAkey,
          tokenMintB: tokenMintBkey,
          makerTokenAccountA: makerTokenAccountA.publicKey, //Error with the assoicated token account
          escrow,
          vault,
          tokenProgram: TOKEN_PROGRAM,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([maker])
        .rpc();

      await confirmTransaction(connection, transactionSignature);
      */

      // Check our vault contains the tokens offered
      const vaultBalanceResponse = await connection.getTokenAccountBalance(
        vault
      );
      const vaultBalance = new BN(vaultBalanceResponse.value.amount);
      assert(vaultBalance.eq(deposit));

      // Check our Offer account contains the correct data
      const offerAccount = await program.account.escrow.fetch(escrow);

      assert(offerAccount.maker.equals(alice.publicKey));
      assert(offerAccount.tokenMintA.equals(tokenMintAkey));
      assert(offerAccount.tokenMintB.equals(tokenMintBkey));
      assert(offerAccount.receiveAmount.eq(amount));
    } catch (error) {
      console.error('Transaction Error: ', error);
      assert(false);
    }
  });
});
