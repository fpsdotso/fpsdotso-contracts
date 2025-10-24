use anchor_lang::prelude::*;
use ephemeral_rollups_sdk::anchor::commit;
use ephemeral_rollups_sdk::ephem::commit_and_undelegate_accounts;
use crate::state::GamePlayer;

/// Undelegate GamePlayer account from game ephemeral rollup
/// This should be called when the game ends
pub fn handler(ctx: Context<UndelegateGamePlayer>) -> Result<()> {
    let player_key = ctx.accounts.game_player.key();

    msg!("Undelegating GamePlayer {} from game ephemeral", player_key);

    // Commit final state and undelegate from game ephemeral rollup
    commit_and_undelegate_accounts(
        &ctx.accounts.payer,
        vec![&ctx.accounts.game_player.to_account_info()],
        &ctx.accounts.magic_context,
        &ctx.accounts.magic_program,
    )?;

    msg!("GamePlayer {} successfully undelegated from game ephemeral", player_key);

    Ok(())
}

/// Account context for undelegating GamePlayer from game ephemeral
#[commit]
#[derive(Accounts)]
pub struct UndelegateGamePlayer<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    /// GamePlayer account to undelegate from game ephemeral
    #[account(mut)]
    pub game_player: Account<'info, GamePlayer>,
}
