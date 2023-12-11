use crate::*;

pub struct ContainerParams {
    pub program_id: Pubkey,
    pub max_value: u8,
    pub global: Pubkey,
    pub mint: Pubkey,
}

impl ContainerParams {
    pub fn decode(container_params: &Vec<u8>) -> std::result::Result<Self, SbError> {
        let params = String::from_utf8(container_params.clone()).unwrap();

        let mut program_id: Pubkey = Pubkey::default();
        let mut max_value: u8 = 0;
        let mut global: Pubkey = Pubkey::default();
        let mut mint: Pubkey = Pubkey::default();

        for env_pair in params.split(',') {
            let pair: Vec<&str> = env_pair.splitn(2, '=').collect();
            if pair.len() == 2 {
                match pair[0] {
                    "PID" => program_id = Pubkey::from_str(pair[1]).unwrap(),
                    "MAX_VALUE" => max_value = pair[1].parse::<u8>().unwrap(),
                    "GLOBAL" => global = Pubkey::from_str(pair[1]).unwrap(),
                    "MINT" => mint = Pubkey::from_str(pair[1]).unwrap(),
                    _ => {}
                }
            }
        }

        if program_id == Pubkey::default() {
            return Err(SbError::CustomMessage(
                "PID cannot be undefined".to_string(),
            ));
        }
        if global == Pubkey::default() {
            return Err(SbError::CustomMessage(
                "PID cannot be undefined".to_string(),
            ));
        }
        if mint == Pubkey::default() {
            return Err(SbError::CustomMessage(
                "PID cannot be undefined".to_string(),
            ));
        }
        if max_value == 0 {
            return Err(SbError::CustomMessage(
                "MAX_VALUE must be greater than 0".to_string(),
            ));
        }

        Ok(Self {
            program_id,
            max_value,
            global,
            mint,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_params_decode() {
        let request_params_string = format!("PID={},MAX_VALUE={}", anchor_spl::token::ID, 1,);
        let request_params_bytes = request_params_string.into_bytes();

        let params = ContainerParams::decode(&request_params_bytes).unwrap();

        assert_eq!(params.program_id, anchor_spl::token::ID);
        assert_eq!(params.max_value, 1);
    }
}
