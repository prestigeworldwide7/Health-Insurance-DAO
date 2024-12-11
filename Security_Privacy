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

// Define role for access control
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
enum Role {
    Admin,
    Member,
    Verifier,
}

// Enhance Member structure with privacy and security features
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct Member {
    pub member_address: Pubkey,
    pub joined_timestamp: i64,
    pub role: Role, // Assign roles to members for access control
    pub encrypted_data_hash: [u8; 32], // Hash of off-chain encrypted data
}

// Enhance Claim structure to include privacy considerations
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct Claim {
    pub claim_id: u64,
    pub member: Pubkey,
    pub amount: u64, // In lamports for simplicity
    pub zkp_proof: Vec<u8>, // Zero-knowledge proof for claim validation (simplified)
}

// Program state with added security and privacy components
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct HealthInsuranceDAO {
    pub admin: Pubkey,
    pub members: Vec<Member>,
    pub claims: Vec<Claim>,
    pub multi_sig_signers: Vec<Pubkey>, // List of public keys required for multi-sig operations
}

entrypoint!(process_instruction);

fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let account = next_account_info(accounts_iter)?;

    if account.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }

    let mut dao_data = HealthInsuranceDAO::try_from_slice(&account.data.borrow())?;

    match instruction_data[0] {
        0 => {
            // Join DAO - Enhanced for security 
            let new_member = next_account_info(accounts_iter)?;
            let encrypted_data_hash = instruction_data[1..33].try_into().map_err(|_| ProgramError::InvalidInstructionData)?;
            let role = match instruction_data[33] {
                1 => Role::Member,
                _ => return Err(ProgramError::InvalidInstructionData),
            };

            // Check if the member is not already in the DAO
            if dao_data.members.iter().any(|m| m.member_address == *new_member.key) {
                return Err(ProgramError::AccountAlreadyInitialized);
            }

            dao_data.members.push(Member {
                member_address: *new_member.key,
                joined_timestamp: Clock::get()?.unix_timestamp,
                role,
                encrypted_data_hash,
            });

            msg!("New member joined the DAO with role {:?}", role);
        }
        1 => {
            // Submit Claim - Enhanced with basic ZKP for privacy
            let member = next_account_info(accounts_iter)?;
            let treasury = next_account_info(accounts_iter)?;
            let zkp_proof = instruction_data[1..].to_vec();

            // Verify member's role (simplified, in reality, you'd check against actual data)
            if !dao_data.members.iter().any(|m| m.member_address == *member.key && m.role == Role::Member) {
                return Err(ProgramError::InvalidArgument);
            }

            // Here, you would implement or check the ZKP. This is a placeholder:
            if !verify_zkp(&zkp_proof) { // This function would need to be implemented or integrated
                return Err(ProgramError::InvalidArgument);
            }

            dao_data.claims.push(Claim {
                claim_id: dao_data.claims.len() as u64,
                member: *member.key,
                amount: 1000000,
                zkp_proof,
            });
            msg!("Claim submitted for {} lamports with ZKP", 1000000);
        }
        2 => {
            // New instruction for multi-sig operation
            let signers = accounts_iter.take_while(|a| a.is_signer).collect::<Vec<_>>();
            
            if signers.len() < dao_data.multi_sig_signers.len() {
                return Err(ProgramError::InvalidArgument); // Not enough signatures
            }
            
            // Here you would implement the multi-sig logic. This is just a placeholder:
            msg!("Multi-signature operation executed with {} signers", signers.len());
        }
        _ => return Err(ProgramError::InvalidInstructionData),
    }

    dao_data.serialize(&mut &mut account.data.borrow_mut()[..])?;
    Ok(())
}

// Placeholder for ZKP verification
fn verify_zkp(proof: &[u8]) -> bool {
    // In a real scenario, this would involve complex cryptographic verification
    proof.len() > 0 // Very basic check for this example
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

        let mut data = vec![0, 1]; // Instruction type and role
        data.extend([0u8; 32]); // Encrypted data hash

        let instruction = Instruction {
            program_id,
            accounts: vec![
                AccountMeta::new(dao_account.pubkey(), false),
                AccountMeta::new(member.pubkey(), true),
            ],
            data,
        };

        let transaction = Transaction::new_signed_with_payer(
            &[instruction],
            Some(&payer.pubkey()),
            &[&payer, &member],
            recent_blockhash,
        );

        banks_client.process_transaction(transaction).await.unwrap();
    }
}
