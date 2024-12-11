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

// Define an enum for claim status
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
enum ClaimStatus {
    Pending,
    Verified,
    Rejected,
    Paid
}

// Enhanced claim structure
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct Claim {
    pub claim_id: u64,           // Unique identifier for each claim
    pub member: Pubkey,          // The member who submitted the claim
    pub amount: u64,             // The amount of the claim in lamports
    pub service_date: i64,       // Date of the medical service or event
    pub service_type: String,    // Type of medical service or event
    pub provider: Pubkey,        // The provider's public key
    pub status: ClaimStatus,     // Current status of the claim
    pub verifiers: Vec<Pubkey>,  // List of oracles or verifiers who have checked this claim
}

// Main DAO structure with additional fields
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct HealthInsuranceDAO {
    pub admin: Pubkey,           // The admin who manages the DAO
    pub members: Vec<Member>,    // List of all members in the DAO
    pub claims: Vec<Claim>,      // List of all claims submitted to the DAO
    pub treasury: Pubkey,        // Address of the treasury account for payouts
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

    if account.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }

    let mut dao_data = HealthInsuranceDAO::try_from_slice(&account.data.borrow())?;

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
            let provider = next_account_info(accounts_iter)?;
            let amount = u64::from_le_bytes(instruction_data[1..9].try_into().unwrap());
            let service_date = i64::from_le_bytes(instruction_data[9..17].try_into().unwrap());
            let service_type = String::from_utf8(instruction_data[17..].to_vec()).map_err(|_| ProgramError::InvalidInstructionData)?;

            dao_data.claims.push(Claim {
                claim_id: dao_data.claims.len() as u64,
                member: *member.key,
                amount,
                service_date,
                service_type,
                provider: *provider.key,
                status: ClaimStatus::Pending,
                verifiers: Vec::new(),
            });
            msg!("Claim submitted for {} lamports", amount);
        }
        2 => {
            // Instruction for verifying a claim
            let verifier = next_account_info(accounts_iter)?;
            let claim_index = u64::from_le_bytes(instruction_data[1..9].try_into().map_err(|_| ProgramError::InvalidInstructionData)?);

            if let Some(claim) = dao_data.claims.get_mut(claim_index as usize) {
                match claim.status {
                    ClaimStatus::Pending => {
                        claim.verifiers.push(*verifier.key);
                        if claim.verifiers.len() >= 2 { // Example: Require at least two verifications
                            claim.status = ClaimStatus::Verified;
                        }
                        msg!("Claim {} verification in progress. Verifiers: {}", claim.claim_id, claim.verifiers.len());
                    },
                    _ => return Err(ProgramError::InvalidAccountData), // Claim should not be verified twice
                }
            } else {
                return Err(ProgramError::InvalidAccountData);
            }
        }
        3 => {
            // Instruction for paying out a verified claim
            let treasury = next_account_info(accounts_iter)?;
            let member_account = next_account_info(accounts_iter)?;
            let system_program = next_account_info(accounts_iter)?;
            
            let claim_index = u64::from_le_bytes(instruction_data[1..9].try_into().map_err(|_| ProgramError::InvalidInstructionData)?);
            if let Some(claim) = dao_data.claims.get_mut(claim_index as usize) {
                if claim.status == ClaimStatus::Verified {
                    // Here, we'd typically transfer funds. Since this is a simulation:
                    msg!("Transferring {} lamports from treasury to {}", claim.amount, member_account.key);
                    // In real scenarios, use Solana's `invoke` to call the system program for transfer
                    claim.status = ClaimStatus::Paid;
                } else {
                    return Err(ProgramError::InvalidAccountData); // Claim must be verified before payout
                }
            } else {
                return Err(ProgramError::InvalidAccountData);
            }
        }
        _ => return Err(ProgramError::InvalidInstructionData),
    }

    dao_data.serialize(&mut &mut account.data.borrow_mut()[..])?;
    Ok(())
}
