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

use std::sync::Arc;

use tester::{Case, Definition};

use crate::stages::{advanced_features::*, base::*};

pub fn build() -> Definition {
    Definition {
        executable_name: "your_program.sh".to_string(),
        legacy_executable_name: Some("your_program.sh".to_string()),
        cases: vec![
            // Base Stages
            Case::new("ry1", Arc::new(ry1::test_declare_id)),
            Case::new("of2", Arc::new(of2::test_offer)),
            Case::new("mo3", Arc::new(mo3::test_makeoffer_context)),
            Case::new("pd4", Arc::new(pd4::test_pda_init)),
            Case::new("tt5", Arc::new(tt5::test_token_transfer)),
            Case::new("mf6", Arc::new(mf6::test_makeoffer_function)),
            Case::new("to7", Arc::new(to7::test_takeoffer_context)),
            Case::new("er8", Arc::new(er8::test_error_codes)),
            Case::new("tf9", Arc::new(tf9::test_takeoffer_function)),
            Case::new("td10", Arc::new(td10::test_testing_deploy)),
            // Advanced Features
            Case::new("te1", Arc::new(te1::test_time_expiration)),
            Case::new("pf2", Arc::new(pf2::test_partial_fills)),
        ],
        ..Default::default()
    }
}
