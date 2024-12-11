fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    // Initialize an iterator over the provided accounts
    let accounts_iter = &mut accounts.iter();
    
    // Get the first account, which we expect to be our DAO state account
    let account = next_account_info(accounts_iter)?;

    // Check if this program owns the account we're about to modify
    if account.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }

    // Deserialize the DAO data from the account's data
    let mut dao_data = HealthInsuranceDAO::try_from_slice(&account.data.borrow())?;

    // Use the first byte of instruction_data to determine which operation to perform
    match instruction_data[0] {
        // ... existing instructions ...
        3 => {
            // Mint Tokens Instruction
            // This instruction allows new tokens to be minted into circulation

            // Extract accounts needed for minting: mint address, destination address, mint authority, and token program
            let mint = next_account_info(accounts_iter)?;         // The token mint account
            let to = next_account_info(accounts_iter)?;           // The account to receive the minted tokens
            let authority = next_account_info(accounts_iter)?;    // The account with authority to mint tokens
            let token_program = next_account_info(accounts_iter)?; // The SPL Token program's ID

            // Verify the mint authority is signing this transaction
            if !authority.is_signer {
                return Err(ProgramError::MissingRequiredSignature);
            }

            // Unpack the mint account to get its data
            let mint_info = Mint::unpack_unchecked(&mint.data.borrow())?;
            
            // Extract the amount of tokens to mint from instruction data
            let amount = u64::from_le_bytes(instruction_data[1..9].try_into().unwrap());

            // Call the SPL Token program to mint tokens to the 'to' account
            spl_token::instruction::mint_to(
                token_program.key, // ID of the SPL Token program
                mint.key,          // Mint address
                to.key,            // Destination token account
                authority.key,     // Mint authority
                &[],               // Additional signers (none in this case)
                amount,            // Amount of tokens to mint
            )?;

            // Update the DAO's total supply, ensuring we check for overflow
            if let Some(token_management) = &mut dao_data.token_management {
                token_management.total_supply = token_management.total_supply.checked_add(amount).ok_or(ProgramError::ArithmeticOverflow)?;
            } else {
                // If token management is not set up, we can't proceed with token operations
                return Err(ProgramError::InvalidAccountData); // No token management setup
            }

            // Log the minting action
            msg!("Minted {} tokens to {}", amount, to.key);
        }
        
        4 => {
            // Transfer Tokens Instruction
            // This instruction allows transferring tokens between token accounts

            // Retrieve accounts for transfer operation
            let from = next_account_info(accounts_iter)?;         // Source token account
            let to = next_account_info(accounts_iter)?;           // Destination token account
            let authority = next_account_info(accounts_iter)?;    // Account with authority to transfer
            let token_program = next_account_info(accounts_iter)?; // SPL Token program ID

            // Ensure the authority account is signing the transaction
            if !authority.is_signer {
                return Err(ProgramError::MissingRequiredSignature);
            }

            // Extract the transfer amount from instruction data
            let amount = u64::from_le_bytes(instruction_data[1..9].try_into().unwrap());

            // Use the SPL Token program to execute the transfer
            spl_token::instruction::transfer(
                token_program.key, // Token program ID
                from.key,          // Source token account
                to.key,            // Destination token account
                authority.key,     // Authority to transfer
                &[],               // Additional signers (none here)
                amount,            // Amount to transfer
            )?;

            // Log the transfer action
            msg!("Transferred {} tokens from {} to {}", amount, from.key, to.key);
        }
        
        5 => {
            // Burn Tokens Instruction
            // This allows removing tokens from circulation

            // Get accounts needed for burning: token account to burn from, mint account, authority, and token program
            let token_account = next_account_info(accounts_iter)?; // Token account to burn tokens from
            let mint = next_account_info(accounts_iter)?;          // Mint address of the tokens
            let authority = next_account_info(accounts_iter)?;     // Account with authority to burn tokens
            let token_program = next_account_info(accounts_iter)?; // SPL Token program ID

            // Check if the authority is signing this transaction
            if !authority.is_signer {
                return Err(ProgramError::MissingRequiredSignature);
            }

            // Extract the amount of tokens to burn from instruction data
            let amount = u64::from_le_bytes(instruction_data[1..9].try_into().unwrap());

            // Execute the burn operation through the SPL Token program
            spl_token::instruction::burn(
                token_program.key, // Token program ID
                token_account.key, // Token account to burn from
                mint.key,          // Mint address
                authority.key,     // Burn authority
                &[],               // Additional signers (none here)
                amount,            // Amount to burn
            )?;

            // Decrease the total supply count in our DAO data
            if let Some(token_management) = &mut dao_data.token_management {
                token_management.total_supply = token_management.total_supply.checked_sub(amount).ok_or(ProgramError::ArithmeticOverflow)?;
            } else {
                return Err(ProgramError::InvalidAccountData); // No token management setup
            }

            // Log the burning action
            msg!("Burned {} tokens", amount);
        }
        _ => return Err(ProgramError::InvalidInstructionData), // If the instruction type is unknown
    }

    // Serialize the updated DAO state back into the account's data
    dao_data.serialize(&mut &mut account.data.borrow_mut()[..])?;
    Ok(())
}
