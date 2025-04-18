use anchor_lang::prelude::*;
use anchor_spl::{
    metadata::{
        mpl_token_metadata::instructions::{
            FreezeDelegatedAccountCpi, FreezeDelegatedAccountCpiAccounts,
        },
        MasterEditionAccount, Metadata, MetadataAccount,
    },
    token::{approve, Approve, Mint, Token, TokenAccount},
};

use crate::error::StakeError;
use crate::state::{StakeAccount, StakeConfig, UserAccount};

#[derive(Accounts)]
pub struct Stake<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    pub nft_mint: Account<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = nft_mint,
        associated_token::authority = user,
    )]
    pub nft_mint_ata: Account<'info, TokenAccount>,
    pub collection_mint: Account<'info, Mint>,
    #[account(
        seeds = [
            b"metadata",
            metadata_program.key().as_ref(),
            nft_mint.key().as_ref(),
        ],
        bump,
        seeds::program = metadata_program.key(),
        constraint = metadata.collection.as_ref().unwrap().key.as_ref() == collection_mint.key().as_ref(),
        constraint = metadata.collection.as_ref().unwrap().verified,
    )]
    pub metadata: Account<'info, MetadataAccount>,

    #[account(
        seeds = [
            b"metadata",
            metadata_program.key().as_ref(),
            nft_mint.key().as_ref(),
            b"edition"
        ],
        seeds::program = metadata_program.key(),
        bump,
    )]
    pub edition: Account<'info, MasterEditionAccount>,

    #[account(
        seeds = [b"config".as_ref()],
        bump = config.bump,
    )]
    pub config: Account<'info, StakeConfig>,

    #[account(
        mut,
        seeds = [
            b"user".as_ref(),
            user.key().as_ref(),
        ],
        bump = user_account.bump,
    )]
    pub user_account: Account<'info, UserAccount>,

    #[account(
        init,
        payer = user,
        space = StakeAccount::INIT_SPACE+8,
        seeds = [
            b"stake_account",
            nft_mint.key().as_ref(),
            config.key().as_ref(),
        ],
        bump,
    )]
    pub stake_account: Account<'info, StakeAccount>,

    pub metadata_program: Program<'info, Metadata>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

impl Stake<'_> {
    pub fn stake(&mut self, bumps: &StakeBumps) -> Result<()> {
        require!(
            self.user_account.amount_staked <= self.config.max_stake,
            StakeError::MaxStakeReached
        );

        self.user_account.amount_staked += 1;

        let clock = Clock::get()?;
        self.stake_account.set_inner(StakeAccount {
            owner: self.user.key(),
            nft_mint: self.nft_mint.key(),
            staked_at: clock.unix_timestamp,
            bump: bumps.stake_account,
        });

        let cpi_program = self.token_program.to_account_info();

        let cpi_accounts = Approve {
            to: self.nft_mint_ata.to_account_info(),
            delegate: self.stake_account.to_account_info(),
            authority: self.user.to_account_info(),
        };

        let cpi_cpx = CpiContext::new(cpi_program, cpi_accounts);

        approve(cpi_cpx, 1)?;

        // got authority to freeze the account

        let cpi_program = &self.metadata_program.to_account_info();
        let token_program = &self.token_program.to_account_info();

        let cpi_accounts = FreezeDelegatedAccountCpiAccounts {
            delegate: &self.stake_account.to_account_info(),
            token_account: &self.nft_mint_ata.to_account_info(),
            edition: &self.edition.to_account_info(),
            mint: &self.nft_mint.to_account_info(),
            token_program,
        };

        let seeds = &[
            b"stake_account",
            self.nft_mint.to_account_info().key.as_ref(),
            self.config.to_account_info().key.as_ref(),
            &[self.stake_account.bump],
        ];
        let signer_seeds = &[&seeds[..]];

        FreezeDelegatedAccountCpi::new(cpi_program, cpi_accounts).invoke_signed(signer_seeds)?;

        // thawDelegatedAccountCpi()
        // revoke(token_program_id, source_pubkey, owner_pubkey, signer_pubkeys);

        Ok(())
    }
}
