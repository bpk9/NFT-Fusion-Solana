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

const COLLECTION_NAME: string = 'NFT Fusion Solana';
const COLLECTION_SYMBOL: string = 'NFS';
const CREATOR_ADDRESS: string = 'LFujUyg8wPiwqt2DFGdSe6wApqwNvpf4zdMebdPVMbz';
const MAX_NFT_ID: number = 1000;
const MIN_NFT_ID: number = 1;
const ONE_SOL: number = 1000000000;
const IPFS_CID: string = 'CID'; // Mock IPFS CID

describe('nft-fusion-solana', () => {
    // Configure the client to use the local cluster.
    anchor.setProvider(anchor.AnchorProvider.env());

    const payer: Signer = Keypair.generate();
    const provider: anchor.AnchorProvider = anchor.AnchorProvider.env();
    const program = anchor.workspace
        .NftFusionSolana as Program<NftFusionSolana>;

    let authority: PublicKey;
    let collectionMasterEdition: PublicKey;
    let collectionMetadata: PublicKey;
    let collectionMint: PublicKey;

    beforeAll(async () => {
        // Fund the payer account
        const airdropTx = await provider.connection.requestAirdrop(
            payer.publicKey,
            ONE_SOL
        );
        await provider.connection.confirmTransaction(airdropTx);

        // Derive the authority account
        [authority] = await anchor.web3.PublicKey.findProgramAddress(
            [payer.publicKey.toBuffer(), Buffer.from('nfs-authority')],
            program.programId
        );

        // Derive the collection mint account
        [collectionMint] = await anchor.web3.PublicKey.findProgramAddress(
            [Buffer.from('nfs-collection-mint')],
            program.programId
        );

        // Derive the collection master edition account
        const mplTokenMetadataProgram = new anchor.web3.PublicKey(
            MPL_TOKEN_METADATA_PROGRAM_ID
        );
        [collectionMasterEdition] =
            await anchor.web3.PublicKey.findProgramAddress(
                [
                    Buffer.from('metadata'),
                    mplTokenMetadataProgram.toBuffer(),
                    collectionMint.toBuffer(),
                    Buffer.from('edition')
                ],
                mplTokenMetadataProgram
            );

        // Derive the collection metadata account
        [collectionMetadata] = await anchor.web3.PublicKey.findProgramAddress(
            [
                Buffer.from('metadata'),
                mplTokenMetadataProgram.toBuffer(),
                collectionMint.toBuffer()
            ],
            mplTokenMetadataProgram
        );
    });

    it('Initializes the collection', async () => {
        // Get the address of the token account that will hold the minted NFT collection
        const tokenAccount = await getAssociatedTokenAddress(
            collectionMint,
            payer.publicKey
        );

        // Mint the NFT collection
        const mintTx = await program.methods
            .initialize(IPFS_CID)
            .accounts({
                authority: authority,
                collectionMasterEdition: collectionMasterEdition,
                collectionMetadata: collectionMetadata,
                collectionMint: collectionMint,
                metadataProgram: MPL_TOKEN_METADATA_PROGRAM_ID,
                signer: payer.publicKey,
                tokenAccount: tokenAccount
            })
            .signers([payer])
            .rpc();

        console.log('Mint NFT collection transaction signature', mintTx);

        // Fetch the on-chain metadata
        const { data: metadataAccountData } =
            await provider.connection.getAccountInfo(collectionMetadata);
        const onChainMetadata =
            getMetadataAccountDataSerializer().deserialize(
                metadataAccountData
            )[0];

        // Assert that the on-chain metadata matches the expected metadata
        expect(onChainMetadata.name).toBe(COLLECTION_NAME);
        expect(onChainMetadata.symbol).toBe(COLLECTION_SYMBOL);
        expect(onChainMetadata.uri).toBe(
            `https://mygateway.mypinata.cloud/ipfs/${IPFS_CID}`
        );
    });

    const mintNft = async (nft1: number, nft2: number) => {
        // Derive the mint account
        const [mint]: [PublicKey, number] =
            anchor.web3.PublicKey.findProgramAddressSync(
                [
                    new anchor.BN(nft1).toBuffer('be', 2),
                    new anchor.BN(nft2).toBuffer('be', 2)
                ],
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
            .mintNft(IPFS_CID, nft1, nft2)
            .accounts({
                authority: authority,
                collectionMasterEdition: collectionMasterEdition,
                collectionMetadata: collectionMetadata,
                collectionMint: collectionMint,
                metadata: metadata,
                metadataProgram: MPL_TOKEN_METADATA_PROGRAM_ID,
                mint: mint,
                signer: payer.publicKey,
                tokenAccount: tokenAccount
            })
            .signers([payer])
            .rpc();

        console.log(
            `Mint NFT transaction signature for #${nft1} + #${nft2}`,
            mintTx
        );

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
        expect(onChainMetadata.name).toBe(`#${nft1} + #${nft2}`);
        expect(onChainMetadata.symbol).toBe(COLLECTION_SYMBOL);
        expect(onChainMetadata.uri).toBe(
            `https://mygateway.mypinata.cloud/ipfs/${IPFS_CID}`
        );
        expect(onChainMetadata.sellerFeeBasisPoints).toBe(500); // 5%
        expect(creators).toHaveLength(2);
        expect(creators[0].address.toString()).toBe(authority.toString());
        expect(creators[0].verified).toBe(true);
        expect(creators[0].share).toBe(0);
        expect(creators[1].address.toString()).toBe(CREATOR_ADDRESS);
        expect(creators[1].verified).toBe(false);
        expect(creators[1].share).toBe(100);
        expect(onChainMetadata.isMutable).toBe(false);
        expect(tokenStandard).toBe(1); // BK TODO: Make this a pNFT
        expect(collection.key).toBe(collectionMint.toString());
        expect(collection.verified).toBe(true);
        expect(uses).toBe(undefined);
    };

    it('Mints an NFT', async () => {
        await mintNft(1, 2);
    });

    it('Mints a second NFT', async () => {
        await mintNft(2, 3);
    });

    it('Does not allow duplicate NFTs', async () => {
        let error = false;
        try {
            await mintNft(1, 2);
        } catch {
            error = true;
        }
        expect(error).toBe(true);
    });

    it(`Does not allow NFT IDs over ${MAX_NFT_ID}`, async () => {
        let error = false;
        try {
            await mintNft(1, MAX_NFT_ID + 1);
        } catch {
            error = true;
        }
        expect(error).toBe(true);
    });

    it(`Does not allow NFT IDs below ${MIN_NFT_ID}`, async () => {
        let error = false;
        try {
            await mintNft(MIN_NFT_ID - 1, 2);
        } catch {
            error = true;
        }
        expect(error).toBe(true);
    });

    it('Does not allow NFT IDs below 0', async () => {
        let error = false;
        try {
            await mintNft(-1, 2);
        } catch {
            error = true;
        }
        expect(error).toBe(true);
    });
});
