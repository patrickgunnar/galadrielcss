// some constants for alphanumeric characters, len the alphabetic characters and the seed
const ALPHA: &str = "abcdefghijklmnopqrstuvwxyz";
const ALPHA_LEN: usize = 26;
const ALPHANUMERIC: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
const ALPHANUMERIC_LEN: usize = 62;
const SEED: u64 = 5381;

// Cipher is responsible for hashing the receives string
fn cipher(seed: u64, str: &str) -> u64 {
    let mut i = str.len();
    let mut hash = seed;

    while i > 0 {
        i -= 1;
        hash = hash.wrapping_mul(33) ^ str.as_bytes()[i] as u64;
    }

    (hash.wrapping_mul(33)) ^ str.len() as u64
}

// Alphanumeric is responsible for collecting
// a char on the alphanumeric constant
fn alphanumeric(n: usize, is_alpha: bool) -> char {
    if is_alpha {
        ALPHA.chars().nth(n).unwrap()
    } else {
        ALPHANUMERIC.chars().nth(n).unwrap()
    }
}

// Gen prefix is responsible to generate a name identifier
// from a received string, than returns the generated name identifier
pub fn generate_prefix(input: &str, is_alpha: bool, size: usize) -> String {
    let alpha_const = match is_alpha {
        true => ALPHA_LEN,
        false => ALPHANUMERIC_LEN,
    };

    let mut name = String::new();
    let code = cipher(SEED, input);
    let mut x = code;

    while x > alpha_const as u64 {
        let remainder = (x % alpha_const as u64) as usize;

        name = alphanumeric(remainder, is_alpha).to_string() + &name;
        x /= alpha_const as u64;
    }

    name = alphanumeric((x % alpha_const as u64) as usize, is_alpha).to_string() + &name;

    if name.len() > size {
        name[name.len() - size..].to_string()
    } else {
        name
    }
}
