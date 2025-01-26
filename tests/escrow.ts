import * as anchor from '@coral-xyz/anchor';
import { BN, Program } from '@coral-xyz/anchor';
import { Escrow } from '../target/types/escrow';
import { LAMPORTS_PER_SOL, Keypair, PublicKey } from '@solana/web3.js';
import {
  TOKEN_2022_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAssociatedTokenAddressSync,
  createAssociatedTokenAccount,
  mintToChecked,
  createMint,
  TOKEN_PROGRAM_ID,
  getOrCreateAssociatedTokenAccount,
  mintTo,
} from '@solana/spl-token';
import {
  confirmTransaction,
  createAccountsMintsAndTokenAccounts,
  makeKeypairs,
} from '@solana-developers/helpers';
import { randomBytes } from 'node:crypto';
import { assert } from 'chai';
import { SYSTEM_PROGRAM_ID } from '@coral-xyz/anchor/dist/cjs/native/system';

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

  // Pick a random seed for the offer we'll make
  const seed = new BN(randomBytes(8));

  let amount = new BN(1_000_000);
  let deposit = new BN(500_000);

  let alice = anchor.web3.Keypair.generate();
  let bob = anchor.web3.Keypair.generate();

  console.log('Alice Key', alice.publicKey.toBase58());
  console.log('Bob Key', bob.publicKey.toBase58());

  let maker;
  let taker;
  let tokenMintAkey;
  let makerTokenAccountA;
  let tokenMintBkey;
  let takerTokenAccountB;

  let // Then determine the account addresses we'll use for the escrow and the vault
    escrow = PublicKey.findProgramAddressSync(
      [
        Buffer.from('escrow'),
        alice.publicKey.toBuffer(),
        seed.toArrayLike(Buffer, 'le', 8),
      ],
      programId
    )[0];

  let vault;

  before(
    'Creates Alice and Bob accounts, 2 token mints, and associated token accounts for both tokens for both users',
    async () => {
      //airdrop some SOL to both alice and bob
      let tx1 = await provider.connection.requestAirdrop(
        alice.publicKey,
        2 * LAMPORTS_PER_SOL
      );

      await confirmTransaction(connection, tx1, 'confirmed');

      let tx2 = await provider.connection.requestAirdrop(
        bob.publicKey,
        2 * LAMPORTS_PER_SOL
      );

      await confirmTransaction(connection, tx2, 'confirmed');

      //create token mints
      let mintPubkeyA = await createMint(
        connection, // connection
        alice, // fee payer
        alice.publicKey, // mint authority
        null, // freeze authority (you can use `null` to disable it. when you disable it, you can't turn it on again)
        6 // decimals
      );
      console.log(`mint A: ${mintPubkeyA}`);

      let mintPubkeyB = await createMint(
        connection, // connection
        alice, // fee payer
        alice.publicKey, // mint authority
        null, // freeze authority (you can use `null` to disable it. when you disable it, you can't turn it on again)
        6 // decimals
      );
      console.log(`mint B: ${mintPubkeyB}`);

      // create associated token accounts for both alice and bob
      let makerATAA = await getOrCreateAssociatedTokenAccount(
        connection, // connection
        alice, // fee payer
        mintPubkeyA, // mint
        alice.publicKey // owner,
      );
      console.log(`maker ATAA: ${makerATAA.address}`);

      let takerATAB = await getOrCreateAssociatedTokenAccount(
        connection, // connection
        alice, // fee payer
        mintPubkeyB, // mint
        bob.publicKey // owner,
      );
      console.log(`taker ATAB: ${takerATAB.address}`);

      // mint tokens to both alice and bob
      await mintTo(
        connection, // connection
        alice, // fee payer
        mintPubkeyA, // mint
        makerATAA.address, // receiver (should be a token account)
        alice, // mint authority
        10000 * 10 ** 6 // amount. if your decimals is 8, you mint 10^8 for 1 token.
      );
      console.log(`Maker ATA: `, makerATAA.amount.valueOf());

      await mintTo(
        connection, // connection
        bob, // fee payer
        mintPubkeyB, // mint
        takerATAB.address, // receiver (should be a token account)
        alice, // mint authority
        10000 * 10 ** 6 // amount. if your decimals is 8, you mint 10^8 for 1 token.
      );
      console.log(`taker ATA: `, takerATAB.amount.valueOf());

      //set variables to be used in the tests
      maker = alice;
      taker = bob;
      tokenMintAkey = mintPubkeyA;
      tokenMintBkey = mintPubkeyB;
      makerTokenAccountA = makerATAA.address;
      takerTokenAccountB = takerATAB.address;
    }
  );

  it('Alice makes an offer for token B and deposits token A', async () => {
    vault = getAssociatedTokenAddressSync(
      tokenMintAkey,
      escrow,
      true,
      TOKEN_PROGRAM_ID
    );

    try {
      /*
      const makeIx = await program.methods
        .make(seed, amount, deposit)
        .accounts({
          maker,
          tokenMintA: tokenMintAkey,
          tokenMintB: tokenMintBkey,
          tokenProgram: TOKEN_PROGRAM_ID,
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
        [maker]
      );

      console.log(`Signature: ${signature}`);
  */

      //different way to send the transaction

      let accounts = {
        maker: maker.publicKey,
        tokenMintA: tokenMintAkey,
        tokenMintB: tokenMintBkey,
        makerTokenAccountA, //Error with the assoicated token account
        escrow,
        vault,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      };

      const transactionSignature = await program.methods
        .make(seed, amount, deposit)
        .accounts({
          maker: maker.publicKey,
          tokenMintA: tokenMintAkey,
          tokenMintB: tokenMintBkey,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([maker])
        .rpc();

      await confirmTransaction(connection, transactionSignature);

      // Check our vault contains the tokens offered
      const vaultBalanceResponse = await connection.getTokenAccountBalance(
        vault
      );
      const vaultBalance = new BN(vaultBalanceResponse.value.amount);
      assert(vaultBalance.eq(deposit));

      // Check our Offer account contains the correct data
      const offerAccount = await program.account.escrow.fetch(escrow);

      assert(offerAccount.maker.equals(maker.publicKey));
      assert(offerAccount.tokenMintA.equals(tokenMintAkey));
      assert(offerAccount.tokenMintB.equals(tokenMintBkey));
      assert(offerAccount.receiveAmount.eq(amount));
    } catch (error) {
      console.error('Transaction Error: ', error);
      assert(false);
    }
  });
});
