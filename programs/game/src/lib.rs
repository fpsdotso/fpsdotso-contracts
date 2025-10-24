use anchor_lang::prelude::*;
use ephemeral_rollups_sdk::anchor::{commit, ephemeral};
use ephemeral_rollups_sdk::ephem::commit_accounts;
use std::io::Write as IoWrite;

declare_id!("7TE8ZZqRyFMR7K3ocVCL2hBNfW29cH57pMTvj1qTj8cX");

pub const PLAYER_SEED: &[u8] = b"player";

// Offset to position/rotation fields in Player account from matchmaking program
// discriminator(8) + authority(32) + username(4+32) + has_logged_in(1) + team(1) +
// current_game(1+32) + is_alive(1) + last_login_timestamp(8) + total_matches_played(4) +
// level(4) + is_ready(1) + game_counter(4) = 133 bytes before position fields
const POSITION_ROTATION_OFFSET: usize = 133;

// Allow floating point arithmetic - safe for game logic on f32 values
#[allow(clippy::arithmetic_side_effects)]
#[ephemeral]
#[program]
pub mod game {
    use super::*;

    /// Combined input processing - movement and rotation in one call
    /// This is the main function for real-time gameplay
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
    ) -> Result<()> {
        let player_account = &ctx.accounts.player;
        let mut data = player_account.data.borrow_mut();

        // Read current position and rotation
        let mut pos_x = read_f32(&data, POSITION_ROTATION_OFFSET);
        let mut pos_y = read_f32(&data, POSITION_ROTATION_OFFSET + 4);
        let mut pos_z = read_f32(&data, POSITION_ROTATION_OFFSET + 8);
        let mut rot_x = read_f32(&data, POSITION_ROTATION_OFFSET + 12);
        let mut rot_y = read_f32(&data, POSITION_ROTATION_OFFSET + 16);
        let rot_z = read_f32(&data, POSITION_ROTATION_OFFSET + 20);

        // Process rotation first
        let rotation_x_delta = delta_y * sensitivity;
        let rotation_y_delta = delta_x * sensitivity;

        rot_y += rotation_y_delta;
        rot_x += rotation_x_delta;

        // Clamp pitch to prevent camera flipping
        const MAX_PITCH: f32 = std::f32::consts::FRAC_PI_2 - 0.1; // 89 degrees
        rot_x = rot_x.clamp(-MAX_PITCH, MAX_PITCH);

        // Normalize yaw to 0-2PI range
        const TWO_PI: f32 = std::f32::consts::PI * 2.0;
        if rot_y > TWO_PI {
            rot_y -= TWO_PI;
        } else if rot_y < 0.0 {
            rot_y += TWO_PI;
        }

        // Process movement based on updated rotation
        let move_speed = 5.0;
        let movement = move_speed * delta_time;

        let yaw = rot_y;
        let forward_x = yaw.sin();
        let forward_z = yaw.cos();
        const HALF_PI: f32 = std::f32::consts::FRAC_PI_2;
        let right_x = (yaw + HALF_PI).sin();
        let right_z = (yaw + HALF_PI).cos();

        if forward {
            pos_x += forward_x * movement;
            pos_z += forward_z * movement;
        }
        if backward {
            pos_x -= forward_x * movement;
            pos_z -= forward_z * movement;
        }
        if left {
            pos_x -= right_x * movement;
            pos_z -= right_z * movement;
        }
        if right {
            pos_x += right_x * movement;
            pos_z += right_z * movement;
        }

        // Write updated values back
        write_f32(&mut data, POSITION_ROTATION_OFFSET, pos_x);
        write_f32(&mut data, POSITION_ROTATION_OFFSET + 4, pos_y);
        write_f32(&mut data, POSITION_ROTATION_OFFSET + 8, pos_z);
        write_f32(&mut data, POSITION_ROTATION_OFFSET + 12, rot_x);
        write_f32(&mut data, POSITION_ROTATION_OFFSET + 16, rot_y);
        write_f32(&mut data, POSITION_ROTATION_OFFSET + 20, rot_z);

        msg!(
            "Player {} - Pos: ({:.2}, {:.2}, {:.2}), Rot: ({:.2}, {:.2}, {:.2})",
            player_account.key(),
            pos_x,
            pos_y,
            pos_z,
            rot_x,
            rot_y,
            rot_z
        );

        Ok(())
    }

    /// Separate movement processing for when you only need to update position
    pub fn process_movement(
        ctx: Context<ProcessPlayerInput>,
        forward: bool,
        backward: bool,
        left: bool,
        right: bool,
        delta_time: f32,
    ) -> Result<()> {
        let player_account = &ctx.accounts.player;
        let mut data = player_account.data.borrow_mut();

        let mut pos_x = read_f32(&data, POSITION_ROTATION_OFFSET);
        let pos_y = read_f32(&data, POSITION_ROTATION_OFFSET + 4);
        let mut pos_z = read_f32(&data, POSITION_ROTATION_OFFSET + 8);
        let rot_y = read_f32(&data, POSITION_ROTATION_OFFSET + 16);

        let move_speed = 5.0;
        let movement = move_speed * delta_time;

        let yaw = rot_y;
        let forward_x = yaw.sin();
        let forward_z = yaw.cos();
        const HALF_PI: f32 = std::f32::consts::FRAC_PI_2;
        let right_x = (yaw + HALF_PI).sin();
        let right_z = (yaw + HALF_PI).cos();

        if forward {
            pos_x += forward_x * movement;
            pos_z += forward_z * movement;
        }
        if backward {
            pos_x -= forward_x * movement;
            pos_z -= forward_z * movement;
        }
        if left {
            pos_x -= right_x * movement;
            pos_z -= right_z * movement;
        }
        if right {
            pos_x += right_x * movement;
            pos_z += right_z * movement;
        }

        write_f32(&mut data, POSITION_ROTATION_OFFSET, pos_x);
        write_f32(&mut data, POSITION_ROTATION_OFFSET + 8, pos_z);

        msg!(
            "Player {} moved to: ({:.2}, {:.2}, {:.2})",
            player_account.key(),
            pos_x,
            pos_y,
            pos_z
        );

        Ok(())
    }

    /// Separate rotation processing for camera updates
    pub fn process_rotation(
        ctx: Context<ProcessPlayerInput>,
        delta_x: f32,
        delta_y: f32,
        sensitivity: f32,
    ) -> Result<()> {
        let player_account = &ctx.accounts.player;
        let mut data = player_account.data.borrow_mut();

        let mut rot_x = read_f32(&data, POSITION_ROTATION_OFFSET + 12);
        let mut rot_y = read_f32(&data, POSITION_ROTATION_OFFSET + 16);

        let rotation_x_delta = delta_y * sensitivity;
        let rotation_y_delta = delta_x * sensitivity;

        rot_y += rotation_y_delta;
        rot_x += rotation_x_delta;

        const MAX_PITCH: f32 = std::f32::consts::FRAC_PI_2 - 0.1;
        rot_x = rot_x.clamp(-MAX_PITCH, MAX_PITCH);

        const TWO_PI: f32 = std::f32::consts::PI * 2.0;
        if rot_y > TWO_PI {
            rot_y -= TWO_PI;
        } else if rot_y < 0.0 {
            rot_y += TWO_PI;
        }

        write_f32(&mut data, POSITION_ROTATION_OFFSET + 12, rot_x);
        write_f32(&mut data, POSITION_ROTATION_OFFSET + 16, rot_y);

        msg!(
            "Player {} rotation: pitch={:.2}, yaw={:.2}",
            player_account.key(),
            rot_x,
            rot_y
        );

        Ok(())
    }

    /// Manual commit player state back to mainnet
    pub fn commit_player_state(ctx: Context<CommitPlayerState>) -> Result<()> {
        msg!("Committing player {} state to mainnet", ctx.accounts.player.key());

        commit_accounts(
            &ctx.accounts.payer,
            vec![&ctx.accounts.player],
            &ctx.accounts.magic_context,
            &ctx.accounts.magic_program,
        )?;

        Ok(())
    }

    /// Get player state for debugging
    pub fn get_player_state(ctx: Context<GetPlayerState>) -> Result<PlayerStateData> {
        let player_account = &ctx.accounts.player;
        let data = player_account.data.borrow();

        let pos_x = read_f32(&data, POSITION_ROTATION_OFFSET);
        let pos_y = read_f32(&data, POSITION_ROTATION_OFFSET + 4);
        let pos_z = read_f32(&data, POSITION_ROTATION_OFFSET + 8);
        let rot_x = read_f32(&data, POSITION_ROTATION_OFFSET + 12);
        let rot_y = read_f32(&data, POSITION_ROTATION_OFFSET + 16);
        let rot_z = read_f32(&data, POSITION_ROTATION_OFFSET + 20);

        msg!(
            "Player {} - Pos: ({:.2}, {:.2}, {:.2}), Rot: ({:.2}, {:.2}, {:.2})",
            player_account.key(),
            pos_x,
            pos_y,
            pos_z,
            rot_x,
            rot_y,
            rot_z
        );

        Ok(PlayerStateData {
            position_x: pos_x,
            position_y: pos_y,
            position_z: pos_z,
            rotation_x: rot_x,
            rotation_y: rot_y,
            rotation_z: rot_z,
        })
    }
}

// Helper functions for reading/writing f32 from account data
fn read_f32(data: &[u8], offset: usize) -> f32 {
    f32::from_le_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
    ])
}

fn write_f32(data: &mut [u8], offset: usize, value: f32) {
    let bytes = value.to_le_bytes();
    data[offset..offset + 4].copy_from_slice(&bytes);
}

#[derive(Accounts)]
pub struct ProcessPlayerInput<'info> {
    /// CHECK: Player account from matchmaking program, delegated to ephemeral rollup
    #[account(mut)]
    pub player: AccountInfo<'info>,
}

#[commit]
#[derive(Accounts)]
pub struct CommitPlayerState<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    /// CHECK: Player account to commit
    #[account(mut)]
    pub player: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct GetPlayerState<'info> {
    /// CHECK: Player account
    pub player: AccountInfo<'info>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct PlayerStateData {
    pub position_x: f32,
    pub position_y: f32,
    pub position_z: f32,
    pub rotation_x: f32,
    pub rotation_y: f32,
    pub rotation_z: f32,
}
