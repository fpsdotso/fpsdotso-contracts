use anchor_lang::prelude::*;
use ephemeral_rollups_sdk::anchor::{commit, ephemeral};
use ephemeral_rollups_sdk::ephem::commit_accounts;

declare_id!("7TE8ZZqRyFMR7K3ocVCL2hBNfW29cH57pMTvj1qTj8cX");

pub const PLAYER_SEED: &[u8] = b"player";

mod instructions;
mod state;

use instructions::*;
use state::*;

// Allow floating point arithmetic - safe for game logic on f32 values
#[allow(clippy::arithmetic_side_effects)]
#[ephemeral]
#[program]
pub mod game {
    use super::*;

    /// Initialize game player (called after matchmaking ready)
    pub fn init_game_player(
        ctx: Context<InitGamePlayer>,
        game_id: Pubkey,
        team: u8,
        spawn_x: f32,
        spawn_y: f32,
        spawn_z: f32,
    ) -> Result<()> {
        init_game_player::handler(ctx, game_id, team, spawn_x, spawn_y, spawn_z)
    }

    /// Combined input processing - movement and rotation in one call
    /// This is the main function for real-time gameplay
    #[allow(clippy::arithmetic_side_effects)]
    #[allow(clippy::float_arithmetic)]
    pub fn process_input(
        ctx: Context<ProcessPlayerInput>,
        forward: bool,
        backward: bool,
        left: bool,
        right: bool,
        delta_x: f32,
        delta_y: f32,
        delta_time: f32,
        sensitivity: f32,
        _game_id: Pubkey,
    ) -> Result<()> {
        let player = &mut ctx.accounts.game_player;
        let clock = Clock::get()?;

        // Process rotation first
        let rotation_x_delta = delta_y * sensitivity;
        let rotation_y_delta = delta_x * sensitivity;

        player.rotation_y += rotation_y_delta;
        player.rotation_x += rotation_x_delta;

        // Clamp pitch to prevent camera flipping
        const MAX_PITCH: f32 = std::f32::consts::FRAC_PI_2 - 0.1; // 89 degrees
        player.rotation_x = player.rotation_x.clamp(-MAX_PITCH, MAX_PITCH);

        // Normalize yaw to 0-2PI range
        const TWO_PI: f32 = std::f32::consts::PI * 2.0;
        if player.rotation_y > TWO_PI {
            player.rotation_y -= TWO_PI;
        } else if player.rotation_y < 0.0 {
            player.rotation_y += TWO_PI;
        }

        // Process movement based on updated rotation
        let move_speed = 5.0;
        let movement = move_speed * delta_time;

        let yaw = player.rotation_y;
        let forward_x = yaw.sin();
        let forward_z = yaw.cos();
        const HALF_PI: f32 = std::f32::consts::FRAC_PI_2;
        let right_x = (yaw + HALF_PI).sin();
        let right_z = (yaw + HALF_PI).cos();

        if forward {
            player.position_x += forward_x * movement;
            player.position_z += forward_z * movement;
        }
        if backward {
            player.position_x -= forward_x * movement;
            player.position_z -= forward_z * movement;
        }
        if left {
            player.position_x -= right_x * movement;
            player.position_z -= right_z * movement;
        }
        if right {
            player.position_x += right_x * movement;
            player.position_z += right_z * movement;
        }

        player.last_update = clock.unix_timestamp;

        msg!(
            "Player {} - Pos: ({:.2}, {:.2}, {:.2}), Rot: ({:.2}, {:.2}, {:.2})",
            ctx.accounts.authority.key(),
            player.position_x,
            player.position_y,
            player.position_z,
            player.rotation_x,
            player.rotation_y,
            player.rotation_z
        );

        Ok(())
    }

    /// Manual commit player state back to mainnet
    pub fn commit_player_state(ctx: Context<CommitPlayerState>) -> Result<()> {
        msg!("Committing game player {} state to mainnet", ctx.accounts.game_player.key());

        commit_accounts(
            &ctx.accounts.payer,
            vec![&ctx.accounts.game_player.to_account_info()],
            &ctx.accounts.magic_context,
            &ctx.accounts.magic_program,
        )?;

        Ok(())
    }

    /// Get player state for debugging
    pub fn get_player_state(ctx: Context<GetPlayerState>) -> Result<PlayerStateData> {
        let player = &ctx.accounts.game_player;

        msg!(
            "Player {} - Pos: ({:.2}, {:.2}, {:.2}), Rot: ({:.2}, {:.2}, {:.2})",
            player.authority,
            player.position_x,
            player.position_y,
            player.position_z,
            player.rotation_x,
            player.rotation_y,
            player.rotation_z
        );

        Ok(PlayerStateData {
            position_x: player.position_x,
            position_y: player.position_y,
            position_z: player.position_z,
            rotation_x: player.rotation_x,
            rotation_y: player.rotation_y,
            rotation_z: player.rotation_z,
            health: player.health,
            is_alive: player.is_alive,
            team: player.team,
            kills: player.kills,
            deaths: player.deaths,
            score: player.score,
        })
    }

    /// Delegate GamePlayer account to game ephemeral rollup
    /// This should be called AFTER init_game_player
    pub fn delegate_game_player(ctx: Context<DelegateGamePlayer>, game_id: Pubkey) -> Result<()> {
        delegate_game_player::handler(ctx, game_id)
    }

    /// Undelegate GamePlayer account from game ephemeral rollup
    /// This should be called when game ends
    pub fn undelegate_game_player(ctx: Context<UndelegateGamePlayer>) -> Result<()> {
        undelegate_game_player::handler(ctx)
    }
}

#[derive(Accounts)]
#[instruction(
    forward: bool,
    backward: bool,
    left: bool,
    right: bool,
    delta_x: f32,
    delta_y: f32,
    delta_time: f32,
    sensitivity: f32,
    _game_id: Pubkey
)]
pub struct ProcessPlayerInput<'info> {
    #[account(
        mut,
        seeds = [
            b"game_player",
            authority.key().as_ref(),
            _game_id.as_ref()
        ],
        bump = game_player.bump
    )]
    pub game_player: Account<'info, GamePlayer>,

    pub authority: Signer<'info>,
}

#[commit]
#[derive(Accounts)]
pub struct CommitPlayerState<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(mut)]
    pub game_player: Account<'info, GamePlayer>,
}

#[derive(Accounts)]
pub struct GetPlayerState<'info> {
    pub game_player: Account<'info, GamePlayer>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct PlayerStateData {
    pub position_x: f32,
    pub position_y: f32,
    pub position_z: f32,
    pub rotation_x: f32,
    pub rotation_y: f32,
    pub rotation_z: f32,
    pub health: u8,
    pub is_alive: bool,
    pub team: u8,
    pub kills: u32,
    pub deaths: u32,
    pub score: u32,
}
