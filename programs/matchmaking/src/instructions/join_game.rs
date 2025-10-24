use anchor_lang::prelude::*;
use crate::error::JoinGameError;
use crate::constants::{PLAYER_SEED, MAX_TOTAL_PLAYERS, MAX_PLAYERS_PER_TEAM};

pub fn handler(ctx: Context<JoinGame>) -> Result<()> {
    let player = &mut ctx.accounts.player;
    let game = &mut ctx.accounts.game;

    require!(player.has_logged_in, JoinGameError::PlayerNotRegistered);
    require!(player.current_game.is_none(), JoinGameError::PlayerAlreadyInGame);
    require!(game.game_state == 0, JoinGameError::GameAlreadyStarted);

    // Check if game has space
    let total_players = game.current_players_team_a + game.current_players_team_b;
    require!(total_players < MAX_TOTAL_PLAYERS, JoinGameError::GameFull);

    // Determine which team to join (balance teams)
    let team = if game.current_players_team_a <= game.current_players_team_b { 1 } else { 2 };

    // Check if the selected team has space
    if team == 1 {
        require!(
            game.team_a_players.len() < MAX_PLAYERS_PER_TEAM as usize,
            JoinGameError::TeamFull
        );
    } else {
        require!(
            game.team_b_players.len() < MAX_PLAYERS_PER_TEAM as usize,
            JoinGameError::TeamFull
        );
    }

    player.is_alive = true;
    player.team = team;
    player.current_game = Some(game.key());
    player.is_ready = false; // Reset ready state when joining

    // Add player PDA to the appropriate team vector
    if player.team == 1 {
        game.team_a_players.push(player.key());
        game.current_players_team_a += 1;
    } else {
        game.team_b_players.push(player.key());
        game.current_players_team_b += 1;
    }

    Ok(())
}

#[derive(Accounts)]
pub struct JoinGame<'info> {
    #[account(mut)]
    pub game: Account<'info, crate::state::Game>,
    
    #[account(
        mut,
        seeds = [PLAYER_SEED.as_bytes(), authority.key().as_ref()],
        bump
    )]
    pub player: Account<'info, crate::state::Player>,
    
    pub authority: Signer<'info>,
}