use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::{ self, AssociatedToken },
    token::{ self, Mint, MintTo, Token }
};

declare_id!("5gJMGxawUbdqfRpdVKj1eJrPfgMQ1Rrroa5WEoidLWQU");

#[program]
pub mod nft_fusion_solana {
    use super::*;

    pub fn mint_nft(ctx: Context<MintNFT>) -> Result<()> {
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
        let mint_authority_seeds = &[ctx.accounts.signer.key.as_ref(), b"nfs-mint-authority", &[ctx.bumps.mint_authority]];;
        token::mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(), 
                MintTo {
                    mint: ctx.accounts.mint.to_account_info(),
                    to: ctx.accounts.token_account.to_account_info(),
                    authority: ctx.accounts.mint_authority.to_account_info(),
                },
                &[&mint_authority_seeds[..]]
            ), 
            1
        )?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct MintNFT<'info> {
    #[account(address = anchor_spl::associated_token::ID)]
    pub associated_token_program: Program<'info, AssociatedToken>,

    #[account(
        init,
        payer = signer,
        seeds = [signer.key.as_ref(), b"nft-fusion-solana-mint"],
        bump,
        mint::decimals = 0,
        mint::authority = mint_authority,
    )]
    pub mint: Account<'info, Mint>,

    /// CHECK: This PDA is only used to mint the NFT.
    #[account(
        init,
        payer = signer,
        space = 0,
        seeds = [signer.key.as_ref(), b"nfs-mint-authority"],
        bump
    )]
    pub mint_authority: UncheckedAccount<'info>,

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