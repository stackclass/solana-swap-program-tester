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

use crate::verifier::get_program_info;

pub fn test_error_messages(_harness: &tester::Harness) -> Result<(), tester::CaseError> {
    let info = get_program_info()?;

    let has_messages = info.errors.iter().any(|err| !err.message.is_empty());
    if has_messages {
        Ok(())
    } else {
        Err(Box::new(std::io::Error::other("Error messages not found".to_string())))
    }
}
