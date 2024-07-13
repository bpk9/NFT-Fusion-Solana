import * as anchor from '@coral-xyz/anchor';
import { Program } from '@coral-xyz/anchor';
import {
    createMint,
    getOrCreateAssociatedTokenAccount
} from '@solana/spl-token';
import { Keypair, PublicKey, Signer } from '@solana/web3.js';
import { NftFusionSolana } from '../target/types/nft_fusion_solana';

const ONE_SOL: number = 1000000000;

describe('nft-fusion-solana', () => {
    // Configure the client to use the local cluster.
    anchor.setProvider(anchor.AnchorProvider.env());

    const program = anchor.workspace
        .NftFusionSolana as Program<NftFusionSolana>;

    it('Mints an NFT', async () => {
        const payer: Signer = Keypair.generate();
        const provider: anchor.AnchorProvider = anchor.AnchorProvider.env();

        // Fund the payer account
        const airdropTx = await provider.connection.requestAirdrop(
            payer.publicKey,
            ONE_SOL
        );
        await provider.connection.confirmTransaction(airdropTx);

        // Create a new mint account
        const mint: PublicKey = await createMint(
            provider.connection,
            payer,
            provider.wallet.publicKey, // mint authority
            null, // freeze authority
            0 // decimals
        );

        // Create a token account to hold the minted NFT
        const tokenAccount = await getOrCreateAssociatedTokenAccount(
            provider.connection,
            payer,
            mint,
            provider.wallet.publicKey
        );

        // Mint the NFT
        const mintTx = await program.methods
            .mint()
            .accounts({
                mint: mint,
                signer: provider.wallet.publicKey,
                tokenAccount: tokenAccount.address
            })
            .signers([])
            .rpc();

        console.log('Mint NFT transaction signature', mintTx);

        // Fetch the token account to verify the minting
        const accountInfo = await provider.connection.getTokenAccountBalance(
            tokenAccount.address
        );
        console.log('Token account info', accountInfo);

        // Assert that the token account has 1 token (the minted NFT)
        expect(accountInfo.value.amount).toBe('1');
    });
});
