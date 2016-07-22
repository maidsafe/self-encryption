// Copyright 2015 MaidSafe.net limited.
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

use std::io::Error as IoError;
use std::io::ErrorKind as IoErrorKind;
use std::io::Write;
use std::ops::{Deref, DerefMut, Index, IndexMut};

use memmap::{Mmap, Protection};
use super::MAX_FILE_SIZE;

pub const MAX_IN_MEMORY_SIZE: usize = 50 * 1024 * 1024;

/// Optionally create a sequence of bytes via a vector or memory map.
pub struct Sequencer {
    vector: Option<Vec<u8>>,
    mmap: Option<Mmap>,
}

#[cfg_attr(feature="clippy", allow(len_without_is_empty))]
impl Sequencer {
    /// Initialise as a vector.
    pub fn new_as_vector() -> Sequencer {
        Sequencer {
            vector: Some(Vec::with_capacity(MAX_IN_MEMORY_SIZE)),
            mmap: None,
        }
    }

    /// Initialise as a memory map
    pub fn new_as_mmap() -> Result<Sequencer, IoError> {
        Ok(Sequencer {
            vector: None,
            mmap: Some(try!(Mmap::anonymous(MAX_FILE_SIZE, Protection::ReadWrite))),
        })
    }

    /// Return the current length of the sequencer.
    pub fn len(&self) -> usize {
        match self.vector {
            Some(ref vector) => vector.len(),
            None => {
                match self.mmap {
                    Some(ref mmap) => mmap.len(),
                    None => 0usize,
                }
            }
        }
    }

    #[allow(unsafe_code)]
    /// Initialise with the Sequencer with 'content'.
    pub fn init(&mut self, content: &[u8]) {
        match self.vector {
            Some(ref mut vector) => {
                for ch in content {
                    vector.push(*ch);
                }
            }
            None => {
                if let Some(ref mut mmap) = self.mmap {
                    let _ = unsafe { mmap.as_mut_slice() }.write_all(&content[..]);
                }
            }
        }
    }

    /// Truncate internal object to given size. Note that this affects the vector only since the
    /// memory map is a fixed size.
    pub fn truncate(&mut self, size: usize) {
        if let Some(ref mut vector) = self.vector {
            vector.truncate(size);
        }
    }

    #[allow(unsafe_code)]
    /// Create a memory map if we haven't already done so.
    pub fn create_mapping(&mut self) -> Result<(), IoError> {
        if self.mmap.is_some() {
            return Ok(());
        }
        match self.vector {
            Some(ref mut vector) => {
                let mut mmap = try!(Mmap::anonymous(MAX_FILE_SIZE, Protection::ReadWrite));
                let _ = unsafe { mmap.as_mut_slice() }.write_all(&vector[..]);
                self.mmap = Some(mmap);
            }
            None => {
                return Err(IoError::new(IoErrorKind::WriteZero,
                                        "Failed to create mapping from uninitialised vector."))
            }
        };

        if self.mmap.is_some() {
            self.vector = None;
        }
        Ok(())
    }

    /// If we are a vector return the vector otherwise return empty vector.
    pub fn to_vec(&self) -> Vec<u8> {
        match self.vector {
            Some(ref vector) => vector.clone(),
            None => Vec::<u8>::new(),
        }
    }
}

#[allow(unsafe_code)]
impl Index<usize> for Sequencer {
    type Output = u8;
    fn index(&self, index: usize) -> &u8 {
        match self.vector {
            Some(ref vector) => &vector[index],
            None => {
                match self.mmap {
                    Some(ref mmap) => unsafe { &mmap.as_slice()[index] },
                    None => panic!("Uninitialised"),
                }
            }
        }
    }
}

#[allow(unsafe_code)]
impl IndexMut<usize> for Sequencer {
    fn index_mut(&mut self, index: usize) -> &mut u8 {
        match self.vector {
            Some(ref mut vector) => &mut vector[index],
            None => {
                match self.mmap {
                    Some(ref mut mmap) => unsafe { &mut mmap.as_mut_slice()[index] },
                    None => panic!("Uninitialised"),
                }
            }
        }
    }
}

#[allow(unsafe_code)]
impl Deref for Sequencer {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        match self.vector {
            Some(ref vector) => &*vector,
            None => {
                match self.mmap {
                    Some(ref mmap) => unsafe { mmap.as_slice() },
                    None => panic!("Uninitialised"),
                }
            }
        }
    }
}

#[allow(unsafe_code)]
impl DerefMut for Sequencer {
    fn deref_mut(&mut self) -> &mut [u8] {
        match self.vector {
            Some(ref mut vector) => &mut *vector,
            None => {
                match self.mmap {
                    Some(ref mut mmap) => unsafe { &mut *mmap.as_mut_slice() },
                    None => panic!("Uninitialised"),
                }
            }
        }
    }
}

impl Extend<u8> for Sequencer {
    fn extend<I>(&mut self, iterable: I)
        where I: IntoIterator<Item = u8>
    {
        if let Some(ref mut vector) = self.vector {
            vector.extend(iterable);
        }
    }
}
