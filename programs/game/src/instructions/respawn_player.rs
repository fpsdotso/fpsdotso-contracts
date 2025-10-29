use anchor_lang::prelude::*;
use crate::state::GamePlayer;

#[error_code]
pub enum RespawnError {
    #[msg("Player is still alive and cannot respawn")]
    PlayerStillAlive,
    #[msg("Respawn cooldown not finished. Wait 3 seconds after death.")]
    RespawnCooldownActive,
}

/// Respawn a dead player at a spawn point
/// Enforces 3-second cooldown after death
pub fn handler(
    ctx: Context<RespawnPlayer>,
    spawn_x: f32,
    spawn_y: f32,
    spawn_z: f32,
) -> Result<()> {
    let player = &mut ctx.accounts.game_player;
    let clock = Clock::get()?;

    // Check if player is dead
    require!(!player.is_alive, RespawnError::PlayerStillAlive);

    // Check if 3 seconds have passed since death
    const RESPAWN_COOLDOWN_SECONDS: i64 = 3;
    let time_since_death = clock.unix_timestamp - player.death_timestamp;

    require!(
        time_since_death >= RESPAWN_COOLDOWN_SECONDS,
        RespawnError::RespawnCooldownActive
    );

    // Reset player state
    player.health = 100;
    player.is_alive = true;
    player.death_timestamp = 0; // Clear death timestamp

    // Set spawn position
    player.position_x = spawn_x;
    player.position_y = spawn_y;
    player.position_z = spawn_z;

    // Reset rotation to default
    player.rotation_x = 0.0;
    player.rotation_y = 0.0;
    player.rotation_z = 0.0;

    player.last_update = clock.unix_timestamp;

    msg!(
        "Player {} respawned at ({:.2}, {:.2}, {:.2})",
        player.authority,
        spawn_x,
        spawn_y,
        spawn_z
    );

    Ok(())
}

#[derive(Accounts)]
pub struct RespawnPlayer<'info> {
    #[account(mut)]
    pub game_player: Account<'info, GamePlayer>,

    pub authority: Signer<'info>,
}
