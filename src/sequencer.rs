// Copyright 2018 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::MAX_FILE_SIZE;
use memmap::{Mmap, Protection};
use std::{
    io::{Error as IoError, Write},
    ops::{Deref, DerefMut, Index, IndexMut},
};

pub const MAX_IN_MEMORY_SIZE: usize = 50 * 1024 * 1024;

enum Data {
    Vector(Vec<u8>),
    Mmap(Mmap),
}

/// Optionally create a sequence of bytes via a vector or memory map.
pub struct Sequencer {
    data: Data,
}

#[allow(clippy::len_without_is_empty)]
impl Sequencer {
    /// Initialise as a vector.
    pub fn new_as_vector() -> Sequencer {
        Sequencer {
            data: Data::Vector(Vec::with_capacity(MAX_IN_MEMORY_SIZE)),
        }
    }

    /// Initialise as a memory map
    pub fn new_as_mmap() -> Result<Sequencer, IoError> {
        Ok(Sequencer {
            data: Data::Mmap(Mmap::anonymous(MAX_FILE_SIZE, Protection::ReadWrite)?),
        })
    }

    /// Return the current length of the sequencer.
    pub fn len(&self) -> usize {
        match self.data {
            Data::Vector(ref vector) => vector.len(),
            Data::Mmap(ref mmap) => mmap.len(),
        }
    }

    #[allow(unsafe_code)]
    /// Initialise with the Sequencer with 'content'.
    pub fn init(&mut self, content: &[u8]) {
        match self.data {
            Data::Vector(ref mut vector) => vector.extend_from_slice(content),
            Data::Mmap(ref mut mmap) => {
                let _ = unsafe { mmap.as_mut_slice() }.write_all(&content[..]);
            }
        }
    }

    /// Truncate internal object to given size. Note that this affects the vector only since the
    /// memory map is a fixed size.
    pub fn truncate(&mut self, size: usize) {
        if let Data::Vector(ref mut vector) = self.data {
            vector.truncate(size);
        }
    }

    #[allow(unsafe_code)]
    /// Create a memory map if we haven't already done so.
    pub fn create_mapping(&mut self) -> Result<(), IoError> {
        self.data = match self.data {
            Data::Mmap(_) => return Ok(()),
            Data::Vector(ref mut vector) => {
                let mut mmap = Mmap::anonymous(MAX_FILE_SIZE, Protection::ReadWrite)?;
                let _ = unsafe { mmap.as_mut_slice() }.write_all(&vector[..]);
                Data::Mmap(mmap)
            }
        };
        Ok(())
    }
}

#[allow(unsafe_code)]
impl Index<usize> for Sequencer {
    type Output = u8;
    fn index(&self, index: usize) -> &u8 {
        match self.data {
            Data::Vector(ref vector) => &vector[index],
            Data::Mmap(ref mmap) => unsafe { &mmap.as_slice()[index] },
        }
    }
}

#[allow(unsafe_code)]
impl IndexMut<usize> for Sequencer {
    fn index_mut(&mut self, index: usize) -> &mut u8 {
        match self.data {
            Data::Vector(ref mut vector) => &mut vector[index],
            Data::Mmap(ref mut mmap) => unsafe { &mut mmap.as_mut_slice()[index] },
        }
    }
}

#[allow(unsafe_code)]
impl Deref for Sequencer {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        match self.data {
            Data::Vector(ref vector) => &*vector,
            Data::Mmap(ref mmap) => unsafe { mmap.as_slice() },
        }
    }
}

#[allow(unsafe_code)]
impl DerefMut for Sequencer {
    fn deref_mut(&mut self) -> &mut [u8] {
        match self.data {
            Data::Vector(ref mut vector) => &mut *vector,
            Data::Mmap(ref mut mmap) => unsafe { &mut *mmap.as_mut_slice() },
        }
    }
}

impl Extend<u8> for Sequencer {
    fn extend<I>(&mut self, iterable: I)
    where
        I: IntoIterator<Item = u8>,
    {
        if let Data::Vector(ref mut vector) = self.data {
            vector.extend(iterable);
        }
    }
}
