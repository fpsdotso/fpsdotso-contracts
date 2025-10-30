use anchor_lang::prelude::*;

/// Game-specific player state containing position, rotation, and game stats
/// This is separate from the matchmaking Player account
#[account]
pub struct GamePlayer {
    /// The player's authority (wallet pubkey)
    pub authority: Pubkey,

    /// Reference to the game this player is in
    pub game_id: Pubkey,

    /// Position in 3D space
    pub position_x: f32,
    pub position_y: f32,
    pub position_z: f32,

    /// Rotation (Euler angles in radians)
    pub rotation_x: f32, // pitch
    pub rotation_y: f32, // yaw
    pub rotation_z: f32, // roll

    /// Health (0-100)
    pub health: u8,

    /// Is player alive
    pub is_alive: bool,

    /// Team (0 or 1)
    pub team: u8,

    /// Is spectator (cannot shoot or be shot)
    pub is_spectator: bool,

    /// Game stats
    pub kills: u32,
    pub deaths: u32,
    pub score: u32,

    /// Last update timestamp
    pub last_update: i64,

    /// Timestamp when player died (0 if alive)
    /// Used to enforce respawn cooldown
    pub death_timestamp: i64,

    /// Current bullet count (max 10)
    pub bullet_count: u8,

    /// Timestamp when reload started (0 if not reloading)
    /// Reload takes 0.5 seconds
    pub reload_start_timestamp: i64,

    /// Bump seed for PDA
    pub bump: u8,
}

impl GamePlayer {
    pub const SIZE: usize = 8 + // discriminator
        32 + // authority
        32 + // game_id
        4 + 4 + 4 + // position (3 x f32)
        4 + 4 + 4 + // rotation (3 x f32)
        1 + // health
        1 + // is_alive
        1 + // team
        1 + // is_spectator
        4 + 4 + 4 + // kills, deaths, score
        8 + // last_update
        8 + // death_timestamp
        1 + // bullet_count
        8 + // reload_start_timestamp
        1; // bump
}
