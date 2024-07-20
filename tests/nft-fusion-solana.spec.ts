import * as anchor from '@coral-xyz/anchor';
import { Program } from '@coral-xyz/anchor';
import {
    Collection,
    Creator,
    MPL_TOKEN_METADATA_PROGRAM_ID,
    TokenStandard,
    Uses,
    getMetadataAccountDataSerializer
} from '@metaplex-foundation/mpl-token-metadata';
import { getAssociatedTokenAddress } from '@solana/spl-token';
import { Keypair, PublicKey, Signer } from '@solana/web3.js';
import { NftFusionSolana } from '../target/types/nft_fusion_solana';

const ONE_SOL: number = 1000000000;
const IPFS_CID: string = 'CID'; // Mock IPFS CID

describe('nft-fusion-solana', () => {
    // Configure the client to use the local cluster.
    anchor.setProvider(anchor.AnchorProvider.env());

    const payer: Signer = Keypair.generate();
    const provider: anchor.AnchorProvider = anchor.AnchorProvider.env();
    const program = anchor.workspace
        .NftFusionSolana as Program<NftFusionSolana>;

    beforeAll(async () => {
        // Fund the payer account
        const airdropTx = await provider.connection.requestAirdrop(
            payer.publicKey,
            ONE_SOL
        );
        await provider.connection.confirmTransaction(airdropTx);
    });

    it('Mints an NFT', async () => {
        // Derive the mint account
        const [mint]: [PublicKey, number] =
            await anchor.web3.PublicKey.findProgramAddress(
                [payer.publicKey.toBuffer(), Buffer.from('nfs-mint')],
                program.programId
            );

        // Derive the mint authority account
        const [authority]: [PublicKey, number] =
            await anchor.web3.PublicKey.findProgramAddress(
                [payer.publicKey.toBuffer(), Buffer.from('nfs-authority')],
                program.programId
            );

        // Derive the metadata account
        const mplTokenMetadataProgram = new anchor.web3.PublicKey(
            MPL_TOKEN_METADATA_PROGRAM_ID
        );
        const [metadata]: [PublicKey, number] =
            await anchor.web3.PublicKey.findProgramAddress(
                [
                    Buffer.from('metadata'),
                    mplTokenMetadataProgram.toBuffer(),
                    mint.toBuffer()
                ],
                mplTokenMetadataProgram
            );

        // Get the address of the token account that will hold the minted NFT
        const tokenAccount = await getAssociatedTokenAddress(
            mint,
            payer.publicKey
        );

        // Mint the NFT
        const mintTx = await program.methods
            .mintNft(IPFS_CID)
            .accounts({
                authority: authority,
                metadata: metadata,
                metadataProgram: MPL_TOKEN_METADATA_PROGRAM_ID,
                mint: mint,
                signer: payer.publicKey,
                tokenAccount: tokenAccount
            })
            .signers([payer])
            .rpc();

        console.log('Mint NFT transaction signature', mintTx);

        // Fetch the token account to verify the minting
        const accountInfo =
            await provider.connection.getTokenAccountBalance(tokenAccount);

        // Assert that the token account has 1 token (the minted NFT)
        expect(accountInfo.value.amount).toBe('1');

        // Fetch the on-chain metadata
        const { data: metadataAccountData } =
            await provider.connection.getAccountInfo(metadata);
        const onChainMetadata =
            getMetadataAccountDataSerializer().deserialize(
                metadataAccountData
            )[0];

        // Parse the on-chain metadata
        const creators = (onChainMetadata.creators as any)
            .value as Array<Creator>;
        const tokenStandard = (onChainMetadata.tokenStandard as any)
            .value as TokenStandard;
        const collection = (onChainMetadata.collection as any)
            .value as Collection;
        const uses = (onChainMetadata.uses as any).value as Uses;

        // Assert that the on-chain metadata matches the expected metadata
        expect(onChainMetadata.name).toBe('#1');
        expect(onChainMetadata.symbol).toBe('NFS');
        expect(onChainMetadata.uri).toBe(
            `https://mygateway.mypinata.cloud/ipfs/${IPFS_CID}`
        );
        expect(onChainMetadata.sellerFeeBasisPoints).toBe(500); // 5%
        expect(creators).toHaveLength(2);
        expect(creators[0].address.toString()).toBe(authority.toString());
        expect(creators[0].verified).toBe(true);
        expect(creators[0].share).toBe(0);
        expect(creators[1].address.toString()).toBe(
            'LFujUyg8wPiwqt2DFGdSe6wApqwNvpf4zdMebdPVMbz'
        );
        expect(creators[1].verified).toBe(false);
        expect(creators[1].share).toBe(100);
        expect(onChainMetadata.isMutable).toBe(false);
        expect(tokenStandard).toBe(1); // BK TODO: Make this a pNFT
        expect(collection).toBe(undefined);
        expect(uses).toBe(undefined);
    });
});
