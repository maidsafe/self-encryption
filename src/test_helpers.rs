// Copyright 2016 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under (1) the MaidSafe.net Commercial License,
// version 1.0 or later, or (2) The General Public License (GPL), version 3, depending on which
// licence you accepted on initial access to the Software (the "Licences").
//
// By contributing code to the SAFE Network Software, or to this project generally, you agree to be
// bound by the terms of the MaidSafe Contributor Agreement, version 1.1.  This, along with the
// Licenses can be found in the root directory of this project at LICENSE, COPYING and CONTRIBUTOR.
//
// Unless required by applicable law or agreed to in writing, the Safe Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.
//
// Please review the Licences for the specific language governing permissions and limitations
// relating to use of the SAFE Network Software.

#![doc(hidden)]

use std::error::Error;
use std::fmt::{self, Display, Formatter};

use super::{Storage, StorageError};

#[derive(Debug, Clone)]
pub struct SimpleStorageError {}

impl Display for SimpleStorageError {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "Failed to get data from SimpleStorage")
    }
}

impl Error for SimpleStorageError {
    fn description(&self) -> &str {
        "SimpleStorage::get() error"
    }
}

impl StorageError for SimpleStorageError {}



struct Entry {
    name: Vec<u8>,
    data: Vec<u8>,
}


#[derive(Default)]
pub struct SimpleStorage {
    entries: Vec<Entry>,
}

impl SimpleStorage {
    pub fn new() -> SimpleStorage {
        SimpleStorage { entries: vec![] }
    }

    pub fn has_chunk(&self, name: &[u8]) -> bool {
        self.entries.iter().any(|ref entry| entry.name == name)
    }

    pub fn num_entries(&self) -> usize {
        self.entries.len()
    }
}

impl Storage<SimpleStorageError> for SimpleStorage {
    fn get(&self, name: &[u8]) -> Result<Vec<u8>, SimpleStorageError> {
        match self.entries.iter().find(|ref entry| entry.name == name) {
            Some(entry) => Ok(entry.data.clone()),
            None => Err(SimpleStorageError {}),
        }
    }

    fn put(&mut self, name: Vec<u8>, data: Vec<u8>) -> Result<(), SimpleStorageError> {
        Ok(self.entries.push(Entry {
            name: name,
            data: data,
        }))
    }
}
