use anchor_lang::prelude::*;
use crate::state::GamePlayer;

#[error_code]
pub enum ReloadError {
    #[msg("Magazine is already full. No need to reload.")]
    MagazineAlreadyFull,
}

const MAX_BULLETS: u8 = 10;

/// Reload the gun instantly
/// This refills the magazine to 10 bullets
pub fn handler(ctx: Context<Reload>) -> Result<()> {
    let player = &mut ctx.accounts.game_player;
    let clock = Clock::get()?;

    // Check if magazine is already full
    require!(player.bullet_count < MAX_BULLETS, ReloadError::MagazineAlreadyFull);

    // Reload instantly
    player.bullet_count = MAX_BULLETS;
    player.last_update = clock.unix_timestamp;

    msg!(
        "Player {} reloaded. Bullets: {}/{}",
        player.authority,
        player.bullet_count,
        MAX_BULLETS
    );

    Ok(())
}

#[derive(Accounts)]
pub struct Reload<'info> {
    #[account(mut)]
    pub game_player: Account<'info, GamePlayer>,

    pub authority: Signer<'info>,
}
