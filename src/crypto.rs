// Copyright 2019 Fredrik Portström <https://portstrom.com>
// This is free software distributed under the terms specified in
// the file LICENSE at the top-level directory of this distribution.

use sha2::Digest;

const MAC_SIZE: usize = 32;

pub struct SubKeys {
    cipher: [u8; 16],
    mac: [u8; 16],
}

impl SubKeys {
    pub fn new(key: [u8; 16]) -> Self {
        let mut hasher = sha2::Sha256::new();
        hasher.input(key);
        let hash = hasher.result();
        let mut cipher = [0; 16];
        let mut mac = [0; 16];
        cipher.copy_from_slice(&hash[..16]);
        mac.copy_from_slice(&hash[16..]);
        SubKeys { cipher, mac }
    }
}

pub fn create_user_passphrase_key(passphrase: &str, salt: &[u8]) -> [u8; 16] {
    let mut hasher = sha2::Sha256::new();
    hasher.input(passphrase);
    let user_passphrase_hash = hasher.result();
    let mut user_passphrase_key_source = [0; 24];
    crypto::bcrypt::bcrypt(
        8,
        salt,
        &user_passphrase_hash,
        &mut user_passphrase_key_source,
    );
    let mut user_passphrase_key = [0; 16];
    user_passphrase_key.copy_from_slice(&user_passphrase_key_source[..16]);
    user_passphrase_key
}

pub fn decrypt_key(key: &[u8], message: &[u8]) -> Option<[u8; 16]> {
    if key.len() == 16 && message.len() == 16 {
        use aes::block_cipher_trait::BlockCipher;
        let mut output = [0; 16];
        output.copy_from_slice(message);
        let cipher = aes::Aes128::new(key.into());
        cipher.decrypt_block(output.as_mut().into());
        for byte in &mut output {
            *byte ^= 0x88;
        }
        Some(output)
    } else {
        None
    }
}

pub fn decrypt_with_mac(sub_keys: &SubKeys, message: &[u8]) -> Option<Vec<u8>> {
    use {block_modes::BlockMode, hmac::Mac};
    if message.len() < MAC_SIZE || message.len() % 16 != 1 {
        return None;
    }
    let mut mac = hmac::Hmac::<sha2::Sha256>::new_varkey(&sub_keys.mac).unwrap();
    let message_without_mac = &message[1..message.len() - MAC_SIZE];
    mac.input(message_without_mac);
    if mac.verify(&message[message.len() - MAC_SIZE..]).is_err() {
        return None;
    }
    let cipher = block_modes::Cbc::<aes::Aes128, block_modes::block_padding::Pkcs7>::new_fix(
        sub_keys.cipher[..].into(),
        message_without_mac[..16].into(),
    );
    cipher.decrypt_vec(&message_without_mac[16..]).ok()
}
