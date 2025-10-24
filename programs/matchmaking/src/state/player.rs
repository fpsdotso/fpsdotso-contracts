use anchor_lang::prelude::*;

#[account]
pub struct Player {
    pub authority: Pubkey,          
    pub username: String,            
    pub has_logged_in: bool,          
    pub team: u8,                    // 0 = no team, 1 = Team A, 2 = Team B
    pub current_game: Option<Pubkey>, // PDA of the current game the player is in
    pub is_alive: bool,              
    pub last_login_timestamp: i64,   
    pub total_matches_played: u32,   
    pub level: u32,
    
    // NEW: Lobby state
    pub is_ready: bool,
    
    // Game creation counter for unique Game PDA derivation
    pub game_counter: u32,
}