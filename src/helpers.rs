// Copyright (c) The StackClass Authors. All rights reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Helper functions for testing the swap program.

use crate::mollusk::{
    ProgramLoadError, TestContextError, init_test_context, load_swap_program, load_swap_program_id,
};
use mollusk_svm::{program::keyed_account_for_system_program, result::Check};
use mollusk_svm_programs_token::{associated_token, token};
use sha2::{Digest, Sha256};
use solana_account::Account;
use solana_instruction::{AccountMeta, Instruction};
use solana_program_option::COption;
use solana_pubkey::Pubkey;
use spl_associated_token_account_interface::address::get_associated_token_address_with_program_id;
use spl_token_interface::state::{Account as TokenAccount, AccountState, Mint};
use std::{convert::TryInto, path::Path};

/// Get the repository directory from environment variables.
///
/// This function reads the `STACKCLASS_REPOSITORY_DIR` environment variable
/// and returns it as a Path.
///
/// # Returns
///
/// * `Ok(PathBuf)` - The repository directory path
/// * `Err(ProgramLoadError)` - If the environment variable is not set
pub fn get_repo_dir() -> Result<std::path::PathBuf, ProgramLoadError> {
    std::env::var("STACKCLASS_REPOSITORY_DIR")
        .map_err(|_| ProgramLoadError::RepoNotFound(std::path::PathBuf::from("Not set")))
        .map(std::path::PathBuf::from)
}

/// Create a test error message for reporting to the user.
///
/// # Arguments
///
/// * `stage_name` - The name of the test stage
/// * `error` - The error that occurred
///
/// # Returns
///
/// * `String` - A formatted error message
#[allow(dead_code)]
pub fn format_test_error(stage_name: &str, error: &TestContextError) -> String {
    format!("Test '{}' failed: {}", stage_name, error)
}

/// Create a success message for reporting to the user.
///
/// # Arguments
///
/// * `stage_name` - The name of the test stage
///
/// # Returns
///
/// * `String` - A formatted success message
#[allow(dead_code)]
pub fn format_test_success(stage_name: &str) -> String {
    format!("Test '{}' passed successfully", stage_name)
}

/// Create a basic system account with lamports.
///
/// # Arguments
///
/// * `lamports` - The amount of lamports
///
/// # Returns
///
/// * `Account` - A system account
#[allow(dead_code)]
pub fn create_system_account(lamports: u64) -> Account {
    Account { lamports, owner: solana_system_program::id(), ..Default::default() }
}

/// Create a PDA (Program Derived Address) for the swap program.
///
/// # Arguments
///
/// * `seeds` - The seeds to derive the PDA from
/// * `program_id` - The program ID used for derivation
///
/// # Returns
///
/// * `(Pubkey, u8)` - The PDA public key and bump seed
#[allow(dead_code)]
pub fn create_pda(seeds: &[&[u8]], program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(seeds, program_id)
}

/// Create a check for successful execution.
///
/// # Returns
///
/// * `Check` - A success check
#[allow(dead_code)]
pub fn success_check() -> Check<'static> {
    Check::success()
}

/// Create a check for account lamports.
///
/// # Arguments
///
/// * `pubkey` - The account public key
/// * `expected_lamports` - The expected lamports
///
/// # Returns
///
/// * `Check` - A lamports check
#[allow(dead_code)]
pub fn lamports_check(pubkey: &Pubkey, expected_lamports: u64) -> Check<'_> {
    Check::account(pubkey).lamports(expected_lamports).build()
}

/// Create a check for account data.
///
/// # Arguments
///
/// * `pubkey` - The account public key
/// * `expected_data` - The expected account data
///
/// # Returns
///
/// * `Check` - A data check
#[allow(dead_code)]
pub fn data_check<'a>(pubkey: &'a Pubkey, expected_data: &'a [u8]) -> Check<'a> {
    Check::account(pubkey).data(expected_data).build()
}

/// Create a check for account owner.
///
/// # Arguments
///
/// * `pubkey` - The account public key
/// * `expected_owner` - The expected owner
///
/// # Returns
///
/// * `Check` - An owner check
#[allow(dead_code)]
pub fn owner_check<'a>(pubkey: &'a Pubkey, expected_owner: &'a Pubkey) -> Check<'a> {
    Check::account(pubkey).owner(expected_owner).build()
}

/// Create a check for account executability.
///
/// # Arguments
///
/// * `pubkey` - The account public key
/// * `expected_executable` - The expected executability
///
/// # Returns
///
/// * `Check` - An executable check
#[allow(dead_code)]
pub fn executable_check(pubkey: &Pubkey, expected_executable: bool) -> Check<'_> {
    Check::account(pubkey).executable(expected_executable).build()
}

/// Convert a TestContextError to a tester::CaseError.
///
/// # Arguments
///
/// * `error` - The TestContextError to convert
///
/// # Returns
///
/// * `tester::CaseError` - The converted error
pub fn to_case_error(error: TestContextError) -> tester::CaseError {
    Box::new(error) as Box<dyn std::error::Error + Send + Sync>
}

/// Convert a ProgramLoadError to a tester::CaseError.
///
/// # Arguments
///
/// * `error` - The ProgramLoadError to convert
///
/// # Returns
///
/// * `tester::CaseError` - The converted error
pub fn to_case_error_from_load(error: crate::mollusk::ProgramLoadError) -> tester::CaseError {
    Box::new(error) as Box<dyn std::error::Error + Send + Sync>
}

/// Convert a TestContextError to a tester::CaseError (for use with map_err).
///
/// This is a helper function for converting errors in a map_err context.
///
/// # Arguments
///
/// * `error` - The TestContextError to convert
///
/// # Returns
///
/// * `tester::CaseError` - The converted error
pub fn to_case_error_from_context(error: TestContextError) -> tester::CaseError {
    Box::new(error) as Box<dyn std::error::Error + Send + Sync>
}

/// Check if a program is available for testing.
///
/// # Arguments
///
/// * `repo_dir` - The repository directory
///
/// # Returns
///
/// * `Ok(())` - If the program is available
/// * `Err(tester::CaseError)` - If the program is not available
pub fn check_program_available(repo_dir: &Path) -> Result<(), tester::CaseError> {
    match load_swap_program(repo_dir) {
        Ok(_) => Ok(()),
        Err(e) => Err(Box::new(e) as Box<dyn std::error::Error + Send + Sync>),
    }
}

/// Create a test instruction for the swap program.
///
/// # Arguments
///
/// * `program_id` - The swap program ID
/// * `data` - The instruction data
/// * `accounts` - The accounts to pass to the instruction
///
/// # Returns
///
/// * `Instruction` - A swap program instruction
pub fn create_swap_instruction(
    program_id: Pubkey,
    data: Vec<u8>,
    accounts: Vec<AccountMeta>,
) -> Instruction {
    Instruction::new_with_bytes(program_id, &data, accounts)
}

const DEFAULT_OFFERED_AMOUNT: u64 = 1_000_000;
const DEFAULT_WANTED_AMOUNT: u64 = 1_000_000;
const DEFAULT_MINT_DECIMALS: u8 = 6;
const OFFER_SEED_PREFIX: &[u8] = b"offer";

#[derive(Debug, Clone)]
pub struct OfferData {
    pub id: u64,
    pub maker: Pubkey,
    pub token_mint_a: Pubkey,
    pub token_mint_b: Pubkey,
    pub token_b_wanted_amount: u64,
    pub bump: u8,
}

pub struct SwapFixture {
    context: crate::mollusk::SwapTestContext,
    program_id: Pubkey,
    pub maker: Pubkey,
    pub taker: Pubkey,
    pub token_mint_a: Pubkey,
    pub token_mint_b: Pubkey,
    pub maker_token_account_a: Pubkey,
    pub maker_token_account_b: Pubkey,
    pub taker_token_account_a: Pubkey,
    pub taker_token_account_b: Pubkey,
    pub offer_id: u64,
    pub offer: Pubkey,
    pub vault: Pubkey,
    pub token_program: Pubkey,
    pub associated_token_program: Pubkey,
    pub offered_amount: u64,
    pub wanted_amount: u64,
    #[allow(dead_code)]
    pub decimals_a: u8,
}

impl SwapFixture {
    pub fn new_default(repo_dir: &Path) -> Result<Self, TestContextError> {
        Self::new_with_amounts(
            repo_dir,
            DEFAULT_OFFERED_AMOUNT,
            DEFAULT_WANTED_AMOUNT,
            DEFAULT_OFFERED_AMOUNT,
            DEFAULT_WANTED_AMOUNT,
            DEFAULT_MINT_DECIMALS,
        )
    }

    pub fn new_with_amounts(
        repo_dir: &Path,
        offered_amount: u64,
        wanted_amount: u64,
        maker_balance_a: u64,
        taker_balance_b: u64,
        decimals: u8,
    ) -> Result<Self, TestContextError> {
        let mut context = init_test_context(repo_dir)?;
        let program_id = context.program_id();

        let (system_program_id, system_program_account) = keyed_account_for_system_program();
        context.add_account(system_program_id, system_program_account);

        let (token_program_id, token_program_account) = token::keyed_account();
        context.add_account(token_program_id, token_program_account);

        let (associated_program_id, associated_program_account) = associated_token::keyed_account();
        context.add_account(associated_program_id, associated_program_account);

        let maker = context.create_funded_account(1_000_000_000);
        let taker = context.create_funded_account(1_000_000_000);

        let token_mint_a = Pubkey::new_unique();
        let token_mint_b = Pubkey::new_unique();

        let mint_a = Mint {
            mint_authority: COption::Some(maker),
            supply: maker_balance_a,
            decimals,
            is_initialized: true,
            freeze_authority: COption::None,
        };
        let mint_b = Mint {
            mint_authority: COption::Some(taker),
            supply: taker_balance_b,
            decimals,
            is_initialized: true,
            freeze_authority: COption::None,
        };

        context.add_account(token_mint_a, token::create_account_for_mint(mint_a));
        context.add_account(token_mint_b, token::create_account_for_mint(mint_b));

        let maker_token_account_a =
            get_associated_token_address_with_program_id(&maker, &token_mint_a, &token_program_id);
        let maker_token_account_b =
            get_associated_token_address_with_program_id(&maker, &token_mint_b, &token_program_id);
        let taker_token_account_a =
            get_associated_token_address_with_program_id(&taker, &token_mint_a, &token_program_id);
        let taker_token_account_b =
            get_associated_token_address_with_program_id(&taker, &token_mint_b, &token_program_id);

        context.add_account(
            maker_token_account_a,
            token::create_account_for_token_account(TokenAccount {
                mint: token_mint_a,
                owner: maker,
                amount: maker_balance_a,
                delegate: COption::None,
                state: AccountState::Initialized,
                is_native: COption::None,
                delegated_amount: 0,
                close_authority: COption::None,
            }),
        );

        context.add_account(
            maker_token_account_b,
            token::create_account_for_token_account(TokenAccount {
                mint: token_mint_b,
                owner: maker,
                amount: 0,
                delegate: COption::None,
                state: AccountState::Initialized,
                is_native: COption::None,
                delegated_amount: 0,
                close_authority: COption::None,
            }),
        );

        context.add_account(
            taker_token_account_a,
            token::create_account_for_token_account(TokenAccount {
                mint: token_mint_a,
                owner: taker,
                amount: 0,
                delegate: COption::None,
                state: AccountState::Initialized,
                is_native: COption::None,
                delegated_amount: 0,
                close_authority: COption::None,
            }),
        );

        context.add_account(
            taker_token_account_b,
            token::create_account_for_token_account(TokenAccount {
                mint: token_mint_b,
                owner: taker,
                amount: taker_balance_b,
                delegate: COption::None,
                state: AccountState::Initialized,
                is_native: COption::None,
                delegated_amount: 0,
                close_authority: COption::None,
            }),
        );

        let offer_id: i32 = 1;
        let (offer, _bump) = Pubkey::find_program_address(
            &[OFFER_SEED_PREFIX, maker.as_ref(), &offer_id.to_le_bytes()],
            &program_id,
        );
        let vault =
            get_associated_token_address_with_program_id(&offer, &token_mint_a, &token_program_id);

        context.add_account(offer, empty_system_account());
        context.add_account(vault, empty_system_account());

        Ok(Self {
            context,
            program_id,
            maker,
            taker,
            token_mint_a,
            token_mint_b,
            maker_token_account_a,
            maker_token_account_b,
            taker_token_account_a,
            taker_token_account_b,
            offer_id: offer_id.try_into().unwrap(),
            offer,
            vault,
            token_program: token_program_id,
            associated_token_program: associated_program_id,
            offered_amount,
            wanted_amount,
            decimals_a: decimals,
        })
    }

    pub fn make_offer_instruction(&self) -> Instruction {
        let data = build_make_offer_data(self.offer_id, self.offered_amount, self.wanted_amount);
        create_swap_instruction(
            self.program_id,
            data,
            vec![
                AccountMeta::new(self.maker, true),
                AccountMeta::new_readonly(self.token_mint_a, false),
                AccountMeta::new_readonly(self.token_mint_b, false),
                AccountMeta::new(self.maker_token_account_a, false),
                AccountMeta::new(self.offer, false),
                AccountMeta::new(self.vault, false),
                AccountMeta::new_readonly(solana_system_program::id(), false),
                AccountMeta::new_readonly(self.token_program, false),
                AccountMeta::new_readonly(self.associated_token_program, false),
            ],
        )
    }

    pub fn take_offer_instruction(&self) -> Instruction {
        let data = build_take_offer_data();
        create_swap_instruction(
            self.program_id,
            data,
            vec![
                AccountMeta::new(self.taker, true),
                AccountMeta::new(self.maker, false),
                AccountMeta::new_readonly(self.token_mint_a, false),
                AccountMeta::new_readonly(self.token_mint_b, false),
                AccountMeta::new(self.taker_token_account_a, false),
                AccountMeta::new(self.taker_token_account_b, false),
                AccountMeta::new(self.maker_token_account_b, false),
                AccountMeta::new(self.offer, false),
                AccountMeta::new(self.vault, false),
                AccountMeta::new_readonly(solana_system_program::id(), false),
                AccountMeta::new_readonly(self.token_program, false),
                AccountMeta::new_readonly(self.associated_token_program, false),
            ],
        )
    }

    pub fn execute_make_offer(&mut self) -> Result<(), TestContextError> {
        let instruction = self.make_offer_instruction();
        self.context.execute_instruction(&instruction)
    }

    pub fn execute_take_offer(&mut self) -> Result<(), TestContextError> {
        let instruction = self.take_offer_instruction();
        self.context.execute_instruction(&instruction)
    }

    pub fn get_account(&self, pubkey: &Pubkey) -> Result<Account, TestContextError> {
        self.context
            .get_account(pubkey)
            .ok_or_else(|| TestContextError::AccountNotFound(pubkey.to_string()))
    }
}

fn empty_system_account() -> Account {
    Account {
        lamports: 0,
        owner: solana_system_program::id(),
        data: Vec::new(),
        ..Default::default()
    }
}

fn build_make_offer_data(id: u64, offered_amount: u64, wanted_amount: u64) -> Vec<u8> {
    let mut data = Vec::with_capacity(32);
    data.extend_from_slice(&anchor_discriminator("global:make_offer"));
    data.extend_from_slice(&id.to_le_bytes());
    data.extend_from_slice(&offered_amount.to_le_bytes());
    data.extend_from_slice(&wanted_amount.to_le_bytes());
    data
}

fn build_take_offer_data() -> Vec<u8> {
    anchor_discriminator("global:take_offer").to_vec()
}

fn anchor_discriminator(name: &str) -> [u8; 8] {
    let mut hasher = Sha256::new();
    hasher.update(name.as_bytes());
    let hash = hasher.finalize();
    let mut out = [0u8; 8];
    out.copy_from_slice(&hash[..8]);
    out
}

fn read_pubkey(data: &[u8]) -> Result<Pubkey, TestContextError> {
    let bytes: [u8; 32] = data
        .try_into()
        .map_err(|_| TestContextError::ValidationError("Invalid pubkey bytes".to_string()))?;
    Ok(Pubkey::new_from_array(bytes))
}

fn read_u64(data: &[u8]) -> Result<u64, TestContextError> {
    let bytes: [u8; 8] = data
        .try_into()
        .map_err(|_| TestContextError::ValidationError("Invalid u64 bytes".to_string()))?;
    Ok(u64::from_le_bytes(bytes))
}

fn token_account_amount(account: &Account) -> Result<u64, TestContextError> {
    if account.data.len() < 72 {
        return Err(TestContextError::ValidationError("Token account data too short".to_string()));
    }
    read_u64(&account.data[64..72])
}

fn token_account_owner(account: &Account) -> Result<Pubkey, TestContextError> {
    if account.data.len() < 64 {
        return Err(TestContextError::ValidationError("Token account data too short".to_string()));
    }
    read_pubkey(&account.data[32..64])
}

fn token_account_mint(account: &Account) -> Result<Pubkey, TestContextError> {
    if account.data.len() < 32 {
        return Err(TestContextError::ValidationError("Token account data too short".to_string()));
    }
    read_pubkey(&account.data[0..32])
}

fn offer_data_from_account(account: &Account) -> Result<OfferData, TestContextError> {
    if account.data.len() < 8 + 8 + 32 + 32 + 32 + 8 + 1 {
        return Err(TestContextError::ValidationError("Offer account data too short".to_string()));
    }
    let mut offset = 8;
    let id = read_u64(&account.data[offset..offset + 8])?;
    offset += 8;
    let maker = read_pubkey(&account.data[offset..offset + 32])?;
    offset += 32;
    let token_mint_a = read_pubkey(&account.data[offset..offset + 32])?;
    offset += 32;
    let token_mint_b = read_pubkey(&account.data[offset..offset + 32])?;
    offset += 32;
    let token_b_wanted_amount = read_u64(&account.data[offset..offset + 8])?;
    offset += 8;
    let bump = account.data[offset];

    Ok(OfferData { id, maker, token_mint_a, token_mint_b, token_b_wanted_amount, bump })
}

fn make_offer_success(fixture: &mut SwapFixture) -> Result<(), TestContextError> {
    fixture.execute_make_offer()
}

fn take_offer_success(fixture: &mut SwapFixture) -> Result<(), TestContextError> {
    fixture.execute_take_offer()
}

pub fn run_env_setup_check() -> Result<(), tester::CaseError> {
    let repo_path = get_repo_dir().map_err(to_case_error_from_load)?;
    if !repo_path.exists() {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Repository directory not found: {}", repo_path.display()),
        )) as Box<dyn std::error::Error + Send + Sync>);
    }
    check_program_available(&repo_path)?;
    run_make_offer_smoke(&repo_path)
}

pub fn run_rust_basics_check() -> Result<(), tester::CaseError> {
    let repo_path = get_repo_dir().map_err(to_case_error_from_load)?;
    run_make_offer_smoke(&repo_path)
}

pub fn run_solana_model_check() -> Result<(), tester::CaseError> {
    let repo_path = get_repo_dir().map_err(to_case_error_from_load)?;
    let mut fixture = SwapFixture::new_default(&repo_path).map_err(to_case_error)?;
    match fixture.execute_make_offer() {
        Ok(()) => {
            let offer_account = fixture.get_account(&fixture.offer)?;
            if offer_account.owner != fixture.program_id {
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Offer account owner does not match program id",
                )) as Box<dyn std::error::Error + Send + Sync>);
            }
            Ok(())
        }
        Err(TestContextError::ExecutionError(_)) => Ok(()),
        Err(err) => Err(to_case_error(err)),
    }
}

pub fn run_anchor_try_check() -> Result<(), tester::CaseError> {
    let repo_path = get_repo_dir().map_err(to_case_error_from_load)?;
    let program_id = load_swap_program_id(&repo_path).map_err(to_case_error_from_load)?;
    if program_id == Pubkey::default() {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Program ID is still default",
        )) as Box<dyn std::error::Error + Send + Sync>);
    }
    run_make_offer_smoke(&repo_path)
}

pub fn run_spl_token_basics_check() -> Result<(), tester::CaseError> {
    let repo_path = get_repo_dir().map_err(to_case_error_from_load)?;
    let mut fixture = SwapFixture::new_default(&repo_path).map_err(to_case_error)?;
    make_offer_success(&mut fixture).map_err(to_case_error)?;
    let vault_account = fixture.get_account(&fixture.vault)?;
    let vault_mint = token_account_mint(&vault_account).map_err(to_case_error_from_context)?;
    if vault_mint != fixture.token_mint_a {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Vault mint mismatch",
        )) as Box<dyn std::error::Error + Send + Sync>);
    }
    Ok(())
}

pub fn run_cpi_transfer_check() -> Result<(), tester::CaseError> {
    let repo_path = get_repo_dir().map_err(to_case_error_from_load)?;
    let mut fixture = SwapFixture::new_default(&repo_path).map_err(to_case_error)?;
    make_offer_success(&mut fixture).map_err(to_case_error)?;

    let vault_account = fixture.get_account(&fixture.vault)?;
    let vault_amount = token_account_amount(&vault_account).map_err(to_case_error_from_context)?;
    if vault_amount != fixture.offered_amount {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Vault balance does not match offered amount",
        )) as Box<dyn std::error::Error + Send + Sync>);
    }
    Ok(())
}

pub fn run_token_transfer_check() -> Result<(), tester::CaseError> {
    let repo_path = get_repo_dir().map_err(to_case_error_from_load)?;
    let mut fixture = SwapFixture::new_default(&repo_path).map_err(to_case_error)?;
    make_offer_success(&mut fixture).map_err(to_case_error)?;
    take_offer_success(&mut fixture).map_err(to_case_error)?;

    let taker_token_a = fixture.get_account(&fixture.taker_token_account_a)?;
    let maker_token_b = fixture.get_account(&fixture.maker_token_account_b)?;
    let taker_amount = token_account_amount(&taker_token_a).map_err(to_case_error_from_context)?;
    let maker_amount = token_account_amount(&maker_token_b).map_err(to_case_error_from_context)?;

    if taker_amount != fixture.offered_amount || maker_amount != fixture.wanted_amount {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Token balances did not transfer as expected",
        )) as Box<dyn std::error::Error + Send + Sync>);
    }

    Ok(())
}

pub fn run_offer_checks() -> Result<(), tester::CaseError> {
    let repo_path = get_repo_dir().map_err(to_case_error_from_load)?;
    let mut fixture = SwapFixture::new_default(&repo_path).map_err(to_case_error)?;
    make_offer_success(&mut fixture).map_err(to_case_error)?;
    let offer_account = fixture.get_account(&fixture.offer)?;
    let offer = offer_data_from_account(&offer_account).map_err(to_case_error_from_context)?;

    if offer.id != fixture.offer_id
        || offer.maker != fixture.maker
        || offer.token_mint_a != fixture.token_mint_a
        || offer.token_mint_b != fixture.token_mint_b
        || offer.token_b_wanted_amount != fixture.wanted_amount
    {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Offer account data mismatch",
        )) as Box<dyn std::error::Error + Send + Sync>);
    }

    Ok(())
}

pub fn run_make_offer_checks() -> Result<(), tester::CaseError> {
    let repo_path = get_repo_dir().map_err(to_case_error_from_load)?;
    let mut fixture = SwapFixture::new_default(&repo_path).map_err(to_case_error)?;
    make_offer_success(&mut fixture).map_err(to_case_error)?;

    let maker_token_account = fixture.get_account(&fixture.maker_token_account_a)?;
    let vault_account = fixture.get_account(&fixture.vault)?;

    let maker_amount =
        token_account_amount(&maker_token_account).map_err(to_case_error_from_context)?;
    let vault_amount = token_account_amount(&vault_account).map_err(to_case_error_from_context)?;

    if maker_amount != 0 || vault_amount != fixture.offered_amount {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Make offer transfer did not move tokens to vault",
        )) as Box<dyn std::error::Error + Send + Sync>);
    }

    Ok(())
}

pub fn run_take_offer_checks() -> Result<(), tester::CaseError> {
    run_token_transfer_check()
}

pub fn run_pda_checks() -> Result<(), tester::CaseError> {
    let repo_path = get_repo_dir().map_err(to_case_error_from_load)?;
    let mut fixture = SwapFixture::new_default(&repo_path).map_err(to_case_error)?;
    make_offer_success(&mut fixture).map_err(to_case_error)?;
    let offer_account = fixture.get_account(&fixture.offer)?;
    let offer = offer_data_from_account(&offer_account).map_err(to_case_error_from_context)?;

    let (expected_offer, bump) = Pubkey::find_program_address(
        &[OFFER_SEED_PREFIX, fixture.maker.as_ref(), &fixture.offer_id.to_le_bytes()],
        &fixture.program_id,
    );

    if expected_offer != fixture.offer || offer.bump != bump {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Offer PDA derivation mismatch",
        )) as Box<dyn std::error::Error + Send + Sync>);
    }

    Ok(())
}

pub fn run_vault_checks() -> Result<(), tester::CaseError> {
    let repo_path = get_repo_dir().map_err(to_case_error_from_load)?;
    let mut fixture = SwapFixture::new_default(&repo_path).map_err(to_case_error)?;
    make_offer_success(&mut fixture).map_err(to_case_error)?;

    let vault_account = fixture.get_account(&fixture.vault)?;
    let vault_owner = token_account_owner(&vault_account).map_err(to_case_error_from_context)?;
    let vault_mint = token_account_mint(&vault_account).map_err(to_case_error_from_context)?;

    if vault_owner != fixture.offer || vault_mint != fixture.token_mint_a {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Vault ATA ownership or mint mismatch",
        )) as Box<dyn std::error::Error + Send + Sync>);
    }

    Ok(())
}

pub fn run_security_checks() -> Result<(), tester::CaseError> {
    let repo_path = get_repo_dir().map_err(to_case_error_from_load)?;
    let mut fixture = SwapFixture::new_default(&repo_path).map_err(to_case_error)?;
    make_offer_success(&mut fixture).map_err(to_case_error)?;

    let mut bad_instruction = fixture.take_offer_instruction();
    bad_instruction.accounts[1] = AccountMeta::new(fixture.taker, false);

    match fixture.context.execute_instruction(&bad_instruction) {
        Ok(()) => Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Security check failed: invalid maker accepted",
        )) as Box<dyn std::error::Error + Send + Sync>),
        Err(TestContextError::ExecutionError(_)) => Ok(()),
        Err(err) => Err(to_case_error(err)),
    }
}

pub fn run_error_checks() -> Result<(), tester::CaseError> {
    let repo_path = get_repo_dir().map_err(to_case_error_from_load)?;
    let mut fixture = SwapFixture::new_with_amounts(
        &repo_path,
        DEFAULT_OFFERED_AMOUNT,
        DEFAULT_WANTED_AMOUNT,
        0,
        DEFAULT_WANTED_AMOUNT,
        DEFAULT_MINT_DECIMALS,
    )
    .map_err(to_case_error)?;

    match fixture.execute_make_offer() {
        Ok(()) => Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Expected make_offer to fail with insufficient funds",
        )) as Box<dyn std::error::Error + Send + Sync>),
        Err(TestContextError::ExecutionError(_)) => Ok(()),
        Err(err) => Err(to_case_error(err)),
    }
}

pub fn run_cpi_checks() -> Result<(), tester::CaseError> {
    run_cpi_transfer_check()
}

pub fn run_testing_checks() -> Result<(), tester::CaseError> {
    run_token_transfer_check()
}

pub fn run_deployment_checks() -> Result<(), tester::CaseError> {
    let repo_path = get_repo_dir().map_err(to_case_error_from_load)?;
    let program_id = load_swap_program_id(&repo_path).map_err(to_case_error_from_load)?;
    if program_id == Pubkey::default() {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Program ID is still default",
        )) as Box<dyn std::error::Error + Send + Sync>);
    }
    run_make_offer_smoke(&repo_path)
}

fn run_make_offer_smoke(repo_path: &Path) -> Result<(), tester::CaseError> {
    let mut fixture = SwapFixture::new_default(repo_path).map_err(to_case_error)?;
    match fixture.execute_make_offer() {
        Ok(()) => Ok(()),
        Err(TestContextError::ExecutionError(_)) => Ok(()),
        Err(err) => Err(to_case_error(err)),
    }
}
