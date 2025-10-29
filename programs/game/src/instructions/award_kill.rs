use anchor_lang::prelude::*;
use crate::state::GamePlayer;

/// Award a kill to the shooter
/// This should be called after a successful kill in the shoot instruction
pub fn handler(ctx: Context<AwardKill>, score_points: u32) -> Result<()> {
    let shooter = &mut ctx.accounts.shooter;
    let clock = Clock::get()?;

    shooter.kills = shooter.kills.saturating_add(1);
    shooter.score = shooter.score.saturating_add(score_points);
    shooter.last_update = clock.unix_timestamp;

    msg!(
        "Player {} awarded kill. Stats - Kills: {}, Deaths: {}, Score: {}",
        shooter.authority,
        shooter.kills,
        shooter.deaths,
        shooter.score
    );

    Ok(())
}

#[derive(Accounts)]
pub struct AwardKill<'info> {
    #[account(mut)]
    pub shooter: Account<'info, GamePlayer>,

    pub authority: Signer<'info>,
}
