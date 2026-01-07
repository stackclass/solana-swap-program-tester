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

//! Mollusk management module for the swap program tester.
//!
//! This module provides a wrapper around the Mollusk test harness to simplify
//! testing of the swap program. It handles program loading, account setup,
//! and instruction execution.

pub mod program_loader;
pub mod test_context;

pub use program_loader::{ProgramLoadError, load_swap_program, load_swap_program_id};
pub use test_context::{SwapTestContext, TestContextError};

use mollusk_svm::Mollusk;
use solana_pubkey::Pubkey;
use std::path::Path;

/// Create a new Mollusk instance for testing the swap program.
///
/// This function attempts to load the compiled swap program from the
/// user's repository directory and creates a Mollusk instance configured
/// for testing.
///
/// # Arguments
///
/// * `repo_dir` - Path to the user's repository directory
/// * `program_id` - The swap program ID
///
/// # Returns
///
/// * `Ok(Mollusk)` - A configured Mollusk instance
/// * `Err(ProgramLoadError)` - If the program cannot be loaded
pub fn create_swap_mollusk(
    repo_dir: &Path,
    program_id: &Pubkey,
) -> Result<Mollusk, ProgramLoadError> {
    let program_path = load_swap_program(repo_dir)?;
    let program_name = program_path.file_stem().and_then(|stem| stem.to_str()).unwrap_or("swap");

    let program_dir = program_path
        .parent()
        .ok_or_else(|| ProgramLoadError::ProgramDirNotFound(program_path.clone()))?;

    // SAFETY: set_var is process-global; we set it once before loading the ELF.
    unsafe {
        std::env::set_var("SBF_OUT_DIR", program_dir);
    }

    let mut mollusk = Mollusk::new(program_id, program_name);

    // Add necessary programs for testing
    add_required_programs(&mut mollusk);

    Ok(mollusk)
}

/// Add required programs to the Mollusk instance.
///
/// This includes system programs and SPL Token programs that are commonly
/// used in swap operations.
fn add_required_programs(mollusk: &mut Mollusk) {
    // System program is already included by default in Mollusk

    // SPL Token program and Associated Token program - needed for token operations
    mollusk_svm_programs_token::token::add_program(mollusk);
    mollusk_svm_programs_token::associated_token::add_program(mollusk);
}

/// Initialize a test context with the swap program.
///
/// This is a convenience function that creates both a Mollusk instance and
/// a test context for easier testing.
///
/// # Arguments
///
/// * `repo_dir` - Path to the user's repository directory
///
/// # Returns
///
/// * `Ok(SwapTestContext)` - A configured test context
/// * `Err(TestContextError)` - If initialization fails
pub fn init_test_context(repo_dir: &Path) -> Result<SwapTestContext, TestContextError> {
    let program_id = load_swap_program_id(repo_dir)?;
    let mollusk = create_swap_mollusk(repo_dir, &program_id)?;
    SwapTestContext::new(mollusk, program_id)
}
