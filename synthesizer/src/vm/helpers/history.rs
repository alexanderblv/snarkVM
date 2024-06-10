// Copyright (C) 2019-2023 Aleo Systems Inc.
// This file is part of the snarkVM library.

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at:
// http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use console::prelude::{Deserialize, Serialize};

use aleo_std::{aleo_ledger_dir, StorageMode};

use anyhow::Result;
use serde_json;
use std::{
    fmt::{Display, Formatter},
    path::PathBuf,
};

/// Returns the path where a `history` directory may be stored.
pub fn history_directory_path(network: u16, storage_mode: StorageMode) -> PathBuf {
    const HISTORY_DIRECTORY_NAME: &str = "history";

    // Create the name of the history directory.
    let directory_name = match &storage_mode {
        StorageMode::Development(id) => format!(".{HISTORY_DIRECTORY_NAME}-{network}-{id}"),
        StorageMode::Production | StorageMode::Custom(_) => format!("{HISTORY_DIRECTORY_NAME}-{network}"),
    };

    // Obtain the path to the ledger.
    let mut path = aleo_ledger_dir(network, storage_mode);
    // Go to the folder right above the ledger.
    path.pop();
    // Append the history directory's name.
    path.push(directory_name);

    path
}

#[derive(Copy, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MappingName {
    /// The `bonded` mapping.
    Bonded,
    /// The `delegated` mapping.
    Delegated,
    /// The `unbonding` mapping.
    Unbonding,
}

impl Display for MappingName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bonded => write!(f, "bonded"),
            Self::Delegated => write!(f, "delegated"),
            Self::Unbonding => write!(f, "unbonding"),
        }
    }
}

pub struct History {
    /// The path to the history directory.
    path: PathBuf,
}

impl History {
    /// Initializes a new instance of `History`.
    pub fn new(network: u16, storage_mode: StorageMode) -> Self {
        Self { path: history_directory_path(network, storage_mode) }
    }

    /// Stores a mapping from a given block in the history directory as JSON.
    pub fn store_mapping<T>(&self, height: u32, mapping: MappingName, data: &T) -> Result<()>
    where
        T: Serialize + ?Sized,
    {
        // Get the path to the block directory.
        let block_path = self.path.join(format!("block-{height}"));
        // Create the block directory if it does not exist.
        if !block_path.exists() {
            std::fs::create_dir_all(&block_path)?;
        }

        // Write the entry to the block directory.
        let entry_path = block_path.join(format!("block-{height}-{mapping}.json"));
        std::fs::write(entry_path, serde_json::to_string_pretty(data)?)?;

        Ok(())
    }

    /// Loads the JSON string for a mapping from a given block from the history directory.
    pub fn load_mapping(&self, height: u32, mapping: MappingName) -> Result<String> {
        // Get the path to the block directory.
        let block_path = self.path.join(format!("block-{height}"));
        // Get the path to the entry.
        let entry_path = block_path.join(format!("block-{height}-{mapping}.json"));
        // Load the entry.
        let result = std::fs::read_to_string(entry_path)?;

        Ok(result)
    }
}