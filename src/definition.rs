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

use crate::stages::{
    base::*,
    extensions::{
        cpi::*, deployment::*, error::*, make_offer::*, offer::*, pda::*, security::*,
        take_offer::*, testing::*, vault::*,
    },
};

pub fn build() -> Definition {
    Definition {
        executable_name: "your_program.sh".to_string(),
        legacy_executable_name: Some("your_program.sh".to_string()),
        cases: vec![
            // Base Stages (7 stages)
            Case::new("be1", Arc::new(be1::test_env_setup)),
            Case::new("rs2", Arc::new(rs2::test_rust_basics)),
            Case::new("sm3", Arc::new(sm3::test_solana_model)),
            Case::new("at4", Arc::new(at4::test_anchor_try)),
            Case::new("st5", Arc::new(st5::test_spl_token_basics)),
            Case::new("cp6", Arc::new(cp6::test_cpi_transfer)),
            Case::new("tt7", Arc::new(tt7::test_token_transfer)),
            // Extension Modules (9 modules × 4 stages = 36 cases)
            // PDA Module
            Case::new("pa1", Arc::new(pa1::test_pda_concept)),
            Case::new("pa2", Arc::new(pa2::test_pda_derivation)),
            Case::new("pa3", Arc::new(pa3::test_pda_bump_seeds)),
            Case::new("pa4", Arc::new(pa4::test_pda_practice)),
            // Vault Module
            Case::new("va1", Arc::new(va1::test_vault_intro)),
            Case::new("va2", Arc::new(va2::test_vault_creation)),
            Case::new("va3", Arc::new(va3::test_vault_security)),
            Case::new("va4", Arc::new(va4::test_vault_practice)),
            // Offer Module
            Case::new("of1", Arc::new(of1::test_offer_data_structure)),
            Case::new("of2", Arc::new(of2::test_offer_validation)),
            Case::new("of3", Arc::new(of3::test_offer_pda)),
            Case::new("of4", Arc::new(of4::test_offer_practice)),
            // Make Offer Module
            Case::new("mo1", Arc::new(mo1::test_make_offer_overview)),
            Case::new("mo2", Arc::new(mo2::test_deposit_tokens)),
            Case::new("mo3", Arc::new(mo3::test_save_offer)),
            Case::new("mo4", Arc::new(mo4::test_make_offer_practice)),
            // Take Offer Module
            Case::new("to1", Arc::new(to1::test_take_offer_overview)),
            Case::new("to2", Arc::new(to2::test_receive_tokens)),
            Case::new("to3", Arc::new(to3::test_withdraw_vault)),
            Case::new("to4", Arc::new(to4::test_take_offer_practice)),
            // Security Module
            Case::new("se1", Arc::new(se1::test_common_vulnerabilities)),
            Case::new("se2", Arc::new(se2::test_reentrancy_protection)),
            Case::new("se3", Arc::new(se3::test_account_validation)),
            Case::new("se4", Arc::new(se4::test_security_practice)),
            // CPI Module
            Case::new("cp1", Arc::new(cp1::test_cpi_concept)),
            Case::new("cp2", Arc::new(cp2::test_transfer_checked)),
            Case::new("cp3", Arc::new(cp3::test_cpi_signer)),
            Case::new("cp4", Arc::new(cp4::test_cpi_practice)),
            // Error Module
            Case::new("er1", Arc::new(er1::test_error_basics)),
            Case::new("er2", Arc::new(er2::test_custom_errors)),
            Case::new("er3", Arc::new(er3::test_error_messages)),
            Case::new("er4", Arc::new(er4::test_error_practice)),
            // Testing Module
            Case::new("te1", Arc::new(te1::test_rust_test_basics)),
            Case::new("te2", Arc::new(te2::test_anchor_test_attribute)),
            Case::new("te3", Arc::new(te3::test_testing_setup_teardown)),
            Case::new("te4", Arc::new(te4::test_comprehensive_tests)),
            // Deployment Module
            Case::new("de1", Arc::new(de1::test_local_testing)),
            Case::new("de2", Arc::new(de2::test_devnet_deploy)),
            Case::new("de3", Arc::new(de3::test_mainnet_considerations)),
            Case::new("de4", Arc::new(de4::test_deployment_practice)),
        ],
        ..Default::default()
    }
}
