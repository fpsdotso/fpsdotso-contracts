use anchor_lang::prelude::*;
use ephemeral_rollups_sdk::anchor::{commit, ephemeral};
use ephemeral_rollups_sdk::ephem::commit_accounts;

declare_id!("4pfYuQkFmGXPFMjBNmYUstnC3jjgjxcBS8rSk8qcUUnE");

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
        rotation_x: f32,
        rotation_y: f32,
        rotation_z: f32,
        delta_time: f32,
        _game_id: Pubkey,
    ) -> Result<()> {
        let player = &mut ctx.accounts.game_player;
        let clock = Clock::get()?;

        // Store rotation values directly from frontend
        // Frontend calculates rotation, contract just stores it
        player.rotation_x = rotation_x;
        player.rotation_y = rotation_y;
        player.rotation_z = rotation_z;

        // Process movement based on updated rotation
        // Coordinate system: +X=right, +Y=up, +Z=forward
        // Yaw (rotation_y) rotates around Y axis
        // Only yaw affects movement direction (FPS style - pitch is for aiming only)
        let move_speed = 5.0;
        let movement = move_speed * delta_time;

        let yaw = player.rotation_y;

        // Calculate movement directions relative to camera rotation on horizontal plane
        // Raylib convention: rotation_y = 0° → looking at +X, rotation_y = 90° → looking at +Z
        // Forward direction: move in the direction camera is facing
        let forward_x = yaw.cos();
        let forward_z = yaw.sin();

        // Right direction: 90 degrees clockwise from forward (when viewed from above)
        let right_x = -yaw.sin();
        let right_z = yaw.cos();

        // Apply movement based on input
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
    rotation_x: f32,
    rotation_y: f32,
    rotation_z: f32,
    delta_time: f32,
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
