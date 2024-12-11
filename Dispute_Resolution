use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar::{clock::Clock, Sysvar},
};

// Define structure for a dispute within the DAO
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct Dispute {
    pub dispute_id: u64,                // Unique identifier for the dispute
    pub claim_id: Option<u64>,          // Optional link to a specific claim this dispute relates to
    pub initiator: Pubkey,              // Public key of the member initiating the dispute
    pub respondent: Pubkey,             // Public key of the member or entity the dispute is against
    pub description: String,            // Detailed explanation of the dispute
    pub status: DisputeStatus,          // Current status of the dispute
    pub votes: Vec<(Pubkey, bool)>,     // Collection of votes where each tuple contains the voter's key and their vote (true for supporting the initiator, false otherwise)
}

// Enum to represent the status of a dispute
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
enum DisputeStatus {
    Open,   // Dispute is still open and accepting votes
    Closed, // Dispute has been resolved or voting has concluded
}

// Extend HealthInsuranceDAO structure to manage disputes
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct HealthInsuranceDAO {
    // ... existing fields ...
    pub disputes: Vec<Dispute>,         // Array to hold all disputes within the DAO
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

    // Verify program ownership of the account
    if account.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }

    let mut dao_data = HealthInsuranceDAO::try_from_slice(&account.data.borrow())?;

    match instruction_data[0] {
        // ... existing instructions ...

        7 => {
            // Submit a Dispute - Allows members to raise disputes within the DAO
            let initiator = next_account_info(accounts_iter)?; // Account of the member starting the dispute
            let respondent = next_account_info(accounts_iter)?; // Account of the member or entity being disputed against
            let description = String::from_utf8(instruction_data[1..].to_vec()).map_err(|_| ProgramError::InvalidInstructionData)?; // Text describing the dispute

            dao_data.disputes.push(Dispute {
                dispute_id: dao_data.disputes.len() as u64, // Assign a new ID
                claim_id: None, // Optional field, set to None if not claim-related
                initiator: *initiator.key,
                respondent: *respondent.key,
                description,
                status: DisputeStatus::Open, // New disputes start as open
                votes: Vec::new(), // No votes yet
            });
            msg!("Dispute submitted with ID: {}", dao_data.disputes.len() - 1);
        }

        8 => {
            // Vote on a Dispute - Allows members to cast votes on existing disputes
            let voter = next_account_info(accounts_iter)?; // Account of the member voting
            let dispute_index = u64::from_le_bytes(instruction_data[1..9].try_into().unwrap()); // Index of the dispute in the disputes vector
            let vote = instruction_data[9] != 0; // Boolean interpretation of vote: 1 (true) for agreeing with initiator, 0 (false) for disagreeing

            if let Some(dispute) = dao_data.disputes.get_mut(dispute_index as usize) {
                if dispute.status == DisputeStatus::Open {
                    // Ensure voter hasn't voted on this dispute before
                    if !dispute.votes.iter().any(|(v, _)| v == voter.key) {
                        dispute.votes.push((*voter.key, vote));
                        msg!("Vote cast on dispute {}", dispute.dispute_id);
                    } else {
                        return Err(ProgramError::InvalidArgument); // Voter has already voted on this dispute
                    }

                    // Logic to close the dispute based on vote count
                    if dispute.votes.len() > 5 { // Example threshold, could be more dynamic or based on DAO size
                        dispute.status = DisputeStatus::Closed;
                        msg!("Dispute {} closed due to sufficient votes", dispute.dispute_id);
                        
                        // Simple majority vote to decide outcome
                        let agree_count = dispute.votes.iter().filter(|(_, v)| *v).count();
                        if agree_count * 2 > dispute.votes.len() {
                            msg!("Dispute {} resolved in favor of initiator", dispute.dispute_id);
                        } else {
                            msg!("Dispute {} resolved against initiator", dispute.dispute_id);
                        }
                    }
                } else {
                    return Err(ProgramError::InvalidInstructionData); // Attempt to vote on a closed dispute
                }
            } else {
                return Err(ProgramError::InvalidAccountData); // Dispute not found
            }
        }

        _ => return Err(ProgramError::InvalidInstructionData),
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
    async fn test_dispute_submission() {
        // Test setup and dispute submission logic goes here
        // For example, initializing a DAO, submitting a dispute, checking if it was added correctly
    }

    #[tokio::test]
    async fn test_voting_on_dispute() {
        // Test setup and voting on dispute logic goes here
        // For example, submitting votes, checking if votes are recorded, and if the dispute closes correctly
    }
}
