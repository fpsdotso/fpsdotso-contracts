use anchor_lang::prelude::*;
use crate::error::InitGameError;
use crate::constants::{GAME_SEED, PLAYER_SEED, MAX_PLAYERS_PER_TEAM};

pub fn handler(ctx: Context<InitGame>, map_id: String) -> Result<()> {
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
    game.team_a_kills = 0;
    game.team_b_kills = 0;
    game.current_players_team_a = 0;
    game.current_players_team_b = 0;
    game.match_duration = 300; // 5 minutes
    game.max_players_per_team = MAX_PLAYERS_PER_TEAM;
    game.match_type = 1; // team deathmatch
    game.map_id = map_id;
    
    // NEW: Initialize lobby features
    game.lobby_name = "New Game Room".to_string();
    game.created_by = ctx.accounts.authority.key();
    game.is_private = false;
    game.ready_players = 0;
    game.map_selection = 0;

    // Initialize player tracking vectors
    game.team_a_players = Vec::new();
    game.team_b_players = Vec::new();

    // Add the room creator as the first player in Team A
    let game_key = game.key();
    let player_key = player.key();

    player.team = 1; // Assign to Team A
    player.is_alive = true;
    player.is_ready = false;
    player.current_game = Some(game_key);

    // Add creator to Team A players
    game.team_a_players.push(player_key);
    game.current_players_team_a = 1;

    // Increment the game counter for this player
    player.game_counter += 1;

    Ok(())
}

#[derive(Accounts)]
pub struct InitGame<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + // discriminator
                4 + 4 + 4 + 4 + 4 + 8 + 1 + 8 + 1 + 1 + 1 + 1 + 1 + // basic game fields (scores, kills, duration, timestamps, state, team counts, winning_team, match_type)
                (4 + 50) + // map_id String (4 byte length + up to 50 chars)
                (4 + 32) + // lobby_name string with length prefix
                32 + // created_by pubkey
                1 + 1 + 1 + // is_private + ready_players + map_selection
                (4 + 32 * 5) + (4 + 32 * 5), // team_a_players Vec (4 byte length + max 5 pubkeys) + team_b_players Vec
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
