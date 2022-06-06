// Copyright 2022 SmallWorld Selendra.
// This file is part of Selendra.

// Selendra is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Selendra is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Selendra.  If not, see <http://www.gnu.org/licenses/>.

use sc_chain_spec::ChainType;
// use serde_json::map::Map;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::sr25519;
use sp_runtime::traits::Zero;

use crate::chain_spec::{
	get_account_id_from_seed, get_parachain_authority_keys_from_seed, Extensions,
};

use bitriel_primitives::{AccountId, Balance, BlockNumber};
use rieltest_runtime::{
	constants::currency::UNIT, BalancesConfig, CollatorSelectionConfig, ParachainInfoConfig,
	SelendraXcmConfig, SessionConfig, SessionKeys, SystemConfig,
};

pub type ChainSpec = sc_service::GenericChainSpec<rieltest_runtime::GenesisConfig, Extensions>;

pub const PARA_ID: u32 = 2000; // TODO: need confirm

pub fn rieltest_config() -> Result<ChainSpec, String> {
	ChainSpec::from_json_bytes(&include_bytes!("../../../resources/rieltest.json")[..])
}

// fn rieltest_properties() -> Properties {
// 	let mut properties = Map::new();
// 	properties.insert("tokenSymbol".into(), 42.into());
// 	properties.insert("tokenDecimals".into(), "RTT".into());
// 	properties.insert("ss58Format".into(), 18.into());
// 	properties
// }

pub fn rieltest_dev_config() -> Result<ChainSpec, String> {
	let wasm_binary = rieltest_runtime::WASM_BINARY.unwrap_or_default();

	Ok(ChainSpec::from_genesis(
		"Rieltest Dev",
		"rieltest-dev",
		ChainType::Development,
		move || {
			rieltest_genesis(
				wasm_binary,
				// Initial PoA authorities
				vec![get_parachain_authority_keys_from_seed("Alice")],
				// Sudo account
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				vec![
					(get_account_id_from_seed::<sr25519::Public>("Alice"), 1000 * UNIT),
					(get_account_id_from_seed::<sr25519::Public>("Bob"), 1000 * UNIT),
					(get_account_id_from_seed::<sr25519::Public>("Charlie"), 1000 * UNIT),
				],
				vec![],
				vec![get_account_id_from_seed::<sr25519::Public>("Alice")],
			)
		},
		vec![],
		None,
		None,
		None,
		Some(
			serde_json::from_str(
				"{
            \"tokenDecimals\": 12,
            \"tokenSymbol\": \"BTT\"
        	}",
			)
			.expect("Provided valid json map"),
		),
		Extensions { relay_chain: "cardamom-local".into(), para_id: PARA_ID, bad_blocks: None },
	))
}

pub fn rieltest_staging_config() -> Result<ChainSpec, String> {
	let wasm_binary = bitriel_runtime::WASM_BINARY.unwrap_or_default();

	Ok(ChainSpec::from_genesis(
		"RielTest Dev",
		"rielTest-dev",
		ChainType::Development,
		move || {
			rieltest_genesis(
				wasm_binary,
				// Initial PoA authorities
				vec![
					get_parachain_authority_keys_from_seed("Alice"),
					get_parachain_authority_keys_from_seed("Bob"),
				],
				// Sudo account
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				vec![
					(get_account_id_from_seed::<sr25519::Public>("Alice"), 1000 * UNIT),
					(get_account_id_from_seed::<sr25519::Public>("Bob"), 1000 * UNIT),
					(get_account_id_from_seed::<sr25519::Public>("Charlie"), 1000 * UNIT),
				],
				vec![],
				vec![get_account_id_from_seed::<sr25519::Public>("Alice")],
			)
		},
		vec![],
		None,
		None,
		None,
		Some(
			serde_json::from_str(
				"{
            \"tokenDecimals\": 12,
            \"tokenSymbol\": \"BTT\"
        	}",
			)
			.expect("Provided valid json map"),
		),
		Extensions { relay_chain: "cardamom".into(), para_id: PARA_ID, bad_blocks: None },
	))
}

fn rieltest_genesis(
	wasm_binary: &[u8],
	initial_authorities: Vec<(AccountId, AuraId)>,
	_root_key: AccountId,
	initial_allocation: Vec<(AccountId, Balance)>,
	_vesting_list: Vec<(AccountId, BlockNumber, BlockNumber, u32, Balance)>,
	_general_councils: Vec<AccountId>,
) -> rieltest_runtime::GenesisConfig {
	rieltest_runtime::GenesisConfig {
		system: SystemConfig {
			// Add Wasm runtime to storage.
			code: wasm_binary.to_vec(),
		},
		balances: BalancesConfig { balances: initial_allocation },
		parachain_info: ParachainInfoConfig { parachain_id: PARA_ID.into() },
		session: SessionConfig {
			keys: initial_authorities
				.iter()
				.cloned()
				.map(|(acc, aura)| {
					(
						acc.clone(),          // account id
						acc,                  // validator id
						SessionKeys { aura }, // session keys
					)
				})
				.collect(),
		},
		collator_selection: CollatorSelectionConfig {
			invulnerables: initial_authorities.iter().cloned().map(|(acc, _)| acc).collect(),
			candidacy_bond: Zero::zero(),
			..Default::default()
		},
		aura: Default::default(),
		aura_ext: Default::default(),
		parachain_system: Default::default(),
		selendra_xcm: SelendraXcmConfig { safe_xcm_version: Some(2) },
	}
}
