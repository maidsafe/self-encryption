// Copyright 2016 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under (1) the MaidSafe.net Commercial License,
// version 1.0 or later, or (2) The General Public License (GPL), version 3, depending on which
// licence you accepted on initial access to the Software (the "Licences").
//
// By contributing code to the SAFE Network Software, or to this project generally, you agree to be
// bound by the terms of the MaidSafe Contributor Agreement.  This, along with the Licenses can be
// found in the root directory of this project at LICENSE, COPYING and CONTRIBUTOR.
//
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.
//
// Please review the Licences for the specific language governing permissions and limitations
// relating to use of the SAFE Network Software.

use super::{COMPRESSION_QUALITY, PAD_SIZE, Pad, SelfEncryptionError, StorageError};
use brotli2::write::{BrotliDecoder, BrotliEncoder};
use data_map::ChunkDetails;
use encryption::{self, IV_SIZE, Iv, KEY_SIZE, Key};
#[cfg(test)]
use rand::Rng;
use rust_sodium;
#[cfg(test)]
use std::cmp;
use std::io::Write;
use std::sync::{ONCE_INIT, Once};
use tiny_keccak::Keccak;

pub const HASH_SIZE: usize = 32;

pub fn sha3_256_hash(input: &[u8]) -> Vec<u8> {
    let mut sha3 = Keccak::new_sha3_256();
    sha3.update(input);
    let mut name: [u8; HASH_SIZE] = [0; HASH_SIZE];
    sha3.finalize(&mut name);
    name.to_vec()
}

pub fn get_pad_key_and_iv(chunk_index: usize, chunks: &[ChunkDetails]) -> (Pad, Key, Iv) {
    let (n_1, n_2) = match chunk_index {
        0 => (chunks.len() - 1, chunks.len() - 2),
        1 => (0, chunks.len() - 1),
        n => (n - 1, n - 2),
    };
    let this_pre_hash = &chunks[chunk_index].pre_hash;
    let n_1_pre_hash = &chunks[n_1].pre_hash;
    let n_2_pre_hash = &chunks[n_2].pre_hash;

    let mut pad = [0u8; PAD_SIZE];
    let mut key = [0u8; KEY_SIZE];
    let mut iv = [0u8; IV_SIZE];

    for (pad_iv_el, element) in
        pad.iter_mut()
            .chain(iv.iter_mut())
            .zip(this_pre_hash.iter().chain(n_2_pre_hash.iter())) {
        *pad_iv_el = *element;
    }

    for (key_el, element) in key.iter_mut().zip(n_1_pre_hash.iter()) {
        *key_el = *element;
    }

    (Pad(pad), Key(key), Iv(iv))
}

pub fn encrypt_chunk<E: StorageError>(content: &[u8],
                                      pad_key_iv: (Pad, Key, Iv))
                                      -> Result<Vec<u8>, SelfEncryptionError<E>> {
    let (pad, key, iv) = pad_key_iv;
    let mut compressor = BrotliEncoder::new(vec![], COMPRESSION_QUALITY);
    if compressor.write_all(content).is_err() {
        return Err(SelfEncryptionError::Compression);
    }
    let compressed = match compressor.finish() {
        Ok(data) => data,
        Err(_) => return Err(SelfEncryptionError::Compression),
    };
    let encrypted = encryption::encrypt(&compressed, &key, &iv);
    Ok(xor(&encrypted, &pad))
}

pub fn decrypt_chunk<E: StorageError>(content: &[u8],
                                      pad_key_iv: (Pad, Key, Iv))
                                      -> Result<Vec<u8>, SelfEncryptionError<E>> {
    let (pad, key, iv) = pad_key_iv;
    let xor_result = xor(content, &pad);
    let decrypted = encryption::decrypt(&xor_result, &key, &iv)?;
    let mut decompressor = BrotliDecoder::new(vec![]);
    if decompressor.write_all(&decrypted).is_err() {
        return Err(SelfEncryptionError::Compression);
    }
    decompressor
        .finish()
        .map_err(|_| SelfEncryptionError::Compression)
}

// Helper function to XOR a data with a pad (pad will be rotated to fill the length)
pub fn xor(data: &[u8], &Pad(pad): &Pad) -> Vec<u8> {
    data.iter()
        .zip(pad.iter().cycle())
        .map(|(&a, &b)| a ^ b)
        .collect()
}

pub fn initialise_rust_sodium() {
    static INITIALISE_SODIUMOXIDE: Once = ONCE_INIT;
    INITIALISE_SODIUMOXIDE.call_once(|| assert!(rust_sodium::init()));
}

#[cfg(test)]
pub fn make_random_pieces<'a, T: Rng>(rng: &mut T,
                                      data: &'a [u8],
                                      min_len_of_first_piece: usize)
                                      -> Vec<&'a [u8]> {
    let mut pieces = vec![];
    let mut split_index = 0;
    loop {
        let min_len = if split_index == 0 {
            min_len_of_first_piece
        } else {
            1
        };
        let max_len = cmp::max(data.len() / 3, min_len + 1);
        let new_split_index = split_index + rng.gen_range(min_len, max_len);
        if new_split_index >= data.len() {
            pieces.push(&data[split_index..]);
            break;
        }
        pieces.push(&data[split_index..new_split_index]);
        split_index = new_split_index;
    }
    pieces
}
