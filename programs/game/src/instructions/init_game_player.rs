use anchor_lang::prelude::*;
use crate::state::GamePlayer;

/// Initialize a game player account when a player joins a game
/// This creates the game-specific player state separate from matchmaking
pub fn handler(
    ctx: Context<InitGamePlayer>,
    game_id: Pubkey,
    team: u8,
    spawn_x: f32,
    spawn_y: f32,
    spawn_z: f32,
) -> Result<()> {
    let game_player = &mut ctx.accounts.game_player;
    let clock = Clock::get()?;

    game_player.authority = ctx.accounts.authority.key();
    game_player.game_id = game_id;

    // Set initial position (spawn point)
    game_player.position_x = spawn_x;
    game_player.position_y = spawn_y;
    game_player.position_z = spawn_z;

    // Set initial rotation (facing forward)
    game_player.rotation_x = 0.0;
    game_player.rotation_y = 0.0;
    game_player.rotation_z = 0.0;

    // Set initial game state
    game_player.health = 100;
    game_player.is_alive = true;
    game_player.team = team;

    // Initialize stats
    game_player.kills = 0;
    game_player.deaths = 0;
    game_player.score = 0;

    game_player.last_update = clock.unix_timestamp;
    game_player.bump = ctx.bumps.game_player;

    msg!(
        "Initialized game player {} for game {} at position ({}, {}, {})",
        ctx.accounts.authority.key(),
        game_id,
        spawn_x,
        spawn_y,
        spawn_z
    );

    Ok(())
}

#[derive(Accounts)]
#[instruction(game_id: Pubkey)]
pub struct InitGamePlayer<'info> {
    #[account(
        init,
        payer = authority,
        space = GamePlayer::SIZE,
        seeds = [
            b"game_player",
            authority.key().as_ref(),
            game_id.as_ref()
        ],
        bump
    )]
    pub game_player: Account<'info, GamePlayer>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}
