use anchor_lang::prelude::*;
use ephemeral_rollups_sdk::anchor::delegate;
use ephemeral_rollups_sdk::cpi::DelegateConfig;

/// Delegate GamePlayer account to game ephemeral rollup
/// This should be called AFTER init_game_player
/// IMPORTANT: Pass game_id as parameter to avoid reading from account after delegation
pub fn handler(ctx: Context<DelegateGamePlayer>, game_id: Pubkey) -> Result<()> {
    let player_key = ctx.accounts.game_player.key();
    let authority_key = ctx.accounts.authority.key();

    // Get validator from remaining accounts if provided
    let validator = if !ctx.remaining_accounts.is_empty() {
        ctx.remaining_accounts.first().map(|acc| acc.key())
    } else {
        None
    };

    // Seeds for the GamePlayer PDA
    let seeds = &[
        b"game_player".as_ref(),
        authority_key.as_ref(),
        game_id.as_ref(),
    ];

    // Delegate GamePlayer account to game ephemeral rollup
    ctx.accounts.delegate_game_player(
        &ctx.accounts.signer,
        seeds,
        DelegateConfig {
            validator,
            commit_frequency_ms: 5000,
            ..Default::default()
        },
    )?;

    if let Some(validator_key) = validator {
        msg!("GamePlayer {} delegated to game ephemeral via validator {}",
            player_key, validator_key);
    } else {
        msg!("GamePlayer {} delegated to game ephemeral (no specific validator)",
            player_key);
    }

    Ok(())
}

/// Delegate GamePlayer account to game ephemeral rollup
#[delegate]
#[derive(Accounts)]
pub struct DelegateGamePlayer<'info> {
    /// CHECK: The GamePlayer PDA to delegate to game ephemeral
    #[account(mut, del)]
    pub game_player: AccountInfo<'info>,

    /// The player's authority/wallet
    pub authority: Signer<'info>,

    /// The signer paying for delegation
    #[account(mut)]
    pub signer: Signer<'info>,
}
