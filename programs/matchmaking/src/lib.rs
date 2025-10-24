pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use error::*;
pub use instructions::*;
pub use state::*;

declare_id!("FK438BMUMqFqZj33Argueh89XGTwxEPXCEU7JgeMXLvA");

#[program]
pub mod matchmaking {
    use super::*;

    pub fn init_player(ctx: Context<InitPlayer>, args: Vec<u8>) -> Result<()> {
        init_player::handler(ctx, args)
    }

    pub fn init_game(ctx: Context<InitGame>) -> Result<()> {
        init_game::handler(ctx)
    }

    pub fn join_game(ctx: Context<JoinGame>) -> Result<()> {
        join_game::handler(ctx)
    }

    pub fn leave_game(ctx: Context<LeaveGame>) -> Result<()> {
        leave_game::handler(ctx)
    }

    pub fn start_game(ctx: Context<StartGame>) -> Result<()> {
        start_game::handler(ctx)
    }
}
