use anchor_lang::prelude::*;

pub const MAP_REGISTRY_SEED: &[u8] = b"fps.so map-registry";
pub const MAP_METADATA_SEED: &[u8] = b"fps.so map-metadata";
pub const MAP_DATA_SEED: &[u8] = b"fps.so map-data";
pub const USER_MAP_INDEX_SEED: &[u8] = b"fps.so user-map-index";

declare_id!("6XPHneawKSf2BWTtfZurtMdVvBiKsriTnGLKjoWdK791");

#[program]
pub mod map_registry {
    use super::*;

    /// Initializes the global map registry
    /// 
    /// This should be called once to set up the program's global state.
    /// It creates a PDA account that tracks the total number of default and user-created maps.
    /// 
    /// # Arguments
    /// * `ctx` - The context containing the map_registry account to initialize
    /// 
    /// # Accounts
    /// * `map_registry` - The global registry account (PDA)
    /// * `user` - The signer paying for account creation
    /// * `system_program` - Solana system program for account creation
    pub fn initialize(ctx: Context<InitializeMapRegistry>) -> Result<()> {
        let map_registry = &mut ctx.accounts.map_registry;
        // Initialize counters to zero
        map_registry.default_maps_count = 0;
        map_registry.user_maps_count = 0;
        Ok(())
    }

    /// Creates a new map with both metadata and data
    /// 
    /// This instruction:
    /// 1. Creates a MapMetadata PDA to store map information (name, description, creator, etc.)
    /// 2. Creates a MapData PDA to store the actual map data (game level data, tiles, etc.)
    /// 3. Updates the global map registry counters
    /// 4. Adds the map to the creator's personal index for easy lookup
    /// 
    /// # Arguments
    /// * `ctx` - The context containing all required accounts
    /// * `map_id` - Unique identifier for the map (used to derive PDAs)
    /// * `name` - Display name of the map
    /// * `description` - Description of the map
    /// * `is_default` - Whether this is a default/official map or user-created
    /// * `map_data` - The actual map data as a byte array (level layout, objects, etc.)
    /// 
    /// # Accounts
    /// * `map_metadata` - New PDA to store map metadata
    /// * `map_data_account` - New PDA to store actual map data
    /// * `map_registry` - Global registry to update counters
    /// * `user_map_index` - User's personal index to track their maps
    /// * `user` - The map creator and transaction signer
    /// * `system_program` - Solana system program
    pub fn create_map(
        ctx: Context<CreateMap>,
        map_id: String,
        name: String,
        description: String,
        is_default: bool,
        map_data: Vec<u8>,
    ) -> Result<()> {
        let map_metadata = &mut ctx.accounts.map_metadata;
        let map_registry = &mut ctx.accounts.map_registry;
        let user_index = &mut ctx.accounts.user_map_index;
        let map_data_account = &mut ctx.accounts.map_data_account;
        
        // Store all metadata about the map
        map_metadata.map_id = map_id.clone();
        map_metadata.name = name;
        map_metadata.description = description;
        map_metadata.creator = ctx.accounts.user.key();

        let current_timestamp = Clock::get()?.unix_timestamp;
        map_metadata.created_at = current_timestamp;
        map_metadata.updated_at = current_timestamp;
        map_metadata.is_default = is_default;
        
        // Validate map data size and store it
        require!(
            map_data.len() <= MapData::MAX_SIZE,
            ErrorCode::MapDataTooLarge
        );
        map_data_account.data = map_data;

        // Update global counters based on map type
        if is_default {
            map_registry.default_maps_count += 1;
        } else {
            map_registry.user_maps_count += 1;
        }

        // Add map to the user's personal index
        // This allows users to easily query all maps they've created
        require!(
            user_index.map_ids.len() < 100,
            ErrorCode::UserMapLimitReached
        );
        user_index.map_ids.push(map_id);
        user_index.map_count += 1;

        Ok(())
    }

    /// Updates the metadata of an existing map
    /// 
    /// Only the map creator can update their map's metadata.
    /// This allows changing the name and/or description without touching the actual map data.
    /// The updated_at timestamp is automatically refreshed.
    /// 
    /// # Arguments
    /// * `ctx` - The context containing the map_metadata account
    /// * `name` - Optional new name for the map
    /// * `description` - Optional new description for the map
    /// 
    /// # Accounts
    /// * `map_metadata` - The map's metadata account to update
    /// * `user` - Must be the original creator of the map
    /// 
    /// # Security
    /// * Checks that the signer is the map creator before allowing updates
    pub fn update_map_metadata(
        ctx: Context<UpdateMapMetadata>,
        name: Option<String>,
        description: Option<String>,
    ) -> Result<()> {
        let map_metadata = &mut ctx.accounts.map_metadata;
        
        // Only the creator can update their map
        require!(
            map_metadata.creator == ctx.accounts.user.key(),
            ErrorCode::Unauthorized
        );

        // Update only the fields that were provided
        if let Some(new_name) = name {
            map_metadata.name = new_name;
        }
        if let Some(new_description) = description {
            map_metadata.description = new_description;
        }
        
        // Update the timestamp to reflect when the map was last modified
        map_metadata.updated_at = Clock::get()?.unix_timestamp;

        Ok(())
    }

    /// Updates the actual map data
    /// 
    /// Only the map creator can update their map's data.
    /// This replaces the entire map data with new data.
    /// Uses reallocation to resize the account if the new data is a different size.
    /// 
    /// # Arguments
    /// * `ctx` - The context containing map_metadata and map_data_account
    /// * `map_data` - The new map data to replace the existing data
    /// 
    /// # Accounts
    /// * `map_metadata` - Used to verify the creator (read-only)
    /// * `map_data_account` - The data account to update (will be reallocated if needed)
    /// * `user` - Must be the original creator of the map
    /// * `system_program` - Needed for reallocation
    /// 
    /// # Security
    /// * Checks that the signer is the map creator before allowing updates
    /// * Validates new data size is within limits
    pub fn update_map_data(
        ctx: Context<UpdateMapData>,
        map_data: Vec<u8>,
    ) -> Result<()> {
        let map_metadata = &ctx.accounts.map_metadata;
        let map_data_account = &mut ctx.accounts.map_data_account;
        
        // Only the creator can update their map
        require!(
            map_metadata.creator == ctx.accounts.user.key(),
            ErrorCode::Unauthorized
        );
        
        // Validate the new data size
        require!(
            map_data.len() <= MapData::MAX_SIZE,
            ErrorCode::MapDataTooLarge
        );
        
        // Replace the old data with new data
        // The account is automatically resized via realloc in the Context
        map_data_account.data = map_data;

        Ok(())
    }

    /// Deletes a map and all its associated data
    /// 
    /// Only the map creator can delete their map.
    /// This instruction:
    /// 1. Removes the map from the user's personal index
    /// 2. Closes the map_metadata account (refunds rent to user)
    /// 3. Closes the map_data_account (refunds rent to user)
    /// 
    /// # Arguments
    /// * `ctx` - The context containing all required accounts
    /// 
    /// # Accounts
    /// * `map_metadata` - The metadata account to close
    /// * `map_data_account` - The data account to close
    /// * `user_map_index` - The user's index to remove the map from
    /// * `user` - Must be the original creator (receives rent refund)
    /// * `creator` - The creator's public key for validation
    /// * `system_program` - Needed for closing accounts
    /// 
    /// # Security
    /// * Checks that the signer is the map creator before allowing deletion
    /// * Closes accounts to prevent them from being used again
    pub fn delete_map(ctx: Context<DeleteMap>) -> Result<()> {
        let map_metadata = &ctx.accounts.map_metadata;
        let user_index = &mut ctx.accounts.user_map_index;
        
        // Only the creator can delete their map
        require!(
            map_metadata.creator == ctx.accounts.user.key(),
            ErrorCode::Unauthorized
        );

        // Remove the map from the user's personal index
        // This maintains the integrity of the user's map list
        let map_id = &map_metadata.map_id;
        if let Some(index) = user_index.map_ids[0..user_index.map_count as usize]
            .iter()
            .position(|id| id == map_id) 
        {
            // Remove the map_id from the vector
            user_index.map_ids.remove(index);
            user_index.map_count -= 1;
        }

        // The accounts are automatically closed due to the 'close' constraint
        // This refunds the rent to the user
        Ok(())
    }
}

// ============================================================================
// Account Contexts (define which accounts each instruction needs)
// ============================================================================

/// Context for initializing the global map registry
#[derive(Accounts)]
pub struct InitializeMapRegistry<'info> {
    /// The global registry PDA that tracks all maps
    /// Uses init_if_needed so it can be called multiple times safely
    #[account(
        init_if_needed,
        payer = user,
        space = 8 + MapRegistry::INIT_SPACE,
        seeds = [MAP_REGISTRY_SEED],
        bump
    )]
    pub map_registry: Account<'info, MapRegistry>,
    
    /// The user paying for the account creation
    #[account(mut)]
    pub user: Signer<'info>,
    
    /// Solana system program (required for creating accounts)
    pub system_program: Program<'info, System>,
}

/// Context for creating a new map
#[derive(Accounts)]
#[instruction(map_id: String, name: String, description: String, is_default: bool, map_data: Vec<u8>)]
pub struct CreateMap<'info> {
    /// New PDA account to store map metadata (name, creator, timestamps, etc.)
    /// Derived from [MAP_METADATA_SEED, map_id]
    #[account(
        init,
        payer = user,
        space = 8 + MapMetadata::INIT_SPACE,
        seeds = [MAP_METADATA_SEED, map_id.as_bytes()],
        bump
    )]
    pub map_metadata: Account<'info, MapMetadata>,
    
    /// New PDA account to store the actual map data (level layout, objects, etc.)
    /// Derived from [MAP_DATA_SEED, map_id]
    /// Space is calculated dynamically based on the size of map_data
    #[account(
        init,
        payer = user,
        space = 8 + 4 + map_data.len(), // 8 (discriminator) + 4 (vec length) + data
        seeds = [MAP_DATA_SEED, map_id.as_bytes()],
        bump
    )]
    pub map_data_account: Account<'info, MapData>,
    
    /// The global registry to update counters
    #[account(mut)]
    pub map_registry: Account<'info, MapRegistry>,
    
    /// The user's personal index of maps they've created
    /// Uses init_if_needed for first-time map creators
    #[account(
        init_if_needed,
        payer = user,
        space = 8 + UserMapIndex::INIT_SPACE,
        seeds = [USER_MAP_INDEX_SEED, user.key().as_ref()],
        bump
    )]
    pub user_map_index: Account<'info, UserMapIndex>,
    
    /// The map creator and transaction signer
    #[account(mut)]
    pub user: Signer<'info>,
    
    /// Solana system program
    pub system_program: Program<'info, System>,
}

/// Context for updating map metadata (name, description)
#[derive(Accounts)]
pub struct UpdateMapMetadata<'info> {
    /// The map's metadata account to update
    #[account(mut)]
    pub map_metadata: Account<'info, MapMetadata>,
    
    /// The user attempting to update (must be the creator)
    #[account(mut)]
    pub user: Signer<'info>,
}

/// Context for updating map data
#[derive(Accounts)]
#[instruction(map_data: Vec<u8>)]
pub struct UpdateMapData<'info> {
    /// The map's metadata (used to verify the creator)
    /// Seeds constraint ensures we're working with the correct map
    #[account(
        seeds = [MAP_METADATA_SEED, map_metadata.map_id.as_bytes()],
        bump
    )]
    pub map_metadata: Account<'info, MapMetadata>,
    
    /// The map's data account to update
    /// Uses realloc to resize the account if the new data is a different size
    #[account(
        mut,
        realloc = 8 + 4 + map_data.len(),
        realloc::payer = user,
        realloc::zero = false,
        seeds = [MAP_DATA_SEED, map_metadata.map_id.as_bytes()],
        bump
    )]
    pub map_data_account: Account<'info, MapData>,
    
    /// The user attempting to update (must be the creator)
    #[account(mut)]
    pub user: Signer<'info>,
    
    /// System program (needed for reallocation)
    pub system_program: Program<'info, System>,
}

/// Context for deleting a map
#[derive(Accounts)]
pub struct DeleteMap<'info> {
    /// The map's metadata account
    /// close = user means rent is refunded to user when account is closed
    /// has_one = creator ensures the creator field matches the creator account
    #[account(
        mut,
        close = user,
        has_one = creator @ ErrorCode::Unauthorized,
        seeds = [MAP_METADATA_SEED, map_metadata.map_id.as_bytes()],
        bump
    )]
    pub map_metadata: Account<'info, MapMetadata>,
    
    /// The map's data account to close
    #[account(
        mut,
        close = user,
        seeds = [MAP_DATA_SEED, map_metadata.map_id.as_bytes()],
        bump
    )]
    pub map_data_account: Account<'info, MapData>,
    
    /// The user's personal index to update
    #[account(
        mut,
        seeds = [USER_MAP_INDEX_SEED, user.key().as_ref()],
        bump
    )]
    pub user_map_index: Account<'info, UserMapIndex>,
    
    /// The user deleting the map (must be the creator)
    #[account(mut)]
    pub user: Signer<'info>,
    
    /// The creator's public key for validation
    /// CHECK: This is validated by the has_one constraint on map_metadata
    pub creator: AccountInfo<'info>,
    
    /// System program
    pub system_program: Program<'info, System>,
}

// ============================================================================
// Account Structures (data stored on-chain)
// ============================================================================

/// Global registry that tracks statistics about all maps
#[account]
#[derive(InitSpace)]
pub struct MapRegistry {
    /// Number of official/default maps
    pub default_maps_count: u32,
    /// Number of user-created maps
    pub user_maps_count: u32,
}

/// Metadata about a specific map
#[account]
#[derive(InitSpace)]
pub struct MapMetadata {
    /// Unique identifier for the map (used in PDA derivation)
    #[max_len(50)]
    pub map_id: String,
    
    /// Display name shown to users
    #[max_len(100)]
    pub name: String,
    
    /// Description of the map
    #[max_len(500)]
    pub description: String,
    
    /// Public key of the user who created this map
    pub creator: Pubkey,
    
    /// Unix timestamp when the map was created
    pub created_at: i64,
    
    /// Unix timestamp when the map was last updated
    pub updated_at: i64,
    
    /// Whether this is an official map or user-created
    pub is_default: bool,
}

/// The actual map data (level layout, tiles, objects, etc.)
#[account]
pub struct MapData {
    /// Raw byte data representing the map
    /// Format is application-specific (could be JSON, binary format, etc.)
    pub data: Vec<u8>,
}

impl MapData {
    /// Maximum size for map data (10KB)
    /// Adjust this based on your needs and Solana account size limits
    pub const MAX_SIZE: usize = 10_000;
}

/// Personal index for each user to track their created maps
#[account]
#[derive(InitSpace)]
pub struct UserMapIndex {
    /// The user who owns this index
    pub owner: Pubkey,
    
    /// Number of maps this user has created
    pub map_count: u32,
    
    /// List of map IDs this user has created
    /// Maximum 100 maps per user, each map_id up to 50 characters
    #[max_len(100, 50)]
    pub map_ids: Vec<String>,
}

// ============================================================================
// Custom Error Codes
// ============================================================================

#[error_code]
pub enum ErrorCode {
    #[msg("Unauthorized: You are not the creator of this map")]
    Unauthorized,
    
    #[msg("User has reached the maximum number of maps (100)")]
    UserMapLimitReached,
    
    #[msg("Map data exceeds maximum allowed size (10KB)")]
    MapDataTooLarge,
}