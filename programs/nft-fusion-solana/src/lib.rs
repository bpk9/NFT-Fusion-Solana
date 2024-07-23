use {
    anchor_lang::prelude::*,
    anchor_spl::{
        associated_token::{ self, AssociatedToken },
        token::{ self, Mint, MintTo, Token }
    },
    mpl_token_metadata::{
        ID as TOKEN_METADATA_ID,
        instructions::{ CreateMasterEditionV3CpiBuilder, CreateMetadataAccountV3CpiBuilder, SetAndVerifyCollectionCpiBuilder }, 
        types::{ Collection, Creator, DataV2 }
    }
};

declare_id!("5gJMGxawUbdqfRpdVKj1eJrPfgMQ1Rrroa5WEoidLWQU");

#[program]
pub mod nft_fusion_solana {
    use super::*;

    const BASE_URI: &str = "https://mygateway.mypinata.cloud/ipfs/";
    const COLLECTION_NAME: &str = "NFT Fusion Solana";
    const COLLECTION_SYMBOL: &str = "NFS";
    const CREATOR_ADDRESS: &str = "LFujUyg8wPiwqt2DFGdSe6wApqwNvpf4zdMebdPVMbz";
    const MAX_NFT_ID: u16 = 1000;
    const MIN_NFT_ID: u16 = 1;

    pub fn initialize(ctx: Context<Initialize>, cid: String) -> Result<()> {
        // BK TODO: Ensure the program isn't already initialized

        // Derive the Seeds for the Authority PDA
        let authority_seeds = &[ctx.accounts.signer.key.as_ref(), b"nfs-authority", &[ctx.bumps.authority]];

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
        // Check if either NFT ID is out of range
        if nft_1 < MIN_NFT_ID || nft_1 > MAX_NFT_ID || nft_2 < MIN_NFT_ID || nft_2 > MAX_NFT_ID {
            return Err(NftFusionError::NFTIDOutOfRange.into());
        }

        // Derive the seeds for the authority PDA
        let authority_seeds = &[ctx.accounts.signer.key.as_ref(), b"nfs-authority", &[ctx.bumps.authority]];

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
                    key: ctx.accounts.collection_mint.key(),
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

        // BK TODO: Verify the NFT Collection
        SetAndVerifyCollectionCpiBuilder::new(&ctx.accounts.metadata_program.to_account_info())
            .collection(&ctx.accounts.collection_metadata.to_account_info())
            .collection_authority(&ctx.accounts.authority.to_account_info())
            .collection_master_edition_account(&ctx.accounts.collection_master_edition.to_account_info())
            .collection_mint(&ctx.accounts.collection_mint.to_account_info())
            .metadata(&ctx.accounts.metadata.to_account_info())
            .payer(&ctx.accounts.signer.to_account_info())
            .update_authority(&ctx.accounts.authority.to_account_info())
            .invoke_signed(&[&authority_seeds[..]])?;

        // BK TODO: Create the master edition account

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
        seeds = [signer.key.as_ref(), b"nfs-authority"],
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
        seeds = [b"nfs-collection-mint"],
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
        seeds = [signer.key.as_ref(), b"nfs-authority"],
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
        seeds = [b"nfs-collection-mint"],
        bump
    )]
    pub collection_mint: Account<'info, Mint>,

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
    #[msg(format!("NFT IDs must be between {} and {}.", MIN_NFT_ID, MAX_NFT_ID))]
    NFTIDOutOfRange,
}
