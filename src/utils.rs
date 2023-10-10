use std::{
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

pub fn get_dir_path(account_id: &str) -> PathBuf {
    let mut home_dir = dirs::home_dir().expect("Impossible to get your home dir!");
    home_dir.push(format!(".near-credentials/mainnet/{}.json", account_id));
    home_dir
}

pub fn convert_oct_u128_from_string(s: &str) -> u128 {
    assert!(!s.contains("."), "Can't parse float");
    let oct_decimal = 18;
    u128::from_str_radix(s, 10)
        .unwrap()
        .checked_mul((10 as u128).pow(oct_decimal))
        .unwrap()
}

pub fn timestamp() -> u128 {
    let now = SystemTime::now();

    let earlier = now
        .checked_sub(std::time::Duration::from_secs(30 * 60))
        .unwrap();
    earlier
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_nanos()
}

#[test]
pub fn test_convert_oct_u128_from_string() {
    dbg!(&convert_oct_u128_from_string("1"));
}
