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

use serde::{Deserialize, Serialize};
use std::{
    fmt,
    path::PathBuf,
    process::{Command, Stdio},
};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ProgramInfo {
    pub program_id: String,
    pub instructions: Vec<InstructionInfo>,
    pub accounts: Vec<AccountInfo>,
    pub errors: Vec<ErrorInfo>,
    pub structs: Vec<StructInfo>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct InstructionInfo {
    pub name: String,
    pub arguments: Vec<ArgumentInfo>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ArgumentInfo {
    pub name: String,
    pub type_name: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AccountInfo {
    pub name: String,
    pub fields: Vec<FieldInfo>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FieldInfo {
    pub name: String,
    pub type_name: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ErrorInfo {
    pub name: String,
    pub code: u32,
    pub message: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct StructInfo {
    pub name: String,
    pub fields: Vec<FieldInfo>,
}

#[derive(Debug)]
struct VerificationError {
    message: String,
}

impl fmt::Display for VerificationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for VerificationError {}

pub fn get_program_info() -> Result<ProgramInfo, tester::CaseError> {
    let repository_dir = std::env::var("STACKCLASS_REPOSITORY_DIR").map_err(|_| {
        Box::new(VerificationError { message: "STACKCLASS_REPOSITORY_DIR not set".to_string() })
    })?;

    let executable_path = PathBuf::from(&repository_dir).join("your_program.sh");

    // 运行 dump_info 命令
    let mut cmd = Command::new(&executable_path);
    cmd.arg("dump_info");

    let output = cmd
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| {
            Box::new(VerificationError { message: format!("Failed to get program info: {}", e) })
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(Box::new(VerificationError {
            message: format!("Failed to run dump_info: {}", stderr),
        }));
    }

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let program_info: ProgramInfo = serde_json::from_str(&stdout).map_err(|e| {
        Box::new(VerificationError { message: format!("Failed to parse JSON: {} - {}", e, stdout) })
    })?;

    Ok(program_info)
}
