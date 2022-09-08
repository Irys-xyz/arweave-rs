use std::fs;

use jsonwebkey::JsonWebKey;

use crate::error::ArweaveError;

pub fn load_from_file(path: &str) -> Result<JsonWebKey, ArweaveError> {
    let jwt_str =
        fs::read_to_string(path).unwrap_or_else(|_| panic!("Unable to read file {}", path));
    jwt_str
        .parse::<JsonWebKey>()
        .map_err(|err| ArweaveError::WalletError(err.to_string()))
}

mod tests {
    #[test]
    fn should_load_wallet_correctly() {
        let res = super::load_from_file("res/test_wallet.json");
        res.unwrap();
    }
}
