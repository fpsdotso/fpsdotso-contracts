use anchor_lang::prelude::*;
use crate::state::GamePlayer;

#[error_code]
pub enum ShootError {
    #[msg("No bullets left. Reload your gun first.")]
    NoBulletsLeft,
}

/// Shoot and check if any player is hit
/// Uses ray-box intersection to detect hits
/// Automatically awards kill and score if target is killed
pub fn handler(ctx: Context<Shoot>, damage: u8, kill_score: u32) -> Result<()> {
    let shooter = &mut ctx.accounts.shooter;
    let clock = Clock::get()?;

    // Check if player has bullets
    require!(shooter.bullet_count > 0, ShootError::NoBulletsLeft);

    // Reduce bullet count
    shooter.bullet_count = shooter.bullet_count.saturating_sub(1);

    // Get shooter's position and rotation
    let origin_x = shooter.position_x;
    let origin_y = shooter.position_y + 1.0; // Eye level (middle of player height)
    let origin_z = shooter.position_z;

    // Calculate ray direction from rotation
    // rotation_x = pitch (up/down), rotation_y = yaw (left/right)
    let pitch = shooter.rotation_x;
    let yaw = shooter.rotation_y;

    // Ray direction in 3D space
    // Coordinate system: +X=right, +Y=up, +Z=forward
    // rotation_y = 0° → looking at +X
    let dir_x = yaw.cos() * pitch.cos();
    let dir_y = pitch.sin();
    let dir_z = yaw.sin() * pitch.cos();

    // Normalize direction vector
    let length = (dir_x * dir_x + dir_y * dir_y + dir_z * dir_z).sqrt();
    let dir_x = dir_x / length;
    let dir_y = dir_y / length;
    let dir_z = dir_z / length;

    msg!(
        "Player {} shooting from ({:.2}, {:.2}, {:.2}) dir ({:.2}, {:.2}, {:.2}) - Bullets left: {}",
        shooter.authority,
        origin_x,
        origin_y,
        origin_z,
        dir_x,
        dir_y,
        dir_z,
        shooter.bullet_count
    );

    // Check all other players for hits
    let mut closest_hit_distance = f32::MAX;
    let mut hit_player_index: Option<usize> = None;

    for (i, other_player_info) in ctx.remaining_accounts.iter().enumerate() {
        // Try to deserialize as GamePlayer
        let other_player_data = other_player_info.try_borrow_data()?;
        let other_player = GamePlayer::try_deserialize(&mut &other_player_data[..])?;

        // Skip if it's the shooter, dead, or spectator
        if other_player.authority == shooter.authority
            || !other_player.is_alive
            || other_player.is_spectator {
            continue;
        }

        // Skip if not on opposing team
        if other_player.team == shooter.team {
            continue;
        }

        // Player bounding box size: (1, 2, 1) centered on position
        let box_min_x = other_player.position_x - 0.5;
        let box_max_x = other_player.position_x + 0.5;
        let box_min_y = other_player.position_y;
        let box_max_y = other_player.position_y + 2.0;
        let box_min_z = other_player.position_z - 0.5;
        let box_max_z = other_player.position_z + 0.5;

        // Ray-box intersection test
        if let Some(hit_distance) = ray_box_intersection(
            origin_x, origin_y, origin_z,
            dir_x, dir_y, dir_z,
            box_min_x, box_min_y, box_min_z,
            box_max_x, box_max_y, box_max_z,
        ) {
            // Check if this is the closest hit
            if hit_distance < closest_hit_distance {
                closest_hit_distance = hit_distance;
                hit_player_index = Some(i);
            }
        }
    }

    // Apply damage to the hit player
    if let Some(hit_index) = hit_player_index {
        let hit_player_info = &ctx.remaining_accounts[hit_index];
        let mut hit_player_data = hit_player_info.try_borrow_mut_data()?;
        let mut hit_player = GamePlayer::try_deserialize(&mut &hit_player_data[..])?;

        // Reduce health
        let old_health = hit_player.health;
        hit_player.health = hit_player.health.saturating_sub(damage);

        // Check if player died
        if hit_player.health == 0 && old_health > 0 {
            hit_player.is_alive = false;
            hit_player.deaths = hit_player.deaths.saturating_add(1);
            hit_player.death_timestamp = clock.unix_timestamp;

            // Award kill and score to shooter
            shooter.kills = shooter.kills.saturating_add(1);
            shooter.score = shooter.score.saturating_add(kill_score);
            shooter.last_update = clock.unix_timestamp;

            msg!(
                "Player {} killed player {} at distance {:.2}. Respawn available in 3 seconds. Shooter stats - Kills: {}, Score: {}",
                shooter.authority,
                hit_player.authority,
                closest_hit_distance,
                shooter.kills,
                shooter.score
            );
        } else {
            msg!(
                "Player {} hit player {} for {} damage (health: {} -> {})",
                shooter.authority,
                hit_player.authority,
                damage,
                old_health,
                hit_player.health
            );
        }

        hit_player.last_update = clock.unix_timestamp;

        // Serialize back
        hit_player.try_serialize(&mut &mut hit_player_data[..])?;
    } else {
        msg!("Player {} missed", shooter.authority);
    }

    Ok(())
}

/// Ray-box intersection using slab method
/// Returns Some(distance) if hit, None if miss
#[allow(clippy::arithmetic_side_effects)]
#[allow(clippy::float_arithmetic)]
fn ray_box_intersection(
    origin_x: f32, origin_y: f32, origin_z: f32,
    dir_x: f32, dir_y: f32, dir_z: f32,
    box_min_x: f32, box_min_y: f32, box_min_z: f32,
    box_max_x: f32, box_max_y: f32, box_max_z: f32,
) -> Option<f32> {
    let mut t_min = 0.0_f32;
    let mut t_max = f32::MAX;

    // X axis
    if dir_x.abs() > f32::EPSILON {
        let t1 = (box_min_x - origin_x) / dir_x;
        let t2 = (box_max_x - origin_x) / dir_x;
        t_min = t_min.max(t1.min(t2));
        t_max = t_max.min(t1.max(t2));
    } else if origin_x < box_min_x || origin_x > box_max_x {
        return None;
    }

    // Y axis
    if dir_y.abs() > f32::EPSILON {
        let t1 = (box_min_y - origin_y) / dir_y;
        let t2 = (box_max_y - origin_y) / dir_y;
        t_min = t_min.max(t1.min(t2));
        t_max = t_max.min(t1.max(t2));
    } else if origin_y < box_min_y || origin_y > box_max_y {
        return None;
    }

    // Z axis
    if dir_z.abs() > f32::EPSILON {
        let t1 = (box_min_z - origin_z) / dir_z;
        let t2 = (box_max_z - origin_z) / dir_z;
        t_min = t_min.max(t1.min(t2));
        t_max = t_max.min(t1.max(t2));
    } else if origin_z < box_min_z || origin_z > box_max_z {
        return None;
    }

    // Check if ray intersects box
    if t_max >= t_min && t_min >= 0.0 {
        Some(t_min)
    } else {
        None
    }
}

#[derive(Accounts)]
pub struct Shoot<'info> {
    /// The player shooting (mutable to update kill stats)
    #[account(mut)]
    pub shooter: Account<'info, GamePlayer>,

    pub authority: Signer<'info>,

    // Remaining accounts: other GamePlayer accounts to check for hits
}
