use anchor_lang::prelude::*;
use crate::error::InitPlayerError;
use crate::constants::PLAYER_SEED;

pub fn handler(ctx: Context<InitPlayer>, args: Vec<u8>) -> Result<()> {
    let clock = Clock::get()?;
    let player = &mut ctx.accounts.player;

    require!(!player.has_logged_in, InitPlayerError::AlreadyLoggedIn);
    
    let username = if args.is_empty() {
        let timestamp_suffix = (clock.unix_timestamp as u32) % 100000;
        format!("Player{}", timestamp_suffix)
    } else {
        let username_len = args[0] as usize;
        if username_len == 0 || username_len > 32 || args.len() < 1 + username_len {
            return Err(InitPlayerError::InvalidUsername.into());
        }
        
        let username_bytes = &args[1..1 + username_len];
        String::from_utf8(username_bytes.to_vec())
            .map_err(|_| InitPlayerError::InvalidUsername)?
    };
    
    require!(username.len() >= 3 && username.len() <= 32, InitPlayerError::InvalidUsernameLength);
    
    player.authority = ctx.accounts.authority.key();
    player.signing_key = ctx.accounts.signing_key.key();
    msg!("Player Signing Key is already ready {}", player.signing_key);
    player.username = username;
    player.has_logged_in = true;

    // is_alive is true when player joins a game
    player.is_alive = false;
    player.team = 0;
    player.current_game = None;
    player.last_login_timestamp = clock.unix_timestamp;
    player.total_matches_played = 0;
    player.level = 1;
    player.is_ready = false;
    player.is_spectator = false;  // Not a spectator by default
    player.game_counter = 0;

    // Initialize position and rotation to default values
    player.position_x = 0.0;
    player.position_y = 0.0;
    player.position_z = 0.0;
    player.rotation_x = 0.0;
    player.rotation_y = 0.0;
    player.rotation_z = 0.0;

    Ok(())
}

#[derive(Accounts)]
pub struct InitPlayer<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + // discriminator
                32 + 32 +// authority + signer
                (4 + 32) + // username string
                1 + 1 + (1 + 32) + 1 + 8 + 4 + 4 + // has_logged_in, team, current_game, is_alive, last_login_timestamp, total_matches_played, level
                1 + 1 + 4 + // is_ready, is_spectator, game_counter
                4 + 4 + 4 + 4 + 4 + 4, // position (x,y,z) + rotation (x,y,z) - 6 f32 fields
        seeds = [PLAYER_SEED.as_bytes(), authority.key().as_ref()],
        bump
    )]
    pub player: Account<'info, crate::state::Player>,
    
    #[account(mut)]
    pub authority: Signer<'info>,

    /// CHECK: Signing key address provided by the user - no validation needed as we only store the pubkey
    pub signing_key: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}
