import * as anchor from '@coral-xyz/anchor';
import { Program } from '@coral-xyz/anchor';
import { Signer, Keypair } from '@solana/web3.js';
import { NftFusionSolana } from '../../target/types/nft_fusion_solana';

// Solana
export const ONE_SOL: number = 1000000000;

// Program
export const MAX_NFT_ID: number = 1000;
export const MIN_NFT_ID: number = 1;

// Parent Collection
export const PARENT_COLLECTION_NAME: string = 'Parent Collection';
export const PARENT_COLLECTION_SYMBOL: string = 'PC';

// Child Collection
export const COLLECTION_NAME: string = 'Child Collection';
export const COLLECTION_SYMBOL: string = 'CC';

// NFT
export const IPFS_CID: string = 'CID'; // Mock IPFS CID

// Anchor
export const payer: Signer = Keypair.generate();
export const provider: anchor.AnchorProvider = anchor.AnchorProvider.env();
export const program = anchor.workspace
    .NftFusionSolana as Program<NftFusionSolana>;
