use anchor_lang::prelude::*;
use ephemeral_rollups_sdk::anchor::{commit, delegate, ephemeral};
use ephemeral_rollups_sdk::cpi::DelegateConfig;
use ephemeral_rollups_sdk::ephem::{commit_accounts, commit_and_undelegate_accounts};

declare_id!("FZFQ2izwmvWzES3Ajn1BtMd5UBbt8UZ7EXgR63hK6sSE");

pub const TEST_PDA_SEED: &[u8] = b"test-pda";
#[ephemeral]
#[program]
pub mod anchor_counter {
    use super::*;

    /// Initialize the counter.
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let game_world = &mut ctx.accounts.game_world;
        game_world.x = 0.0;
        game_world.y = 0.0;
        game_world.z = 0.0;
        msg!("PDA {} location: {},{},{}", game_world.key(), game_world.x, game_world.y, game_world.z);
        Ok(())
    }

    pub fn update_location(ctx: Context<UpdateLocation>, x: f64, y: f64, z: f64) -> Result<()> {
        let game_world = &mut ctx.accounts.game_world;
        game_world.x = x;
        game_world.y = y;
        game_world.z = z;
        msg!("Game World location updated to: ({}, {}, {})", x, y, z);
        Ok(())
    }

    pub fn get_location(ctx: Context<GetLocation>) -> Result<(f64, f64, f64)> {
        let game_world = &ctx.accounts.game_world;
        msg!("Game World location is: ({}, {}, {})", game_world.x, game_world.y, game_world.z);
        Ok((game_world.x, game_world.y, game_world.z))
    }

    /// Delegate the account to the delegation program
    /// Set specific validator based on ER, see https://docs.magicblock.gg/pages/get-started/how-integrate-your-program/local-setup
    pub fn delegate(ctx: Context<DelegateInput>) -> Result<()> {
        msg!("Delegating PDA: {}", ctx.accounts.pda.key());
        ctx.accounts.delegate_pda(
            &ctx.accounts.payer,
            &[TEST_PDA_SEED],
            DelegateConfig {
                // Optionally set a specific validator from the first remaining account
                validator: ctx.remaining_accounts.first().map(|acc| acc.key()),
                ..Default::default()
            },
        )?;
        Ok(())
    }

    /// Manual commit the account in the ER.
    pub fn commit(ctx: Context<UpdateLocationAndCommit>) -> Result<()> {
        commit_accounts(
            &ctx.accounts.payer,
            vec![&ctx.accounts.game_world.to_account_info()],
            &ctx.accounts.magic_context,
            &ctx.accounts.magic_program,
        )?;
        Ok(())
    }

    /// Undelegate the account from the delegation program
    pub fn undelegate(ctx: Context<UpdateLocationAndCommit>) -> Result<()> {
        commit_and_undelegate_accounts(
            &ctx.accounts.payer,
            vec![&ctx.accounts.game_world.to_account_info()],
            &ctx.accounts.magic_context,
            &ctx.accounts.magic_program,
        )?;
        Ok(())
    }

        /// Increment the counter + manual commit the account in the ER.
    pub fn update_location_and_commit(ctx: Context<UpdateLocationAndCommit>, x: f64, y: f64, z: f64) -> Result<()> {
        let game_world = &mut ctx.accounts.game_world;
        game_world.x = x;
        game_world.y = y;
        game_world.z = z;
        msg!("PDA {} location: ({}, {}, {})", game_world.key(), game_world.x, game_world.y, game_world.z);
        // Serialize the Anchor game_world account, commit and undelegate
        game_world.exit(&crate::ID)?;
        commit_accounts(
            &ctx.accounts.payer,
            vec![&ctx.accounts.game_world.to_account_info()],
            &ctx.accounts.magic_context,
            &ctx.accounts.magic_program,
        )?;
        Ok(())
    }

    /// Increment the counter + manual commit the account in the ER.
    pub fn update_location_and_undelegate(ctx: Context<UpdateLocationAndCommit>, x: f64, y: f64, z: f64) -> Result<()> {
        let game_world = &mut ctx.accounts.game_world;
        game_world.x = x;
        game_world.y = y;
        game_world.z = z;
        msg!("PDA {} location: ({}, {}, {})", game_world.key(), game_world.x, game_world.y, game_world.z);
        // Serialize the Anchor game_world account, commit and undelegate
        game_world.exit(&crate::ID)?;
        commit_and_undelegate_accounts(
            &ctx.accounts.payer,
            vec![&ctx.accounts.game_world.to_account_info()],
            &ctx.accounts.magic_context,
            &ctx.accounts.magic_program,
        )?;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init_if_needed, payer = user, space = 8 + 24, seeds = [TEST_PDA_SEED], bump)]
    pub game_world: Account<'info, GameWorld>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

/// Add delegate function to the context
#[delegate]
#[derive(Accounts)]
pub struct DelegateInput<'info> {
    pub payer: Signer<'info>,
    /// CHECK The pda to delegate
    #[account(mut, del)]
    pub pda: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct UpdateLocation<'info> {
    #[account(mut, seeds = [TEST_PDA_SEED], bump)]
    pub game_world: Account<'info, GameWorld>,
}

#[derive(Accounts)]
pub struct GetLocation<'info> {
    #[account(seeds = [TEST_PDA_SEED], bump)]
    pub game_world: Account<'info, GameWorld>,
}

/// Account for the increment instruction + manual commit.
#[commit]
#[derive(Accounts)]
pub struct UpdateLocationAndCommit<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut, seeds = [TEST_PDA_SEED], bump)]
    pub game_world: Account<'info, GameWorld>,
}

#[account]
pub struct GameWorld {
    pub x: f64,
    pub y: f64,
    pub z: f64
}