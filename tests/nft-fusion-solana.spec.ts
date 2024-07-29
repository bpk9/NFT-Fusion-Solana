import * as anchor from '@coral-xyz/anchor';
import {
    MPL_TOKEN_METADATA_PROGRAM_ID,
    TokenStandard,
    burnNft,
    createNft,
    getMetadataAccountDataSerializer,
    mplTokenMetadata,
    transferV1,
    verifyCollection
} from '@metaplex-foundation/mpl-token-metadata';
import {
    createSignerFromKeypair,
    generateSigner,
    keypairIdentity,
    percentAmount,
    sol
} from '@metaplex-foundation/umi';
import { createUmi } from '@metaplex-foundation/umi-bundle-defaults';
import { getAssociatedTokenAddress } from '@solana/spl-token';
import { PublicKey } from '@solana/web3.js';
import {
    COLLECTION_NAME,
    COLLECTION_SYMBOL,
    IPFS_CID,
    MAX_NFT_ID,
    MIN_NFT_ID,
    ONE_SOL,
    PARENT_COLLECTION_NAME,
    payer,
    program,
    provider
} from './constants';
import { mintNft } from './helpers';
import { MintNftAccounts, NFTAccounts } from './interfaces';

describe('nft-fusion-solana', () => {
    // Configure the client to use the local cluster.
    anchor.setProvider(anchor.AnchorProvider.env());

    let authority: PublicKey;
    let childCollectionMasterEdition: PublicKey;
    let childCollectionMetadata: PublicKey;
    let childCollectionMint: PublicKey;
    let mintAccounts: MintNftAccounts;
    const parentNfts: Array<NFTAccounts> = [];

    beforeAll(async () => {
        // Fund the payer account
        const airdropTx = await provider.connection.requestAirdrop(
            payer.publicKey,
            ONE_SOL
        );
        await provider.connection.confirmTransaction(airdropTx);

        // Derive the authority account
        [authority] = await anchor.web3.PublicKey.findProgramAddress(
            [payer.publicKey.toBuffer(), Buffer.from('authority')],
            program.programId
        );

        // Derive the child collection mint account
        [childCollectionMint] = await anchor.web3.PublicKey.findProgramAddress(
            [Buffer.from('child-collection-mint')],
            program.programId
        );

        // Derive the child collection master edition account
        const mplTokenMetadataProgram = new anchor.web3.PublicKey(
            MPL_TOKEN_METADATA_PROGRAM_ID
        );
        [childCollectionMasterEdition] =
            await anchor.web3.PublicKey.findProgramAddress(
                [
                    Buffer.from('metadata'),
                    mplTokenMetadataProgram.toBuffer(),
                    childCollectionMint.toBuffer(),
                    Buffer.from('edition')
                ],
                mplTokenMetadataProgram
            );

        // Derive the child collection metadata account
        [childCollectionMetadata] =
            await anchor.web3.PublicKey.findProgramAddress(
                [
                    Buffer.from('metadata'),
                    mplTokenMetadataProgram.toBuffer(),
                    childCollectionMint.toBuffer()
                ],
                mplTokenMetadataProgram
            );

        // Initialize UMI
        console.log('Initializing UMI...');
        const umi = createUmi('http://127.0.0.1:8899');
        const umiKeypair = umi.eddsa.createKeypairFromSecretKey(
            payer.secretKey
        );
        const umiSigner = createSignerFromKeypair(umi, umiKeypair);
        umi.use(keypairIdentity(umiSigner));
        umi.use(mplTokenMetadata());
        await umi.rpc.airdrop(umiSigner.publicKey, sol(1.5));
        console.log('UMI initialized');

        // Derive the Parent Colleciton Mint Address
        const parentCollectionMint = generateSigner(umi);
        const parentCollectionMintAddress = new PublicKey(
            parentCollectionMint.publicKey
        );

        // Derive the Parent Collection Metadata Account
        const [parentCollectionMetadata] =
            await anchor.web3.PublicKey.findProgramAddress(
                [
                    Buffer.from('metadata'),
                    mplTokenMetadataProgram.toBuffer(),
                    parentCollectionMintAddress.toBuffer()
                ],
                mplTokenMetadataProgram
            );

        // Derive the Parent Collection Master Edition Account
        const [parentCollectionMasterEdition] =
            await anchor.web3.PublicKey.findProgramAddress(
                [
                    Buffer.from('metadata'),
                    mplTokenMetadataProgram.toBuffer(),
                    parentCollectionMintAddress.toBuffer(),
                    Buffer.from('edition')
                ],
                mplTokenMetadataProgram
            );

        // Mint the Parent Collection
        console.log('Creating Parent Collection...');
        await createNft(umi, {
            authority: umiSigner,
            masterEdition: parentCollectionMasterEdition as any,
            metadata: parentCollectionMetadata as any,
            mint: parentCollectionMint,
            name: PARENT_COLLECTION_NAME,
            payer: umiSigner,
            sellerFeeBasisPoints: percentAmount(5),
            uri: 'https://example.com/my-nft.json' // Mock URI,
        }).sendAndConfirm(umi);
        console.log('Created Parent Collection');

        // Create the NFTs for the Parent Collection
        console.log('Creating NFTs...');
        for (let i = 0; i < 4; i++) {
            // Create the mint account
            const parentNftMint = generateSigner(umi);
            const parentNftMintAddress = new PublicKey(parentNftMint.publicKey);

            // Derive the master edition account
            const mplTokenMetadataProgram = new anchor.web3.PublicKey(
                MPL_TOKEN_METADATA_PROGRAM_ID
            );
            const [parentNftMasterEdition] =
                await anchor.web3.PublicKey.findProgramAddress(
                    [
                        Buffer.from('metadata'),
                        mplTokenMetadataProgram.toBuffer(),
                        parentNftMintAddress.toBuffer(),
                        Buffer.from('edition')
                    ],
                    mplTokenMetadataProgram
                );

            // Derive the metadata and token addresses
            const parentNftTokenAccount = await getAssociatedTokenAddress(
                parentNftMintAddress,
                payer.publicKey
            );
            const [parentNftMetadata] =
                await anchor.web3.PublicKey.findProgramAddress(
                    [
                        Buffer.from('metadata'),
                        mplTokenMetadataProgram.toBuffer(),
                        parentNftMintAddress.toBuffer()
                    ],
                    mplTokenMetadataProgram
                );

            // Create the NFT
            await createNft(umi, {
                authority: umiSigner,
                collection: {
                    verified: false,
                    key: parentCollectionMint.publicKey
                },
                masterEdition: parentNftMasterEdition as any,
                metadata: parentNftMetadata as any,
                mint: parentNftMint,
                name: `Parent NFT ${i}`,
                payer: umiSigner,
                sellerFeeBasisPoints: percentAmount(5),
                uri: `https://example.com/my-nft-${i + 1}.json` // Mock URI,
            }).sendAndConfirm(umi);
            console.log(`Successfully created NFT #${i + 1}`);

            // Verify the NFT Collection
            await verifyCollection(umi, {
                collection: parentCollectionMetadata as any,
                payer: umiSigner,
                metadata: parentNftMetadata as any,
                collectionAuthority: umiSigner,
                collectionMint: parentCollectionMint.publicKey as any,
                collectionMasterEditionAccount:
                    parentCollectionMasterEdition as any
            }).sendAndConfirm(umi);

            parentNfts.push({
                masterEdition: parentNftMasterEdition as any,
                metadata: parentNftMetadata,
                mint: parentNftMintAddress,
                tokenAccount: parentNftTokenAccount
            });
        }

        // Send NFT #4 to a random address
        const randomAddress = generateSigner(umi);
        await transferV1(umi, {
            // from: parentNfts[3].mint,
            destinationOwner: randomAddress as any,
            mint: parentNfts[3].mint as any,
            tokenStandard: TokenStandard.NonFungible
        }).sendAndConfirm(umi);

        // Initialize the mint accounts
        mintAccounts = {
            authority: authority,
            childCollectionMasterEdition: childCollectionMasterEdition,
            childCollectionMetadata: childCollectionMetadata,
            childCollectionMint: childCollectionMint,
            nft1Metadata: parentNfts[0].metadata,
            nft1TokenAccount: parentNfts[0].tokenAccount,
            nft2Metadata: parentNfts[1].metadata,
            nft2TokenAccount: parentNfts[1].tokenAccount,
            parentCollectionMint: new PublicKey(parentCollectionMint.publicKey) // TODO: This is only needed for testing purposes.
        };
    });

    it('Initializes the collection', async () => {
        // Get the address of the token account that will hold the minted NFT collection
        const tokenAccount = await getAssociatedTokenAddress(
            childCollectionMint,
            payer.publicKey
        );

        // Mint the NFT collection
        const mintTx = await program.methods
            .initialize(IPFS_CID)
            .accounts({
                authority: authority,
                collectionMasterEdition: childCollectionMasterEdition,
                collectionMetadata: childCollectionMetadata,
                collectionMint: childCollectionMint,
                metadataProgram: MPL_TOKEN_METADATA_PROGRAM_ID,
                signer: payer.publicKey,
                tokenAccount: tokenAccount
            })
            .signers([payer])
            .rpc();

        console.log('Mint NFT collection transaction signature', mintTx);

        // Fetch the on-chain metadata
        const { data: metadataAccountData } =
            await provider.connection.getAccountInfo(childCollectionMetadata);
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

    it('Mints an NFT', async () => {
        await mintNft(1, 2, mintAccounts);
    });

    it('Mints a second NFT', async () => {
        await mintNft(2, 3, {
            ...mintAccounts,
            nft1Metadata: parentNfts[1].metadata,
            nft1TokenAccount: parentNfts[1].tokenAccount,
            nft2Metadata: parentNfts[2].metadata,
            nft2TokenAccount: parentNfts[2].tokenAccount
        });
    });

    it('Does not allow duplicate NFTs', async () => {
        let error = false;

        try {
            await mintNft(1, 2, mintAccounts);
        } catch {
            error = true;
        }

        expect(error).toBe(true);
    });

    it(`Does not allow NFT IDs over ${MAX_NFT_ID}`, async () => {
        let error = false;

        try {
            await mintNft(1, MAX_NFT_ID + 1, mintAccounts);
        } catch (e) {
            error = true;
            expect(e.error.errorCode.code).toBe('NFTIDOutOfRange');
        }

        expect(error).toBe(true);
    });

    it(`Does not allow NFT IDs below ${MIN_NFT_ID}`, async () => {
        let error = false;

        try {
            await mintNft(MIN_NFT_ID - 1, 2, mintAccounts);
        } catch (e) {
            error = true;
            expect(e.error.errorCode.code).toBe('NFTIDOutOfRange');
        }

        expect(error).toBe(true);
    });

    it('Does not allow NFT IDs below 0', async () => {
        let error = false;

        try {
            await mintNft(-1, 2, mintAccounts);
        } catch (e) {
            error = true;
            expect(e.message).toBe(
                'The value of "value" is out of range. It must be >= 0 and <= 65535. Received -1'
            );
        }

        expect(error).toBe(true);
    });

    it('Does not allow NFT IDs to be equal', async () => {
        let error = false;

        try {
            await mintNft(1, 1, {
                ...mintAccounts,
                nft2Metadata: parentNfts[0].metadata,
                nft2TokenAccount: parentNfts[0].tokenAccount
            });
        } catch (e) {
            error = true;
            expect(e.error.errorCode.code).toBe('NFTIDEqual');
        }

        expect(error).toBe(true);
    });

    it('Does not allow NFT 1 to be greater than to NFT 2', async () => {
        let error = false;

        try {
            await mintNft(2, 1, {
                ...mintAccounts,
                nft1Metadata: parentNfts[1].metadata,
                nft1TokenAccount: parentNfts[1].tokenAccount,
                nft2Metadata: parentNfts[0].metadata,
                nft2TokenAccount: parentNfts[0].tokenAccount
            });
        } catch (e) {
            error = true;
            expect(e.error.errorCode.code).toBe('NFTIDOutOfOrder');
        }

        expect(error).toBe(true);
    });

    it('Does not allow minting without owning the provided NFTs', async () => {
        let error = false;

        try {
            await mintNft(3, 4, {
                ...mintAccounts,
                nft1Metadata: parentNfts[2].metadata,
                nft1TokenAccount: parentNfts[2].tokenAccount,
                nft2Metadata: parentNfts[3].metadata,
                nft2TokenAccount: parentNfts[3].tokenAccount
            });
        } catch (e) {
            error = true;
            expect(e.error.errorCode.code).toBe('NFTNotOwned');
        }

        expect(error).toBe(true);
    });
});
