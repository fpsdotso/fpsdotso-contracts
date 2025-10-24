
use anchor_lang::prelude::*;
use crate::error::StartGameError;
use crate::constants::{PLAYER_SEED, MIN_PLAYERS_TO_START};

pub fn handler(ctx: Context<StartGame>) -> Result<()> {
    let game = &mut ctx.accounts.game;
    let player = &ctx.accounts.player;
    let clock = Clock::get()?;
    
    require!(game.game_state == 0, StartGameError::GameAlreadyStarted);

    let total_players = game.current_players_team_a + game.current_players_team_b;
    require!(total_players >= MIN_PLAYERS_TO_START, StartGameError::NotEnoughPlayers);

    let is_lobby_owner = player.key() == game.created_by;
    let all_ready = game.ready_players >= total_players;

    require!(is_lobby_owner || all_ready, StartGameError::CannotStartGame);

    game.game_state = 1;
    game.match_start_timestamp = clock.unix_timestamp;
    
    Ok(())
}

#[derive(Accounts)]
pub struct StartGame<'info> {
    #[account(mut)]
    pub game: Account<'info, crate::state::Game>,
    
    #[account(
        seeds = [PLAYER_SEED.as_bytes(), authority.key().as_ref()],
        bump
    )]
    pub player: Account<'info, crate::state::Player>,
    
    pub authority: Signer<'info>,
}