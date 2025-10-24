use anchor_lang::prelude::*;
use crate::error::UndelegatePlayerError;
use crate::constants::PLAYER_SEED;
use ephemeral_rollups_sdk::anchor::commit;
use ephemeral_rollups_sdk::ephem::commit_and_undelegate_accounts;

/// Undelegate player account from ephemeral rollup
/// This should be called when player leaves the game or game ends
pub fn handler(ctx: Context<UndelegatePlayer>) -> Result<()> {
    let player = &ctx.accounts.player;

    require!(player.has_logged_in, UndelegatePlayerError::PlayerNotRegistered);

    msg!("Undelegating player {}", player.key());

    // Commit final state and undelegate from ephemeral rollup
    commit_and_undelegate_accounts(
        &ctx.accounts.payer,
        vec![&ctx.accounts.player.to_account_info()],
        &ctx.accounts.magic_context,
        &ctx.accounts.magic_program,
    )?;

    msg!("Player {} successfully undelegated", player.key());

    Ok(())
}

/// Account context for undelegating player
#[commit]
#[derive(Accounts)]
pub struct UndelegatePlayer<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        mut,
        seeds = [PLAYER_SEED.as_bytes(), payer.key().as_ref()],
        bump
    )]
    pub player: Account<'info, crate::state::Player>,
}
