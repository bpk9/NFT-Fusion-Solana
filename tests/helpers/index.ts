import * as anchor from '@coral-xyz/anchor';
import {
    Collection,
    Creator,
    MPL_TOKEN_METADATA_PROGRAM_ID,
    TokenStandard,
    Uses,
    getMetadataAccountDataSerializer
} from '@metaplex-foundation/mpl-token-metadata';
import { getAssociatedTokenAddress } from '@solana/spl-token';
import { PublicKey } from '@solana/web3.js';
import {
    COLLECTION_SYMBOL,
    IPFS_CID,
    payer,
    program,
    provider
} from '../constants';
import { MintNftAccounts } from '../interfaces';

const CREATOR_ADDRESS: string = 'LFujUyg8wPiwqt2DFGdSe6wApqwNvpf4zdMebdPVMbz';

export const mintNft = async (
    nft1: number,
    nft2: number,
    accounts: MintNftAccounts
) => {
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
    const tokenAccount = await getAssociatedTokenAddress(mint, payer.publicKey);

    // Mint the NFT
    const mintTx = await program.methods
        .mintNft(IPFS_CID, nft1, nft2)
        .accounts({
            ...accounts,
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
        getMetadataAccountDataSerializer().deserialize(metadataAccountData)[0];

    // Parse the on-chain metadata
    const creators = (onChainMetadata.creators as any).value as Array<Creator>;
    const tokenStandard = (onChainMetadata.tokenStandard as any)
        .value as TokenStandard;
    const collection = (onChainMetadata.collection as any).value as Collection;
    const uses = (onChainMetadata.uses as any).value as Uses;

    // Assert that the on-chain metadata matches the expected metadata
    expect(onChainMetadata.name).toBe(`#${nft1} + #${nft2}`);
    expect(onChainMetadata.symbol).toBe(COLLECTION_SYMBOL);
    expect(onChainMetadata.uri).toBe(
        `https://mygateway.mypinata.cloud/ipfs/${IPFS_CID}`
    );
    expect(onChainMetadata.sellerFeeBasisPoints).toBe(500); // 5%
    expect(creators).toHaveLength(2);
    expect(creators[0].address.toString()).toBe(accounts.authority.toString());
    expect(creators[0].verified).toBe(true);
    expect(creators[0].share).toBe(0);
    expect(creators[1].address.toString()).toBe(CREATOR_ADDRESS);
    expect(creators[1].verified).toBe(false);
    expect(creators[1].share).toBe(100);
    expect(onChainMetadata.isMutable).toBe(false);
    expect(tokenStandard).toBe(1); // TODO: Make this a pNFT
    expect(collection.key).toBe(accounts.childCollectionMint.toString());
    expect(collection.verified).toBe(true);
    expect(uses).toBe(undefined);
};
