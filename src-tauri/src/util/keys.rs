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
