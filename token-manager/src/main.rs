use std::str::FromStr;

pub use switchboard_solana::{get_ixn_discriminator, switchboard_function, sb_error};
pub use switchboard_solana::prelude::*;
use crate::solana_sdk::commitment_config::CommitmentConfig;

mod params;
pub use params::*;

#[switchboard_function]
pub async fn sb_function(runner: FunctionRunner, params: Vec<u8>) -> Result<Vec<Instruction>, SbFunctionError> {
    // parse and validate user provided request params
    println!("incoming param is: {:?}", params);
    let data: ContainerParams = ContainerParams::decode(&params).map_err(|_| Error::ArgParseFail)?;
    // Generate our random result
    println!("max value is: {}", data.max_value);
    let random_result = generate_randomness(1, data.max_value);
    println!("random number is: {}", random_result);
    let mut random_bytes = random_result.to_le_bytes().to_vec();

    // IXN DATA:
    // LEN: 12 bytes
    // [0-8]: Anchor Ixn Discriminator
    // [9-12]: Random Result as u32
    let mut ixn_data = get_ixn_discriminator("trigger_update").to_vec();
    ixn_data.append(&mut random_bytes);

    // ACCOUNTS:
    // 1. Global (mut): global state
    // 4. Enclave Signer (signer): our Gramine generated keypair
    // 2. Switchboard Function
    // 3. Switchboard Function Request
    let request_pubkey = runner.function_request_key.unwrap();

    Ok(vec![Instruction {
        program_id: data.program_id,
        data: ixn_data,
        accounts: vec![
            AccountMeta::new(data.global, false), // Global
            AccountMeta::new(data.mint, false), // Mint
            AccountMeta::new_readonly(runner.signer, true), // Enclave signer
            AccountMeta::new_readonly(runner.switchboard, false), // Switchboard
            AccountMeta::new_readonly(runner.switchboard_state, false), // Switchboard_state
            AccountMeta::new_readonly(runner.attestation_queue.unwrap(), false), // Switchboard_attestation_queue
            AccountMeta::new_readonly(runner.function, false), // switchboard_function
            AccountMeta::new_readonly(request_pubkey.clone(), false), // Switchboard_request
            AccountMeta::new(data.sb_escrow, false), // switchboard_request_escrow 
            AccountMeta::new_readonly(anchor_spl::token::spl_token::native_mint::ID, false), // Switchboard_mint == Native SOL
            AccountMeta::new_readonly(anchor_spl::token::ID, false), // TID
            AccountMeta::new_readonly(anchor_spl::token_2022::ID, false), // TID_22
            AccountMeta::new_readonly(anchor_spl::associated_token::ID, false), // ATID
            AccountMeta::new_readonly(anchor_lang::system_program::ID, false), // SID
        ],
    }])


    // Ok(vec![Instruction {
    //     program_id: data.program_id,
    //     data: ixn_data,
    //     accounts: vec![
    //         AccountMeta::new(data.global, false), // Global
    //         AccountMeta::new(data.mint, false), // Mint
    //         AccountMeta::new_readonly(runner.signer, true), // Enclave signer
    //         AccountMeta::new_readonly(runner.switchboard, false), // Switchboard
    //         AccountMeta::new_readonly(runner.switchboard_state, false), // Switchboard_state
    //         AccountMeta::new_readonly(runner.attestation_queue.unwrap(), false), // Switchboard_attestation_queue
    //         AccountMeta::new_readonly(runner.function, false), // switchboard_function
    //         AccountMeta::new(runner.function_request_key.unwrap().clone(), false), // Switchboard_request
    //         AccountMeta::new_readonly(request_pubkey.clone(), false), // switchboard_request_escrow 
    //         AccountMeta::new_readonly(anchor_spl::token::spl_token::native_mint::ID, false), // Switchboard_mint == Native SOL
    //         AccountMeta::new_readonly(anchor_spl::token::ID, false), // TID
    //         AccountMeta::new_readonly(anchor_spl::token_2022::ID, false), // TID_22
    //         AccountMeta::new_readonly(anchor_spl::associated_token::ID, false), // ATID
    //         AccountMeta::new_readonly(anchor_lang::system_program::ID, false), // SID
    //     ],
    // }])
}

#[sb_error]
pub enum Error {
    ArgParseFail,
}

fn generate_randomness(min: u8, max: u8) -> u8 {
    if min == max {
        return min;
    }
    if min > max {
        return generate_randomness(max, min);
    }

    // We add one so its inclusive [min, max]
    let window = (max + 1) - min;

    let mut bytes: [u8; 4] = [0u8; 4];
    Gramine::read_rand(&mut bytes).expect("gramine failed to generate randomness");
    let raw_result: &[u8] = bytemuck::cast_slice(&bytes[..]);

    (raw_result[0] % window) + min
}

#[cfg(test)]
mod tests {
    use super::*;

    // 1. Check when lower_bound is greater than upper_bound
    #[test]
    fn test_generate_randomness_with_flipped_bounds() {
        let min = 100;
        let max = 50;

        let result = generate_randomness(100, 50);
        assert!(result >= max && result < min);
    }

    // 2. Check when lower_bound is equal to upper_bound
    #[test]
    fn test_generate_randomness_with_equal_bounds() {
        let bound = 100;
        assert_eq!(generate_randomness(bound, bound), bound);
    }

    // 3. Test within a range
    #[test]
    fn test_generate_randomness_within_bounds() {
        let min = 100;
        let max = 200;

        let result = generate_randomness(min, max);

        assert!(result >= min && result < max);
    }

    // 4. Test randomness distribution (not truly deterministic, but a sanity check)
    #[test]
    fn test_generate_randomness_distribution() {
        let min = 0;
        let max = 9;

        let mut counts = vec![0; 10];
        for _ in 0..1000 {
            let result = generate_randomness(min, max);
            let index: usize = result as usize;
            counts[index] += 1;
        }

        // Ensure all counts are non-zero (probabilistically should be the case)
        for count in counts.iter() {
            assert!(*count > 0);
        }
    }
}
