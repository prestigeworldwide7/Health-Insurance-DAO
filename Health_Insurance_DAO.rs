use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
    system_instruction,
    sysvar::{clock::Clock, rent::Rent, Sysvar},
};

// Define structures for the DAO members
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct Member {
    pub member_address: Pubkey, // The public key of a member for identification
    pub joined_timestamp: i64,  // Unix timestamp when the member joined the DAO
}

// Define structures for claims within the DAO
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct Claim {
    pub claim_id: u64,         // Unique identifier for each claim
    pub member: Pubkey,        // The member who submitted the claim
    pub amount: u64,           // The amount of the claim in lamports
    pub verified: bool,        // Indicates whether the claim has been verified by an oracle
}

// Main DAO structure to hold all relevant data
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct HealthInsuranceDAO {
    pub admin: Pubkey,         // The admin who manages the DAO
    pub members: Vec<Member>,  // List of all members in the DAO
    pub claims: Vec<Claim>,    // List of all claims submitted to the DAO
}

// Entrypoint for the program, handling different instructions
entrypoint!(process_instruction);

fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let account = next_account_info(accounts_iter)?;

    // Check if this program owns the account we're about to modify
    if account.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }

    let mut dao_data = HealthInsuranceDAO::try_from_slice(&account.data.borrow())?;

    // Match on the first byte of instruction_data to determine the instruction type
    match instruction_data[0] {
        0 => {
            // Instruction for joining the DAO
            let member = next_account_info(accounts_iter)?;
            dao_data.members.push(Member {
                member_address: *member.key,
                joined_timestamp: Clock::get()?.unix_timestamp,
            });
            msg!("New member joined the DAO");
        }
        1 => {
            // Instruction for submitting a new claim
            let member = next_account_info(accounts_iter)?;
            let treasury = next_account_info(accounts_iter)?;

            dao_data.claims.push(Claim {
                claim_id: dao_data.claims.len() as u64, // Assign a new ID based on current count
                member: *member.key,
                amount: 1000000, // Hardcoded for example; in real-world, this would be dynamic
                verified: false, // Claims start as unverified
            });
            msg!("Claim submitted for {} lamports", 1000000);
        }
        2 => {
            // Instruction for verifying a claim using oracle data
            let oracle = next_account_info(accounts_iter)?;
            // Extract claim index from instruction data
            let claim_index = u64::from_le_bytes(instruction_data[1..9].try_into().map_err(|_| ProgramError::InvalidInstructionData)?);

            // Find and update the claim at the given index
            if let Some(claim) = dao_data.claims.get_mut(claim_index as usize) {
                // Simulate oracle verification. Here, we check the first byte of the oracle's data.
                // In a real scenario, this would involve calling an oracle service for validation.
                let verification_result = oracle.data.borrow()[0] == 1; // 1 means verified, 0 means not verified in our mock setup
                claim.verified = verification_result;
                msg!("Claim {} verification status updated to: {}", claim.claim_id, claim.verified);
            } else {
                return Err(ProgramError::InvalidAccountData); // If the claim index is out of bounds
            }
        }
        _ => return Err(ProgramError::InvalidInstructionData), // If the instruction is unrecognized
    }

    // Save the updated DAO state back into the account's data
    dao_data.serialize(&mut &mut account.data.borrow_mut()[..])?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_program::instruction::{AccountMeta, Instruction};
    use solana_program_test::*;
    use solana_sdk::{
        account::Account,
        signature::{Keypair, Signer},
        transaction::Transaction,
    };

    #[tokio::test]
    async fn test_join_dao() {
        let program_id = Pubkey::new_unique();
        let member = Keypair::new();
        let dao_account = Keypair::new();
        let rent = Rent::default();

        let mut program_test = ProgramTest::new("health_insurance_dao", program_id, processor!(process_instruction));

        // Setup DAO account with initial state
        program_test.add_account(
            dao_account.pubkey(),
            Account {
                lamports: rent.minimum_balance(HealthInsuranceDAO::default().try_to_vec().unwrap().len()),
                data: HealthInsuranceDAO::default().try_to_vec().unwrap(),
                owner: program_id,
                executable: false,
                rent_epoch: 0,
            },
        );

        let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

        // Create instruction to join DAO
        let instruction = Instruction {
            program_id,
            accounts: vec![
                AccountMeta::new(dao_account.pubkey(), false), // The DAO account
                AccountMeta::new(member.pubkey(), true),       // The new member's account
            ],
            data: vec![0], // Join DAO instruction
        };

        let transaction = Transaction::new_signed_with_payer(
            &[instruction],
            Some(&payer.pubkey()),
            &[&payer, &member],
            recent_blockhash,
        );

        // Process the transaction to add the member
        banks_client.process_transaction(transaction).await.unwrap();
    }

    #[tokio::test]
    async fn test_claim_verification() {
        let program_id = Pubkey::new_unique();
        let member = Keypair::new();
        let dao_account = Keypair::new();
        let oracle = Keypair::new();
        let rent = Rent::default();

        let mut program_test = ProgramTest::new("health_insurance_dao", program_id, processor!(process_instruction));

        // Setup DAO account with initial state
        program_test.add_account(
            dao_account.pubkey(),
            Account {
                lamports: rent.minimum_balance(HealthInsuranceDAO::default().try_to_vec().unwrap().len()),
                data: HealthInsuranceDAO::default().try_to_vec().unwrap(),
                owner: program_id,
                executable: false,
                rent_epoch: 0,
            },
        );
        // Setup oracle account for claim verification simulation
        program_test.add_account(
            oracle.pubkey(),
            Account {
                lamports: rent.minimum_balance(1),
                data: vec![1], // Oracle data: 1 for verified, 0 for not verified - this is just for testing
                owner: program_id,
                executable: false,
                rent_epoch: 0,
            },
        );

        let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

        // Submit a claim
        let submit_claim_instruction = Instruction {
            program_id,
            accounts: vec![
                AccountMeta::new(dao_account.pubkey(), false), // DAO account
                AccountMeta::new(member.pubkey(), true),       // Member submitting claim
                AccountMeta::new_readonly(solana_sdk::system_program::id(), false), // System program for potential lamports transfer
            ],
            data: vec![1], // Submit claim instruction
        };
        let submit_claim_transaction = Transaction::new_signed_with_payer(
            &[submit_claim_instruction],
            Some(&payer.pubkey()),
            &[&payer, &member],
            recent_blockhash,
        );
        banks_client.process_transaction(submit_claim_transaction).await.unwrap();

        // Verify the claim
        let verify_claim_instruction = Instruction {
            program_id,
            accounts: vec![
                AccountMeta::new(dao_account.pubkey(), false), // DAO account
                AccountMeta::new_readonly(oracle.pubkey(), false), // Oracle account for verification
            ],
            // Data format: [instruction_type, claim_id (8 bytes, little-endian)]
            data: [2u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 1u8].to_vec(), // 2 for verify claim, claim_id 0
        };
        let verify_claim_transaction = Transaction::new_signed_with_payer(
            &[verify_claim_instruction],
            Some(&payer.pubkey()),
            &[&payer],
            recent_blockhash,
        );
        banks_client.process_transaction(verify_claim_transaction).await.unwrap();
    }
}
