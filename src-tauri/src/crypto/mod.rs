pub mod key;

pub use key::{
    change_password, derive_encryption_key, get_key_config, get_sqlcipher_key,
    initialize_encryption, is_encryption_initialized, KeyConfig, KeyMode,
};
