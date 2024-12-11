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

// Define an enum for proposal status to keep track of where proposals stand in the voting process
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
enum ProposalStatus {
    Pending,    // Proposal is queued but not yet active for voting
    Active,     // Proposal is currently open for voting
    Passed,     // Proposal has passed the vote
    Rejected    // Proposal has been rejected by the vote
}

// Structure for a proposal, capturing all necessary details for voting and status tracking
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct Proposal {
    pub proposal_id: u64,           // Unique identifier for each proposal
    pub proposer: Pubkey,           // The public key of the member who proposed this
    pub description: String,        // A textual description of what the proposal entails
    pub vote_start: i64,            // Unix timestamp marking the start of the voting period
    pub vote_end: i64,              // Unix timestamp marking the end of the voting period
    pub yes_votes: u64,             // Total number of tokens voted 'Yes' for this proposal
    pub no_votes: u64,              // Total number of tokens voted 'No' for this proposal
    pub status: ProposalStatus,     // Current status of the proposal in the voting process
}

// Update the HealthInsuranceDAO structure to include governance capabilities
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct HealthInsuranceDAO {
    pub admin: Pubkey,              // The admin who manages the DAO
    pub members: Vec<Member>,       // List of all members in the DAO
    pub claims: Vec<Claim>,         // List of all claims submitted to the DAO
    pub treasury: Pubkey,           // Address of the treasury account for payouts
    pub proposals: Vec<Proposal>,   // List of all governance proposals within the DAO
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

    // Ensure this program has authority over the account being modified
    if account.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }

    let mut dao_data = HealthInsuranceDAO::try_from_slice(&account.data.borrow())?;

    match instruction_data[0] {
        // ... existing instructions ...
        
        4 => {
            // Create Proposal Instruction - This allows members to propose new actions or changes to the DAO
            let proposer = next_account_info(accounts_iter)?;  // Account of the person proposing
            let description = String::from_utf8(instruction_data[1..].to_vec()).map_err(|_| ProgramError::InvalidInstructionData)?; // Proposal description
            let vote_duration = i64::from_le_bytes(instruction_data[1..9].try_into().unwrap()); // Duration of voting period in seconds

            let now = Clock::get()?.unix_timestamp; // Current time for setting vote start
            dao_data.proposals.push(Proposal {
                proposal_id: dao_data.proposals.len() as u64, // Assign a new ID
                proposer: *proposer.key,
                description,
                vote_start: now,
                vote_end: now + vote_duration, // End time is now plus duration
                yes_votes: 0,
                no_votes: 0,
                status: ProposalStatus::Active, // Proposal starts as active for voting
            });
            msg!("Proposal created with ID: {}", dao_data.proposals.len() - 1);
        }
        
        5 => {
            // Vote on Proposal Instruction - Allows members to cast votes on active proposals
            let voter = next_account_info(accounts_iter)?;      // Account of the voter
            let token_account = next_account_info(accounts_iter)?; // Token account of the voter to check voting power
            let token_program = next_account_info(accounts_iter)?;  // SPL Token program account for token operations
            
            let proposal_index = u64::from_le_bytes(instruction_data[1..9].try_into().unwrap()); // Index of the proposal being voted on
            let vote = instruction_data[9]; // 0 for No vote, 1 for Yes vote

            if let Some(proposal) = dao_data.proposals.get_mut(proposal_index as usize) {
                let current_time = Clock::get()?.unix_timestamp;
                // Check if voting is currently active for this proposal
                if current_time >= proposal.vote_start && current_time <= proposal.vote_end {
                    let token_account_data = TokenAccount::unpack(&token_account.data.borrow())?;
                    let vote_weight = token_account_data.amount; // Number of tokens represents voting power

                    if vote == 0 {
                        proposal.no_votes = proposal.no_votes.checked_add(vote_weight).ok_or(ProgramError::ArithmeticOverflow)?;
                    } else if vote == 1 {
                        proposal.yes_votes = proposal.yes_votes.checked_add(vote_weight).ok_or(ProgramError::ArithmeticOverflow)?;
                    } else {
                        return Err(ProgramError::InvalidInstructionData);
                    }

                    // Check if the voting period has ended to finalize the proposal status
                    if current_time > proposal.vote_end {
                        if proposal.yes_votes > proposal.no_votes {
                            proposal.status = ProposalStatus::Passed;
                        } else {
                            proposal.status = ProposalStatus::Rejected;
                        }
                    }
                    msg!("Vote casted for proposal {} with weight {}", proposal_index, vote_weight);
                } else {
                    return Err(ProgramError::InvalidInstructionData); // Voting period not active
                }
            } else {
                return Err(ProgramError::InvalidAccountData); // Proposal does not exist
            }
        }
        
        _ => return Err(ProgramError::InvalidInstructionData),
    }

    // Save the updated DAO state back into the account's data
    dao_data.serialize(&mut &mut account.data.borrow_mut()[..])?;
    Ok(())
}
