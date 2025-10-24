use anchor_lang::prelude::*;
use crate::error::InitGameError;
use crate::constants::{GAME_SEED, PLAYER_SEED, MAX_PLAYERS_PER_TEAM};

pub fn handler(ctx: Context<InitGame>) -> Result<()> {
    let game = &mut ctx.accounts.game;
    let player = &mut ctx.accounts.player;
    let clock = Clock::get()?;
    
    require!(player.has_logged_in, InitGameError::PlayerNotRegistered);
    require!(player.current_game.is_none(), InitGameError::PlayerAlreadyInGame);
    
    // Initialize game state - the game account itself is the PDA that tracks the room
    game.match_start_timestamp = clock.unix_timestamp;
    game.game_state = 0; // waiting state
    game.team_a_score = 0;
    game.team_b_score = 0;
    game.current_players_team_a = 0;
    game.current_players_team_b = 0;
    game.match_duration = 300; // 5 minutes
    game.max_players_per_team = MAX_PLAYERS_PER_TEAM;
    game.match_type = 1; // team deathmatch
    game.map_name = "New Arena".to_string();
    
    // NEW: Initialize lobby features
    game.lobby_name = "New Game Room".to_string();
    game.created_by = ctx.accounts.authority.key();
    game.is_private = false;
    game.ready_players = 0;
    game.map_selection = 0;
    
    // Set player as the first player in the game
    player.team = 0; // No team assigned yet
    // Set the current game PDA for the player to track this room
    player.current_game = Some(ctx.accounts.game.key());
    
    // Increment the game counter for this player
    player.game_counter += 1;

    Ok(())
}

#[derive(Accounts)]
pub struct InitGame<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + 4 + 4 + 4 + 8 + 1 + 8 + 1 + 1 + 1 + 1 + 1 + 4 + 50 + 32 + 32 + 1 + 1 + 1, // discriminator + all fields
        seeds = [GAME_SEED.as_bytes(), authority.key().as_ref(), &player.game_counter.to_le_bytes()],
        bump
    )]
    pub game: Account<'info, crate::state::Game>,
    
    #[account(
        mut,
        seeds = [PLAYER_SEED.as_bytes(), authority.key().as_ref()],
        bump
    )]
    pub player: Account<'info, crate::state::Player>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}
