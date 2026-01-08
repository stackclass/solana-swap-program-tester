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

//! Test context module for managing state during testing.

use mollusk_svm::{
    Mollusk,
    result::{Check, InstructionResult},
};
use solana_account::Account;
use solana_instruction::Instruction;
use solana_instruction_error::InstructionError;
use solana_pubkey::Pubkey;
use std::collections::HashMap;

/// Error type for test context operations.
#[derive(Debug)]
pub enum TestContextError {
    ExecutionError(String),
    ValidationError(String),
    AccountNotFound(String),
}

impl std::fmt::Display for TestContextError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TestContextError::ExecutionError(msg) => {
                write!(f, "Instruction execution failed: {}", msg)
            }
            TestContextError::ValidationError(msg) => write!(f, "Validation failed: {}", msg),
            TestContextError::AccountNotFound(msg) => write!(f, "Account not found: {}", msg),
        }
    }
}

impl std::error::Error for TestContextError {}

impl From<InstructionError> for TestContextError {
    fn from(err: InstructionError) -> Self {
        TestContextError::ExecutionError(format!("{:?}", err))
    }
}

impl From<crate::mollusk::ProgramLoadError> for TestContextError {
    fn from(err: crate::mollusk::ProgramLoadError) -> Self {
        TestContextError::ExecutionError(err.to_string())
    }
}

/// A test context for the swap program.
///
/// This struct manages the state of accounts during testing and provides
/// convenience methods for executing and validating instructions.
pub struct SwapTestContext {
    /// The Mollusk test harness.
    mollusk: Mollusk,
    /// The current state of accounts.
    accounts: HashMap<Pubkey, Account>,
    /// The program ID being tested.
    program_id: Pubkey,
}

impl SwapTestContext {
    /// Create a new test context.
    ///
    /// # Arguments
    ///
    /// * `mollusk` - The Mollusk test harness
    /// * `program_id` - The swap program ID
    ///
    /// # Returns
    ///
    /// * `Ok(SwapTestContext)` - A new test context
    pub fn new(mollusk: Mollusk, program_id: Pubkey) -> Result<Self, TestContextError> {
        Ok(Self { mollusk, accounts: HashMap::new(), program_id })
    }

    /// Get the program ID.
    pub fn program_id(&self) -> Pubkey {
        self.program_id
    }

    /// Add an account to the test context.
    ///
    /// # Arguments
    ///
    /// * `pubkey` - The account's public key
    /// * `account` - The account data
    pub fn add_account(&mut self, pubkey: Pubkey, account: Account) {
        self.accounts.insert(pubkey, account);
    }

    /// Get an account from the test context.
    ///
    /// # Arguments
    ///
    /// * `pubkey` - The account's public key
    ///
    /// # Returns
    ///
    /// * `Some(Account)` - The account data if it exists
    /// * `None` - If the account does not exist
    pub fn get_account(&self, pubkey: &Pubkey) -> Option<Account> {
        self.accounts.get(pubkey).cloned()
    }

    /// Execute an instruction and update the account state.
    ///
    /// # Arguments
    ///
    /// * `instruction` - The instruction to execute
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the instruction executed successfully
    /// * `Err(TestContextError)` - If execution failed
    pub fn execute_instruction(
        &mut self,
        instruction: &Instruction,
    ) -> Result<(), TestContextError> {
        let result: InstructionResult =
            self.mollusk.process_instruction(instruction, &self.get_account_list());

        // Check if execution was successful
        if result.program_result.is_err() {
            return Err(TestContextError::ExecutionError(format!("{:?}", result.program_result)));
        }

        // Update account state from the result
        for (pubkey, account) in result.resulting_accounts {
            self.accounts.insert(pubkey, account);
        }

        Ok(())
    }

    /// Execute an instruction and validate the result.
    ///
    /// # Arguments
    ///
    /// * `instruction` - The instruction to execute
    /// * `checks` - The checks to apply to the result
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the instruction executed and all checks passed
    /// * `Err(TestContextError)` - If execution or validation failed
    #[allow(dead_code)]
    pub fn execute_and_validate(
        &mut self,
        instruction: &Instruction,
        checks: &[Check],
    ) -> Result<(), TestContextError> {
        let result: InstructionResult = self.mollusk.process_and_validate_instruction(
            instruction,
            &self.get_account_list(),
            checks,
        );

        // Check if execution was successful
        if result.program_result.is_err() {
            return Err(TestContextError::ExecutionError(format!("{:?}", result.program_result)));
        }

        // Update account state from the result
        for (pubkey, account) in result.resulting_accounts {
            self.accounts.insert(pubkey, account);
        }

        Ok(())
    }

    /// Get the current account list for Mollusk.
    fn get_account_list(&self) -> Vec<(Pubkey, Account)> {
        self.accounts.iter().map(|(pubkey, account)| (*pubkey, account.clone())).collect()
    }

    /// Create a new keypair and add a funded account to the context.
    ///
    /// This is a convenience method for creating user accounts with
    /// initial lamports.
    ///
    /// # Arguments
    ///
    /// * `lamports` - Initial lamports to fund the account with
    ///
    /// # Returns
    ///
    /// * `Pubkey` - The public key of the new account
    pub fn create_funded_account(&mut self, lamports: u64) -> Pubkey {
        let pubkey = Pubkey::new_unique();
        let account =
            Account { lamports, owner: solana_system_program::id(), ..Default::default() };
        self.add_account(pubkey, account);
        pubkey
    }

    /// Create a token account.
    ///
    /// This is a convenience method for creating token accounts.
    ///
    /// # Arguments
    ///
    /// * `owner` - The owner of the token account
    /// * `mint` - The mint address
    /// * `amount` - Initial token amount
    ///
    /// # Returns
    ///
    /// * `Pubkey` - The public key of the new token account
    #[allow(dead_code)]
    pub fn create_token_account(&mut self, owner: Pubkey, mint: Pubkey, amount: u64) -> Pubkey {
        let pubkey = Pubkey::new_unique();
        let mut data = vec![0; 82]; // Token account size

        // Set owner (offset 32)
        data[32..64].copy_from_slice(owner.as_ref());

        // Set mint (offset 0)
        data[0..32].copy_from_slice(mint.as_ref());

        // Set amount (offset 64, little-endian u64)
        data[64..72].copy_from_slice(&amount.to_le_bytes());

        let account = Account {
            lamports: solana_rent::Rent::default().minimum_balance(data.len()),
            data,
            owner: spl_token_interface::ID,
            ..Default::default()
        };
        self.add_account(pubkey, account);
        pubkey
    }
}

impl Default for SwapTestContext {
    fn default() -> Self {
        Self {
            mollusk: Mollusk::default(),
            accounts: HashMap::new(),
            program_id: Pubkey::new_unique(),
        }
    }
}
