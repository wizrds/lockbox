use crc32fast::Hasher;
use sha2::{Sha512, Digest};
use rand::{distr::Alphanumeric, Rng, rng};
use constant_time_eq::constant_time_eq;


/// Generates a random alphanumeric string of the specified length. The charset used
/// is the standard alphanumeric characters (0-9, a-z, A-Z) which is 62 characters long.
/// 
/// This ensures the amount of entropy is:
/// entropy = log2(62^length) = length * log2(62) = 5.954 * length
/// 
/// # Arguments
/// * `length` - The desired length of the alphanumeric string.
/// 
/// # Returns
/// A string containing random alphanumeric characters of the specified length.
pub fn generate_random_alphanumeric(length: usize) -> String {
    rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect::<String>()
}


/// Generates a checksum for a slice of strings using CRC32.
/// 
/// # Arguments
/// * `items` - A slice of string slices to generate the checksum from.
/// 
/// # Returns
/// A string representing the checksum, encoded in base62.
/// The checksum is zero-padded to ensure it is at least 6 characters long.
pub fn get_checksum(items: &[&str]) -> String {
    let mut hasher = Hasher::new();

    for item in items {
        hasher.update(item.as_bytes());
    }

    let crc = hasher.finalize();
    let encoded = base62::encode(crc as u128);

    format!("{:0>6}", encoded)
}


/// Hash a string using SHA-512
/// 
/// # Arguments
/// * `input` - A string slice to hash.
/// 
/// # Returns
/// A string representing the SHA-512 hash of the input.
pub fn hash_sha512(input: &str) -> String {
    format!("{:x}", Sha512::digest(input.as_bytes()))
}


/// Perform a constant-time comparison of two strings.
/// 
/// # Arguments
/// * `a` - The first string slice to compare.
/// * `b` - The second string slice to compare.
/// 
/// # Returns
/// `true` if the strings are equal, `false` otherwise.
pub fn constant_cmp(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }

    constant_time_eq(a.as_bytes(), b.as_bytes())
}