use anchor_lang::prelude::*;
use crate::error::SetReadyStateError;
use crate::constants::PLAYER_SEED;

pub fn handler(ctx: Context<SetReadyState>, is_ready: bool) -> Result<()> {
    let player = &mut ctx.accounts.player;
    let game = &mut ctx.accounts.game;

    require!(player.has_logged_in, SetReadyStateError::PlayerNotRegistered);
    require!(player.current_game.is_some(), SetReadyStateError::PlayerNotInGame);

    // Verify player is in this specific game
    require!(
        player.current_game.unwrap() == game.key(),
        SetReadyStateError::PlayerNotInThisGame
    );

    // Can only change ready state in waiting/lobby state
    require!(
        game.game_state == 0,
        SetReadyStateError::GameAlreadyStarted
    );

    let was_ready = player.is_ready;
    player.is_ready = is_ready;

    // Update the game's ready player count
    if is_ready && !was_ready {
        // Player just became ready
        game.ready_players = game.ready_players.saturating_add(1);
    } else if !is_ready && was_ready {
        // Player is no longer ready
        game.ready_players = game.ready_players.saturating_sub(1);
    }

    Ok(())
}

#[derive(Accounts)]
pub struct SetReadyState<'info> {
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
