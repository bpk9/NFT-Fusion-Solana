use {
    anchor_lang::prelude::*,
    anchor_spl::{
        associated_token::{ self, AssociatedToken },
        token::{ self, Mint, MintTo, Token, TokenAccount }
    },
    mpl_token_metadata::{
        ID as TOKEN_METADATA_ID,
        accounts::Metadata,
        instructions::{ CreateMasterEditionV3CpiBuilder, CreateMetadataAccountV3CpiBuilder, SetAndVerifyCollectionCpiBuilder },
        types::{ Collection, Creator, DataV2 }
    }
};

declare_id!("5gJMGxawUbdqfRpdVKj1eJrPfgMQ1Rrroa5WEoidLWQU");

#[program]
pub mod nft_fusion_solana {
    use super::*;

    const BASE_URI: &str = "https://mygateway.mypinata.cloud/ipfs/";
    const COLLECTION_NAME: &str = "Child Collection";
    const COLLECTION_SYMBOL: &str = "CC";
    const CREATOR_ADDRESS: &str = "LFujUyg8wPiwqt2DFGdSe6wApqwNvpf4zdMebdPVMbz";
    const MAX_NFT_ID: u16 = 1000;
    const MIN_NFT_ID: u16 = 1;
    // pub const PARENT_COLLECTION_MINT: &str = "J1S9H3QjnRtBbbuD4HjPV6RpRhwuk4zKbxsnCHuTgh9w";

    pub fn initialize(ctx: Context<Initialize>, cid: String) -> Result<()> {
        // TODO: Ensure the program isn't already initialized

        // Derive the Seeds for the Authority PDA
        let authority_seeds = &[ctx.accounts.signer.key.as_ref(), b"authority", &[ctx.bumps.authority]];

        // Create the Associated Token Account for the Authority
        associated_token::create(
            CpiContext::new(
                ctx.accounts.associated_token_program.to_account_info(),
                associated_token::Create {
                    payer: ctx.accounts.signer.to_account_info(),
                    associated_token: ctx.accounts.token_account.to_account_info(),
                    authority: ctx.accounts.signer.to_account_info(),
                    mint: ctx.accounts.collection_mint.to_account_info(),
                    system_program: ctx.accounts.system_program.to_account_info(),
                    token_program: ctx.accounts.token_program.to_account_info()
                }
            )
        )?;
    
        // Mint the Collection NFT
        token::mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(), 
                MintTo {
                    mint: ctx.accounts.collection_mint.to_account_info(),
                    to: ctx.accounts.token_account.to_account_info(),
                    authority: ctx.accounts.authority.to_account_info(),
                },
                &[&authority_seeds[..]]
            ), 
            1
        )?;

        // Create the Metadata Account for the NFT Collection
        CreateMetadataAccountV3CpiBuilder::new(&ctx.accounts.metadata_program.to_account_info())
            .data(DataV2 {
                name: COLLECTION_NAME.to_string(),
                symbol: COLLECTION_SYMBOL.to_string(),
                uri: format!("{}{}", BASE_URI, cid),
                seller_fee_basis_points: 500, // 5%
                creators: Some(vec![
                    Creator {
                        address: ctx.accounts.authority.key(),
                        share: 0,
                        verified: true
                    },
                    Creator {
                        address: CREATOR_ADDRESS.parse().unwrap(),
                        share: 100,
                        verified: false
                    }
                ]),
                collection: None,
                uses: None
            })
            .is_mutable(false)
            .metadata(&ctx.accounts.collection_metadata.to_account_info())
            .mint(&ctx.accounts.collection_mint.to_account_info())
            .mint_authority(&ctx.accounts.authority.to_account_info())
            .payer(&ctx.accounts.signer.to_account_info())
            .system_program(&ctx.accounts.system_program.to_account_info())
            .update_authority(&ctx.accounts.authority.to_account_info(), true)
            .invoke_signed(&[&authority_seeds[..]])?;

        // Create the Master Edition NFT
        CreateMasterEditionV3CpiBuilder::new(&ctx.accounts.metadata_program.to_account_info())
            .edition(&ctx.accounts.collection_master_edition.to_account_info())
            .max_supply(0)
            .metadata(&ctx.accounts.collection_metadata.to_account_info())
            .mint(&ctx.accounts.collection_mint.to_account_info())
            .mint_authority(&ctx.accounts.authority.to_account_info())
            .payer(&ctx.accounts.signer.to_account_info())
            .system_program(&ctx.accounts.system_program.to_account_info())
            .token_program(&ctx.accounts.token_program.to_account_info())
            .update_authority(&ctx.accounts.authority.to_account_info())
            .invoke_signed(&[&authority_seeds[..]])?;

        Ok(())
    }

    pub fn mint_nft(ctx: Context<MintNFT>, cid: String, nft_1: u16, nft_2: u16) -> Result<()> {
        // Ensure NFT IDs are not equal
        if nft_1 == nft_2 {
            return Err(NftFusionError::NFTIDEqual.into());
        }
        
        // Ensure NFT 1 is less than NFT 2
        if nft_1 > nft_2 {
            return Err(NftFusionError::NFTIDOutOfOrder.into());
        }
        
        // Ensure both NFT IDs are within the valid range
        if nft_1 < MIN_NFT_ID || nft_1 > MAX_NFT_ID || nft_2 < MIN_NFT_ID || nft_2 > MAX_NFT_ID {
            return Err(NftFusionError::NFTIDOutOfRange.into());
        }

        // Check that the ATA Owner is the signer
        if ctx.accounts.nft_1_token_account.owner != ctx.accounts.signer.key() || ctx.accounts.nft_2_token_account.owner != ctx.accounts.signer.key() {
            return Err(NftFusionError::TokenAccountIncorrectOwner.into());
        }

        // Check if the signer owns both NFTs
        if ctx.accounts.nft_1_token_account.amount == 0 || ctx.accounts.nft_2_token_account.amount == 0 {
            return Err(NftFusionError::NFTNotOwned.into());
        }

        // Extract metadata for Parent Collection
        let parent_collection_mint = ctx.accounts.parent_collection_mint.to_account_info().key();

        // Extract metadata for NFT 1
        let nft_1_metadata = Metadata::safe_deserialize(&ctx.accounts.nft_1_metadata.to_account_info().data.borrow())?;
        let nft_1_mint = nft_1_metadata.mint;
        let nft_1_collection = nft_1_metadata.collection.as_ref().unwrap();

        // Extract metadata for NFT 2
        let nft_2_metadata = Metadata::safe_deserialize(&ctx.accounts.nft_2_metadata.to_account_info().data.borrow())?;
        let nft_2_mint = nft_2_metadata.mint;
        let nft_2_collection = nft_2_metadata.collection.as_ref().unwrap();
        
        // Check in the metaplex metadata if collection.key=PARENT_COLLECTION_MINT
        if nft_1_collection.key != parent_collection_mint || nft_2_collection.key != parent_collection_mint {
            return Err(NftFusionError::MetadataCollectionIncorrect.into());
        }

        // Check in the metaplex metadata if collection.verified=true
        if !nft_1_collection.verified || !nft_2_collection.verified {
            return Err(NftFusionError::MetadataCollectionNotVerified.into());
        }

        // Check that the mint from the ATA is the same as the mint on the metadata
        if ctx.accounts.nft_1_token_account.mint != nft_1_mint || ctx.accounts.nft_2_token_account.mint != nft_2_mint {
            return Err(NftFusionError::MetadataIncorrect.into());
        }
        
        // Derive the seeds for the authority PDA
        let authority_seeds = &[ctx.accounts.signer.key.as_ref(), b"authority", &[ctx.bumps.authority]];

        // Create the Associated Token Account for the Signer
        associated_token::create(
            CpiContext::new(
                ctx.accounts.associated_token_program.to_account_info(),
                associated_token::Create {
                    payer: ctx.accounts.signer.to_account_info(),
                    associated_token: ctx.accounts.token_account.to_account_info(),
                    authority: ctx.accounts.signer.to_account_info(),
                    mint: ctx.accounts.mint.to_account_info(),
                    system_program: ctx.accounts.system_program.to_account_info(),
                    token_program: ctx.accounts.token_program.to_account_info()
                }
            )
        )?;

        // Mint the NFT
        token::mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(), 
                MintTo {
                    mint: ctx.accounts.mint.to_account_info(),
                    to: ctx.accounts.token_account.to_account_info(),
                    authority: ctx.accounts.authority.to_account_info(),
                },
                &[&authority_seeds[..]]
            ), 
            1
        )?;

        // Create the Metadata Account
        CreateMetadataAccountV3CpiBuilder::new(&ctx.accounts.metadata_program.to_account_info())
            .data(DataV2 {
                name: format!("#{} + #{}", nft_1, nft_2),
                symbol: COLLECTION_SYMBOL.to_string(),
                uri: format!("{}{}", BASE_URI, cid),
                seller_fee_basis_points: 500, // 5%
                creators: Some(vec![
                    Creator {
                        address: ctx.accounts.authority.key(),
                        share: 0,
                        verified: true
                    },
                    Creator {
                        address: CREATOR_ADDRESS.parse().unwrap(),
                        share: 100,
                        verified: false
                    }
                ]),
                collection: Some(Collection {
                    key: ctx.accounts.child_collection_mint.key(),
                    verified: false
                }),
                uses: None,
            })
            .is_mutable(false)
            .metadata(&ctx.accounts.metadata.to_account_info())
            .mint(&ctx.accounts.mint.to_account_info())
            .mint_authority(&ctx.accounts.authority.to_account_info())
            .payer(&ctx.accounts.signer.to_account_info())
            .system_program(&ctx.accounts.system_program.to_account_info())
            .update_authority(&ctx.accounts.authority.to_account_info(), true)
            .invoke_signed(&[&authority_seeds[..]])?;

        // Verify the NFT Collection
        SetAndVerifyCollectionCpiBuilder::new(&ctx.accounts.metadata_program.to_account_info())
            .collection(&ctx.accounts.child_collection_metadata.to_account_info())
            .collection_authority(&ctx.accounts.authority.to_account_info())
            .collection_master_edition_account(&ctx.accounts.child_collection_master_edition.to_account_info())
            .collection_mint(&ctx.accounts.child_collection_mint.to_account_info())
            .metadata(&ctx.accounts.metadata.to_account_info())
            .payer(&ctx.accounts.signer.to_account_info())
            .update_authority(&ctx.accounts.authority.to_account_info())
            .invoke_signed(&[&authority_seeds[..]])?;

        // TODO: Create the master edition account

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(address = anchor_spl::associated_token::ID)]
    pub associated_token_program: Program<'info, AssociatedToken>,

    /// CHECK: This PDA is only used as the mint/update authority of the NFT collection.
    #[account(
        init,
        payer = signer,
        space = 0,
        seeds = [signer.key.as_ref(), b"authority"],
        bump
    )]
    pub authority: UncheckedAccount<'info>,

    /// CHECK: This PDA is initialized and checked by the program.
    #[account(mut)]
    pub collection_master_edition: UncheckedAccount<'info>,

    /// CHECK: This PDA is initialized and checked by the program.
    #[account(mut)]
    pub collection_metadata: UncheckedAccount<'info>,

    #[account(
        init,
        payer = signer,
        seeds = [b"child-collection-mint"],
        bump,
        mint::decimals = 0,
        mint::authority = authority,
        mint::freeze_authority = authority
    )]
    pub collection_mint: Account<'info, Mint>,

    /// CHECK: This program is constrained to only use the MPL Token Metadata program.
    #[account(address = TOKEN_METADATA_ID)]
    pub metadata_program: UncheckedAccount<'info>,

    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(address = anchor_lang::system_program::ID)]
    pub system_program: Program<'info, System>,

    /// CHECK: This account is initialized by the program.
    #[account(mut)]
    pub token_account: UncheckedAccount<'info>,

    #[account(address = anchor_spl::token::ID)]
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
#[instruction(cid: String, nft_1: u16, nft_2: u16)]
pub struct MintNFT<'info> {
    #[account(address = anchor_spl::associated_token::ID)]
    pub associated_token_program: Program<'info, AssociatedToken>,

    /// CHECK: This PDA is only used as the mint/update authority of the NFT.
    #[account(
        mut,
        seeds = [signer.key.as_ref(), b"authority"],
        bump
    )]
    pub authority: UncheckedAccount<'info>,

    /// CHECK: This PDA is initialized and checked by the program.
    #[account(mut)]
    pub child_collection_master_edition: UncheckedAccount<'info>,

    /// CHECK: This PDA is initialized and checked by the program.
    #[account(mut)]
    pub child_collection_metadata: UncheckedAccount<'info>,

    #[account(
        seeds = [b"child-collection-mint"],
        bump
    )]
    pub child_collection_mint: Account<'info, Mint>,

    /// CHECK: This PDA is initialized and checked by the program.
    #[account(mut)]
    pub metadata: UncheckedAccount<'info>,

    /// CHECK: This program is constrained to only use the MPL Token Metadata program.
    #[account(address = TOKEN_METADATA_ID)]
    pub metadata_program: UncheckedAccount<'info>,

    #[account(
        init,
        payer = signer,
        seeds = [nft_1.to_be_bytes().as_ref(), nft_2.to_be_bytes().as_ref()],
        bump,
        mint::decimals = 0,
        mint::authority = authority,
    )]
    pub mint: Account<'info, Mint>,

    /// CHECK: This account is checked by the program.
    pub nft_1_metadata: UncheckedAccount<'info>,

    #[account(constraint = nft_1_token_account.owner == signer.key())]
    pub nft_1_token_account: Account<'info, TokenAccount>,
    
    /// CHECK: This account is checked by the program.
    pub nft_2_metadata: UncheckedAccount<'info>,

    #[account(constraint = nft_2_token_account.owner == signer.key())]
    pub nft_2_token_account: Account<'info, TokenAccount>,

    // TODO: This is only needed for testing purposes. This should be hard-coded.
    pub parent_collection_mint: Account<'info, Mint>,

    #[account(mut)]
    pub signer: Signer<'info>,
    
    /// CHECK: This account is initialized by the program.
    #[account(mut)]
    pub token_account: UncheckedAccount<'info>,

    #[account(address = anchor_lang::system_program::ID)]
    pub system_program: Program<'info, System>,
    
    #[account(address = anchor_spl::token::ID)]
    pub token_program: Program<'info, Token>,
}

// Error codes
#[error_code]
pub enum NftFusionError {
    #[msg("One or more of the provided metadata accounts does not belong to the parent collection")]
    MetadataCollectionIncorrect,

    #[msg("One or more of the provided metadata accounts contains an unverified collection")]
    MetadataCollectionNotVerified,

    #[msg("One or more of the provided metadata accounts does not match the associated token account")]
    MetadataIncorrect,

    #[msg("NFT IDs can not be equal")]
    NFTIDEqual,

    #[msg("User does not own the provided NFTs")]
    NFTNotOwned,

    #[msg("NFT 1 must be less than NFT 2")]
    NFTIDOutOfOrder,

    #[msg(format!("NFT IDs must be between {} and {}.", MIN_NFT_ID, MAX_NFT_ID))]
    NFTIDOutOfRange,

    #[msg("One or more of the token accounts does not belong to the signer")]
    TokenAccountIncorrectOwner
}