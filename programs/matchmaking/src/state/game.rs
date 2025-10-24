use anchor_lang::prelude::*;

#[account]
pub struct Game {
    pub team_a_score: u32,           
    pub team_b_score: u32,           
    pub match_duration: u32,         
    pub match_start_timestamp: i64,  
    pub match_end_timestamp: Option<i64>,  
    pub game_state: u8,             // 0=waiting, 1=active, 2=ended, 3=paused
    pub max_players_per_team: u8,    
    pub current_players_team_a: u8,  
    pub current_players_team_b: u8, 
    pub winning_team: Option<u8>,   // Winning team (0=draw, 1=team_a, 2=team_b)
    pub match_type: u8,             // Match type (1=team_deathmatch) for now
    pub map_name: String,
    
    // NEW: Lobby features
    pub lobby_name: String,
    pub created_by: Pubkey,
    pub is_private: bool,
    pub ready_players: u8,
    pub map_selection: u8,  // 0=default, 1=map1, 2=map2, etc.
}