use aes_gcm::aead::generic_array::GenericArray;
use aes_gcm::aead::{Aead, NewAead};
use aes_gcm::Aes256Gcm;
use serde_json::Value;
use std::fs;
use std::io::Read;
use std::path::PathBuf;
use winapi::um::dpapi::CryptUnprotectData;
use winapi::um::wincrypt::CRYPTOAPI_BLOB;

pub fn dpapi_decrypt(mut cipher_text: Vec<u8>) -> Vec<u8> {
    let mut in_data = CRYPTOAPI_BLOB {
        cbData: cipher_text.len() as u32,
        pbData: cipher_text.as_mut_ptr(),
    };

    let mut out_data = CRYPTOAPI_BLOB {
        cbData: 0,
        pbData: std::ptr::null_mut(),
    };

    unsafe {
        CryptUnprotectData(
            &mut in_data,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            0,
            &mut out_data,
        );

        let plain_text = Vec::from_raw_parts(
            out_data.pbData,
            out_data.cbData as usize,
            out_data.cbData as usize,
        );

        return plain_text;
    };
}

pub fn aes_decrypt(cipher_text: Vec<u8>, master_key: &Vec<u8>) -> Vec<u8> {
    let key = GenericArray::from_slice(&master_key);
    let cipher = Aes256Gcm::new(key);

    let nonce = GenericArray::from_slice(&cipher_text[3..15]);

    //直接使用aes來進行解密，不成功 的話調用dpapi進行解密
    let plain_text = match cipher.decrypt(nonce, &cipher_text[15..]) {
        Ok(plain_text) => plain_text,
        Err(_) => dpapi_decrypt(cipher_text),
    };

    return plain_text;
}

pub fn get_master_key(master_key_path: &PathBuf) -> Option<Vec<u8>> {
    let contents = fs::read_to_string(master_key_path).ok()?;

    let json: Value = serde_json::from_str(contents.as_str()).ok()?;

    if let Some(encrypted_key) = json["os_crypt"]["encrypted_key"].as_str() {
        let mut plain_text = base64::decode(encrypted_key).ok()?[5..].to_vec();

        let master_key = dpapi_decrypt(plain_text);
        return Some(master_key);
    }
    return None;
}
