use anchor_lang::prelude::*;
use crate::error::JoinGameError;
use crate::constants::{PLAYER_SEED, MAX_TOTAL_PLAYERS};

pub fn handler(ctx: Context<JoinGame>) -> Result<()> {
    let player = &mut ctx.accounts.player;
    let game = &mut ctx.accounts.game;
    
    require!(player.has_logged_in, JoinGameError::PlayerNotRegistered);
    require!(player.current_game.is_none(), JoinGameError::PlayerAlreadyInGame);
    require!(game.game_state == 0, JoinGameError::GameAlreadyStarted);
    
    // Check if game has space
    let total_players = game.current_players_team_a + game.current_players_team_b;
    require!(total_players < MAX_TOTAL_PLAYERS, JoinGameError::GameFull);
    
    player.is_alive = true;
    player.team = if game.current_players_team_a <= game.current_players_team_b { 1 } else { 2 };
    player.current_game = Some(game.key());
    player.is_ready = false; // Reset ready state when joining
    
    if player.team == 1 {
        game.current_players_team_a += 1;
    } else {
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