use {
    anchor_lang::prelude::*,
    anchor_spl::{
        associated_token::{ self, AssociatedToken },
        token::{ self, Mint, MintTo, Token }
    },
    mpl_token_metadata::{
        instructions::CreateMetadataAccountV3CpiBuilder, ID as TOKEN_METADATA_ID,
        types::{ Creator, DataV2 }
    }
};

declare_id!("5gJMGxawUbdqfRpdVKj1eJrPfgMQ1Rrroa5WEoidLWQU");

#[program]
pub mod nft_fusion_solana {
    use super::*;

    pub fn mint_nft(ctx: Context<MintNFT>) -> Result<()> {
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
                name: "#1".to_string(), // BK TODO: Auto-increment name
                symbol: "NFS".to_string(),
                uri: "https://example.com/nft-metadata".to_string(), // BK TODO: Add URI Metadata via IPFS
                seller_fee_basis_points: 500, // 5%
                creators: Some(vec![
                    Creator {
                        address: ctx.accounts.authority.key(),
                        share: 0,
                        verified: true
                    },
                    Creator {
                        address: "LFujUyg8wPiwqt2DFGdSe6wApqwNvpf4zdMebdPVMbz".parse().unwrap(),
                        share: 100,
                        verified: false
                    }
                ]),
                collection: None, // BK TODO: Create the collection
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

        Ok(())
    }
}

#[derive(Accounts)]
pub struct MintNFT<'info> {
    #[account(address = anchor_spl::associated_token::ID)]
    pub associated_token_program: Program<'info, AssociatedToken>,

    /// CHECK: This PDA is only used as the mint/update authority of the NFT.
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
    pub metadata: UncheckedAccount<'info>,

    /// CHECK: This program is constrained to only use the MPL Token Metadata program.
    #[account(address = TOKEN_METADATA_ID)]
    pub metadata_program: UncheckedAccount<'info>,

    #[account(
        init,
        payer = signer,
        seeds = [signer.key.as_ref(), b"nfs-mint"],
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