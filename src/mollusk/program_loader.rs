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

//! Program loader module for loading the swap program from disk.

use mollusk_svm::file;
use solana_pubkey::Pubkey;
use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

/// Error type for program loading operations.
#[derive(Debug)]
pub enum ProgramLoadError {
    RepoNotFound(PathBuf),
    AnchorTomlNotFound(PathBuf),
    ProgramIdNotFound,
    InvalidProgramId(String),
    ProgramDirNotFound(PathBuf),
    ProgramNotFound,
    IoError(std::io::Error),
    #[allow(dead_code)]
    ElfLoadError(String),
}

impl std::fmt::Display for ProgramLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProgramLoadError::RepoNotFound(path) => {
                write!(f, "Repository directory not found: {}", path.display())
            }
            ProgramLoadError::AnchorTomlNotFound(path) => {
                write!(f, "Anchor.toml not found: {}", path.display())
            }
            ProgramLoadError::ProgramIdNotFound => {
                write!(f, "Program ID not found in Anchor.toml")
            }
            ProgramLoadError::InvalidProgramId(value) => {
                write!(f, "Invalid program ID in Anchor.toml: {}", value)
            }
            ProgramLoadError::ProgramDirNotFound(path) => {
                write!(f, "Program directory not found: {}", path.display())
            }
            ProgramLoadError::ProgramNotFound => {
                write!(f, "Program SO file not found in any of the expected locations")
            }
            ProgramLoadError::IoError(err) => write!(f, "Failed to read program file: {}", err),
            ProgramLoadError::ElfLoadError(msg) => write!(f, "Failed to load program ELF: {}", msg),
        }
    }
}

impl std::error::Error for ProgramLoadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ProgramLoadError::IoError(err) => Some(err),
            _ => None,
        }
    }
}

impl From<std::io::Error> for ProgramLoadError {
    fn from(err: std::io::Error) -> Self {
        ProgramLoadError::IoError(err)
    }
}

/// Load the swap program from the user's repository directory.
///
/// This function searches for the compiled program SO file in the following
/// locations (in order):
///
/// 1. `repo_dir/target/deploy/swap.so`
/// 2. `repo_dir/target/sbf-solana-solana/release/swap.so`
/// 3. `repo_dir/artifacts/swap.so`
///
/// # Arguments
///
/// * `repo_dir` - Path to the user's repository directory
///
/// # Returns
///
/// * `Ok(PathBuf)` - Path to the program SO file
/// * `Err(ProgramLoadError)` - If the program cannot be found or loaded
pub fn load_swap_program(repo_dir: &Path) -> Result<PathBuf, ProgramLoadError> {
    if !repo_dir.exists() {
        return Err(ProgramLoadError::RepoNotFound(repo_dir.to_path_buf()));
    }

    // Try standard Anchor deployment path
    let deploy_path = repo_dir.join("target/deploy/swap.so");
    if deploy_path.exists() {
        return Ok(deploy_path);
    }

    // Try SBF release path
    let sbf_path = repo_dir.join("target/sbf-solana-solana/release/swap.so");
    if sbf_path.exists() {
        return Ok(sbf_path);
    }

    // Try artifacts directory
    let artifacts_path = repo_dir.join("artifacts/swap.so");
    if artifacts_path.exists() {
        return Ok(artifacts_path);
    }

    // Try to find any .so file in the target directory
    if let Some(so_file) = find_so_file_in_target(repo_dir) {
        return Ok(so_file);
    }

    Err(ProgramLoadError::ProgramNotFound)
}

/// Load the swap program ID from Anchor.toml.
///
/// This function attempts to parse the program ID from the `programs.*`
/// section in Anchor.toml.
///
/// # Arguments
///
/// * `repo_dir` - Path to the user's repository directory
///
/// # Returns
///
/// * `Ok(Pubkey)` - The program ID
/// * `Err(ProgramLoadError)` - If the program ID cannot be found or parsed
pub fn load_swap_program_id(repo_dir: &Path) -> Result<Pubkey, ProgramLoadError> {
    if !repo_dir.exists() {
        return Err(ProgramLoadError::RepoNotFound(repo_dir.to_path_buf()));
    }

    let anchor_path = repo_dir.join("Anchor.toml");
    if !anchor_path.exists() {
        return Err(ProgramLoadError::AnchorTomlNotFound(anchor_path));
    }

    let content = std::fs::read_to_string(&anchor_path)?;
    let program_id =
        find_program_id(&content, "swap").ok_or(ProgramLoadError::ProgramIdNotFound)?;

    Pubkey::from_str(&program_id).map_err(|_| ProgramLoadError::InvalidProgramId(program_id))
}

fn find_program_id(toml: &str, program_name: &str) -> Option<String> {
    let mut in_programs_section = false;

    for raw_line in toml.lines() {
        let line = raw_line.trim();

        if line.starts_with('[') && line.ends_with(']') {
            let section = &line[1..line.len() - 1];
            in_programs_section = section == "programs" || section.starts_with("programs.");
            continue;
        }

        if !in_programs_section || line.is_empty() || line.starts_with('#') {
            continue;
        }

        if let Some((key, value)) = line.split_once('=') &&
            key.trim() == program_name
        {
            let value = value.trim().trim_matches('"');
            if !value.is_empty() {
                return Some(value.to_string());
            }
        }
    }

    None
}

/// Search for any .so file in the target directory.
fn find_so_file_in_target(repo_dir: &Path) -> Option<PathBuf> {
    let target_dir = repo_dir.join("target");
    if !target_dir.exists() {
        return None;
    }

    // Search recursively for .so files
    let mut found = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&target_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Some(found_in_subdir) = find_so_file_recursive(&path) {
                    found.push(found_in_subdir);
                }
            } else if path.extension().is_some_and(|ext| ext == "so") {
                found.push(path);
            }
        }
    }

    // Return the first found .so file
    found.into_iter().next()
}

/// Recursively search for .so files in a directory.
fn find_so_file_recursive(dir: &Path) -> Option<PathBuf> {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Some(found) = find_so_file_recursive(&path) {
                    return Some(found);
                }
            } else if path.extension().is_some_and(|ext| ext == "so") {
                return Some(path);
            }
        }
    }
    None
}

/// Load the program ELF bytes from a file path.
///
/// This is a wrapper around Mollusk's file loading functionality that
/// provides better error messages.
///
/// # Arguments
///
/// * `path` - Path to the program SO file
///
/// # Returns
///
/// * `Ok(Vec<u8>)` - The program ELF bytes
/// * `Err(ProgramLoadError)` - If the file cannot be read
#[allow(dead_code)]
pub fn load_program_elf(path: &Path) -> Result<Vec<u8>, ProgramLoadError> {
    let elf = file::load_program_elf(path.to_str().unwrap());
    Ok(elf)
}
