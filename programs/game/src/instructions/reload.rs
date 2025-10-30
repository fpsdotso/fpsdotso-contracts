use anchor_lang::prelude::*;
use crate::state::GamePlayer;

#[error_code]
pub enum ReloadError {
    #[msg("Magazine is already full. No need to reload.")]
    MagazineAlreadyFull,
    
    #[msg("Already reloading. Wait for reload to complete.")]
    AlreadyReloading,
    
    #[msg("Not reloading. Call start_reload first.")]
    NotReloading,
    
    #[msg("Reload not ready yet. Must wait 0.5 seconds after starting reload.")]
    ReloadNotReady,
}

const MAX_BULLETS: u8 = 10;

/// Start the reload process
/// Marks the player as reloading and records the start timestamp
pub fn start_reload_handler(ctx: Context<StartReload>) -> Result<()> {
    let player = &mut ctx.accounts.game_player;
    let clock = Clock::get()?;

    // Check if magazine is already full
    require!(player.bullet_count < MAX_BULLETS, ReloadError::MagazineAlreadyFull);

    // Check if already reloading (if current time is less than reload_start_timestamp + 1 second)
    if player.reload_start_timestamp > 0 {
        let reload_end_time = player.reload_start_timestamp.checked_add(1)
            .ok_or(ProgramError::ArithmeticOverflow)?;
        require!(clock.unix_timestamp >= reload_end_time, ReloadError::AlreadyReloading);
    }

    // Start reloading
    player.reload_start_timestamp = clock.unix_timestamp;
    player.last_update = clock.unix_timestamp;

    msg!(
        "Player {} started reloading at timestamp {}",
        player.authority,
        clock.unix_timestamp
    );

    Ok(())
}

/// Complete the reload process
/// This refills the magazine to 10 bullets after 0.5 seconds have passed
pub fn reload_handler(ctx: Context<Reload>) -> Result<()> {
    let player = &mut ctx.accounts.game_player;
    let clock = Clock::get()?;

    // Check if player is reloading
    require!(player.reload_start_timestamp > 0, ReloadError::NotReloading);

    // Check if 1 seconds have passed (using milliseconds for precision)
    let elapsed = clock.unix_timestamp.checked_sub(player.reload_start_timestamp)
        .ok_or(ProgramError::ArithmeticOverflow)?;
    require!(elapsed >= 1, ReloadError::ReloadNotReady); // At least 1 second (since unix_timestamp is in seconds)

    // Complete reload
    player.bullet_count = MAX_BULLETS;
    player.reload_start_timestamp = 0; // Clear reloading state
    player.last_update = clock.unix_timestamp;

    msg!(
        "Player {} completed reload. Bullets: {}/{}",
        player.authority,
        player.bullet_count,
        MAX_BULLETS
    );

    Ok(())
}

#[derive(Accounts)]
pub struct StartReload<'info> {
    #[account(mut)]
    pub game_player: Account<'info, GamePlayer>,

    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct Reload<'info> {
    #[account(mut)]
    pub game_player: Account<'info, GamePlayer>,

    pub authority: Signer<'info>,
}
