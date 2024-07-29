import { PublicKey } from '@solana/web3.js';

export interface NFTAccounts {
    masterEdition: PublicKey;
    metadata: PublicKey;
    mint: PublicKey;
    tokenAccount: PublicKey;
}

export interface MintNftAccounts {
    authority: PublicKey;
    childCollectionMasterEdition: PublicKey;
    childCollectionMetadata: PublicKey;
    childCollectionMint: PublicKey;
    nft1Metadata: PublicKey;
    nft1TokenAccount: PublicKey;
    nft2Metadata: PublicKey;
    nft2TokenAccount: PublicKey;
    parentCollectionMint: PublicKey; // TODO: This is only needed for testing purposes.
}
