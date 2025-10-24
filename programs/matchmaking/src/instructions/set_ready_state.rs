use anchor_lang::prelude::*;
use crate::error::SetReadyStateError;
use crate::constants::PLAYER_SEED;
use crate::state::Player;
use ephemeral_rollups_sdk::anchor::delegate;
use ephemeral_rollups_sdk::cpi::DelegateConfig;

pub fn handler(ctx: Context<SetReadyState>, is_ready: bool) -> Result<()> {
    // Manually deserialize the player account from AccountInfo
    let player_data = ctx.accounts.pda.try_borrow_data()?;
    let mut player: Player = Player::try_deserialize(&mut &player_data[..])?;
    drop(player_data); // Release the borrow

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

    // Update the game's ready player count and handle delegation
    if is_ready && !was_ready {
        // Player just became ready - update state BEFORE delegation
        player.is_ready = true;
        ctx.accounts.game.ready_players = ctx.accounts.game.ready_players.saturating_add(1);

        // Serialize player back to account data
        let mut player_data = ctx.accounts.pda.try_borrow_mut_data()?;
        player.try_serialize(&mut &mut player_data[..])?;
        drop(player_data); // Release the borrow before delegation

        let authority_key = ctx.accounts.authority.key();
        let seeds = &[PLAYER_SEED.as_bytes(), authority_key.as_ref()];
        let player_key = ctx.accounts.pda.key();
        let ready_count = ctx.accounts.game.ready_players;

        // Get validator from remaining accounts if provided
        let validator = if !ctx.remaining_accounts.is_empty() {
            ctx.remaining_accounts.first().map(|acc| acc.key())
        } else {
            None
        };

        // Delegate player account to ephemeral rollup
        // After this point, we CANNOT write to the player account on mainnet
        ctx.accounts.delegate_pda(
            &ctx.accounts.signer,
            seeds,
            DelegateConfig {
                validator,
                commit_frequency_ms: 5000,
                ..Default::default()
            },
        )?;

        if let Some(validator_key) = validator {
            msg!("Player {} delegated to validator {} and is now ready. Total ready: {}",
                player_key, validator_key, ready_count);
        } else {
            msg!("Player {} delegated (no specific validator) and is now ready. Total ready: {}",
                player_key, ready_count);
        }

    } else if !is_ready && was_ready {
        // Player is no longer ready - update state
        // Note: Account should be undelegated first via undelegate_player instruction
        player.is_ready = false;
        ctx.accounts.game.ready_players = ctx.accounts.game.ready_players.saturating_sub(1);

        // Serialize player back to account data
        let mut player_data = ctx.accounts.pda.try_borrow_mut_data()?;
        player.try_serialize(&mut &mut player_data[..])?;

        msg!("Player {} is no longer ready. Total ready: {}", ctx.accounts.pda.key(), ctx.accounts.game.ready_players);
    } else if is_ready && was_ready {
        // Player is already ready - do nothing (already delegated)
        msg!("Player {} is already ready", ctx.accounts.pda.key());
    } else {
        // Player is already not ready
        msg!("Player {} is already not ready", ctx.accounts.pda.key());
    }

    Ok(())
}

/// Delegate player account to ephemeral rollup
#[delegate]
#[derive(Accounts)]
pub struct SetReadyState<'info> {
    #[account(mut)]
    pub game: Account<'info, crate::state::Game>,

    /// CHECK: This is the player PDA to be delegated to ephemeral rollup.
    /// It's validated by seeds and manually deserialized in the handler.
    #[account(
        mut,
        del
    )]
    pub pda: AccountInfo<'info>,

    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(mut)]
    pub signer: Signer<'info>
}
