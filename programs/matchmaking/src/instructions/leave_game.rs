
use anchor_lang::prelude::*;
use crate::error::LeaveGameError;
use crate::constants::PLAYER_SEED;

pub fn handler(ctx: Context<LeaveGame>) -> Result<()> {
    let game = &mut ctx.accounts.game;
    let player = &mut ctx.accounts.player;
    
    require!(player.has_logged_in, LeaveGameError::PlayerNotInGame);
    require!(player.current_game.is_some(), LeaveGameError::PlayerNotInGame);
    
    // Verify player is in this specific game
    require!(player.current_game.unwrap() == game.key(), LeaveGameError::PlayerNotInThisGame);
    
    if player.team == 1 {
        game.current_players_team_a = game.current_players_team_a.saturating_sub(1);
    } else if player.team == 2 {
        game.current_players_team_b = game.current_players_team_b.saturating_sub(1);
    }
    
    player.is_alive = false;
    player.team = 0;
    player.current_game = None;
    
    let total_players = game.current_players_team_a + game.current_players_team_b;
    if total_players == 0 && game.game_state == 1 {
        game.game_state = 2;
        game.match_end_timestamp = Some(Clock::get()?.unix_timestamp);
    }
    
    Ok(())
}

#[derive(Accounts)]
pub struct LeaveGame<'info> {
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