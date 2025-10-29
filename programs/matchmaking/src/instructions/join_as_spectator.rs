use anchor_lang::prelude::*;
use crate::error::JoinGameError;
use crate::state::{Player, Game};
use crate::constants::PLAYER_SEED;

pub fn handler(ctx: Context<JoinAsSpectator>) -> Result<()> {
    let player = &mut ctx.accounts.player;
    let game = &mut ctx.accounts.game;

    require!(player.has_logged_in, JoinGameError::PlayerNotRegistered);
    require!(player.current_game.is_none(), JoinGameError::PlayerAlreadyInGame);

    // Can join as spectator even if game started
    require!(game.game_state <= 1, JoinGameError::InvalidGameState); // 0=waiting, 1=active

    // Set player as spectator
    player.current_game = Some(game.key());
    player.is_spectator = true;
    player.is_alive = false; // Spectators are not alive in-game
    player.team = 0; // No team for spectators

    msg!(
        "Player {} joined game {} as spectator",
        player.authority,
        game.key()
    );

    Ok(())
}

#[derive(Accounts)]
pub struct JoinAsSpectator<'info> {
    #[account(
        mut,
        seeds = [PLAYER_SEED.as_bytes(), authority.key().as_ref()],
        bump
    )]
    pub player: Account<'info, Player>,

    #[account(mut)]
    pub game: Account<'info, Game>,

    #[account(mut)]
    pub authority: Signer<'info>,
}
