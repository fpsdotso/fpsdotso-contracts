use anchor_lang::prelude::*;

#[error_code]
pub enum InitPlayerError {
    #[msg("Player has already logged in")]
    AlreadyLoggedIn,
    #[msg("Invalid username format")]
    InvalidUsername,
    #[msg("Username must be between 3 and 32 characters")]
    InvalidUsernameLength,
}

#[error_code]
pub enum InitGameError {
    #[msg("Player not registered")]
    PlayerNotRegistered,
    #[msg("Player already in a game")]
    PlayerAlreadyInGame,
}

#[error_code]
pub enum JoinGameError {
    #[msg("Player not registered")]
    PlayerNotRegistered,
    #[msg("Player already in a game")]
    PlayerAlreadyInGame,
    #[msg("Game has already started")]
    GameAlreadyStarted,
    #[msg("Game is full")]
    GameFull,
    #[msg("Invalid game state")]
    InvalidGameState,
}

#[error_code]
pub enum LeaveGameError {
    #[msg("Player not in a game")]
    PlayerNotInGame,
    #[msg("Player not in this specific game")]
    PlayerNotInThisGame,
}

#[error_code]
pub enum StartGameError {
    #[msg("Game has already started")]
    GameAlreadyStarted,
    #[msg("Not enough players to start game")]
    NotEnoughPlayers,
    #[msg("Cannot start game - not lobby owner and not all players ready")]
    CannotStartGame,
    #[msg("Only lobby owner can start the game")]
    NotLobbyOwner,
}
