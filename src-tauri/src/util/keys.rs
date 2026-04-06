use base64::{engine::general_purpose::STANDARD, Engine};

const K: &[u8] = b"ZephyrModManager2024";

fn decode(encoded: &str) -> String {
    let data = STANDARD.decode(encoded).unwrap_or_default();
    data.iter()
        .enumerate()
        .map(|(i, &b)| (b ^ K[i % K.len()]) as char)
        .collect()
}

pub fn curseforge_key() -> String {
    decode("flcRTEhCaSA9fQAeOwsiQnl1elFpJiQPDB84WRF/FAEFNw4XAQALTC8yRD9WISAHDBUEXCoUMzsGe3lB")
}

pub fn nexusmods_key() -> String {
    decode("FBUqIRAoJTU+f0pFFwACEGNgBXZ1PCZYDgoFKhA4ACVSJixHBmRdegoMCDA6JSJSSWAOAQsOViUCR3xZGAtCDBwFYEJVCzEjNTIEA31qYUAQMhsZMyI7Di4qXFM=")
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::{engine::general_purpose::STANDARD, Engine};

    fn encode(plaintext: &str) -> String {
        let result: Vec<u8> = plaintext
            .bytes()
            .enumerate()
            .map(|(i, b)| b ^ K[i % K.len()])
            .collect();
        STANDARD.encode(&result)
    }

    #[test]
    fn curseforge_key_is_nonempty() {
        assert!(!curseforge_key().is_empty());
    }

    #[test]
    fn nexusmods_key_is_nonempty() {
        assert!(!nexusmods_key().is_empty());
    }

    #[test]
    fn keys_are_ascii() {
        assert!(curseforge_key().chars().all(|c| c.is_ascii()));
        assert!(nexusmods_key().chars().all(|c| c.is_ascii()));
    }

    #[test]
    fn keys_are_stable() {
        assert_eq!(curseforge_key(), curseforge_key());
        assert_eq!(nexusmods_key(), nexusmods_key());
    }

    #[test]
    fn roundtrip_encode_decode() {
        let original = "test-api-key-12345!@#";
        let encoded = encode(original);
        let decoded = decode(&encoded);
        assert_eq!(decoded, original);
    }

    #[test]
    fn curseforge_key_has_reasonable_length() {
        let key = curseforge_key();
        assert!(key.len() > 20, "key too short: {}", key.len());
    }

    #[test]
    fn nexusmods_key_has_reasonable_length() {
        let key = nexusmods_key();
        assert!(key.len() > 20, "key too short: {}", key.len());
    }
}
