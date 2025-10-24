
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

    // Check if the player leaving is the room owner
    let is_owner = game.created_by == ctx.accounts.authority.key();

    // Remove player from their team's vector
    let player_key = player.key();
    if player.team == 1 {
        // Remove from team A
        if let Some(pos) = game.team_a_players.iter().position(|&x| x == player_key) {
            game.team_a_players.remove(pos);
        }
        game.current_players_team_a = game.current_players_team_a.saturating_sub(1);
    } else if player.team == 2 {
        // Remove from team B
        if let Some(pos) = game.team_b_players.iter().position(|&x| x == player_key) {
            game.team_b_players.remove(pos);
        }
        game.current_players_team_b = game.current_players_team_b.saturating_sub(1);
    }

    player.is_alive = false;
    player.team = 0;
    player.current_game = None;

    let total_players = game.current_players_team_a.checked_add(game.current_players_team_b)
        .ok_or(LeaveGameError::ArithmeticOverflow)?;

    // If owner leaves, close the game account (room gets deleted)
    if is_owner {
        // Properly close the account by transferring lamports to authority
        let game_info = game.to_account_info();
        let authority_info = ctx.accounts.authority.to_account_info();

        let game_lamports = game_info.lamports();
        let authority_lamports = authority_info.lamports();

        // Transfer lamports from game to authority
        **authority_info.try_borrow_mut_lamports()? = authority_lamports
            .checked_add(game_lamports)
            .ok_or(LeaveGameError::ArithmeticOverflow)?;

        // Close the account by zeroing lamports - this is intentional for account closure
        #[allow(clippy::manual_lamports_zeroing)]
        {
            **game_info.try_borrow_mut_lamports()? = 0;
        }
    } else if total_players == 0 && game.game_state == 1 {
        // If all players left during an active game, end the game
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