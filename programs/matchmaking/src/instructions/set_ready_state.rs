use anchor_lang::prelude::*;
use crate::error::SetReadyStateError;
use crate::state::Player;

pub fn handler(ctx: Context<SetReadyState>, is_ready: bool) -> Result<()> {
    let player = &mut ctx.accounts.player;

    require!(player.has_logged_in, SetReadyStateError::PlayerNotRegistered);
    require!(player.current_game.is_some(), SetReadyStateError::PlayerNotInGame);

    // Verify player is in this specific game
    require!(
        player.current_game.unwrap() == ctx.accounts.game.key(),
        SetReadyStateError::PlayerNotInThisGame
    );

    // Can only change ready state in waiting/lobby state
    require!(
        ctx.accounts.game.game_state == 0,
        SetReadyStateError::GameAlreadyStarted
    );

    let was_ready = player.is_ready;

    // Update the game's ready player count
    if is_ready && !was_ready {
        player.is_ready = true;
        ctx.accounts.game.ready_players = ctx.accounts.game.ready_players.saturating_add(1);
        msg!("Player {} is now ready. Total ready: {}", player.key(), ctx.accounts.game.ready_players);
    } else if !is_ready && was_ready {
        player.is_ready = false;
        ctx.accounts.game.ready_players = ctx.accounts.game.ready_players.saturating_sub(1);
        msg!("Player {} is no longer ready. Total ready: {}", player.key(), ctx.accounts.game.ready_players);
    } else if is_ready && was_ready {
        msg!("Player {} is already ready", player.key());
    } else {
        msg!("Player {} is already not ready", player.key());
    }

    Ok(())
}

#[derive(Accounts)]
pub struct SetReadyState<'info> {
    #[account(mut)]
    pub game: Account<'info, crate::state::Game>,

    #[account(
        mut,
        seeds = [crate::constants::PLAYER_SEED.as_bytes(), authority.key().as_ref()],
        bump
    )]
    pub player: Account<'info, Player>,

    #[account(mut)]
    pub authority: Signer<'info>,
}
