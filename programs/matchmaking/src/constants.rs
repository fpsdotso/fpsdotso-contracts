use anchor_lang::prelude::*;

#[constant]
pub const SEED: &str = "anchor";

// PDA Seeds
pub const PLAYER_SEED: &str = "player";
pub const GAME_SEED: &str = "game";

// Game Constraints
pub const MAX_PLAYERS_PER_TEAM: u8 = 5;
pub const MAX_TOTAL_PLAYERS: u8 = 10;
pub const MIN_PLAYERS_TO_START: u8 = 2;
