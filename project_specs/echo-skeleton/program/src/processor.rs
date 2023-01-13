use borsh::BorshDeserialize;
use borsh::BorshSerialize;
use solana_program::account_info::next_account_info;
use solana_program::program::invoke;
use solana_program::program::invoke_signed;
use solana_program::rent::Rent;
use solana_program::system_instruction;
use solana_program::sysvar::Sysvar;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey,
};
use spl_token;

use crate::error::EchoError;
use crate::instruction::EchoInstruction;
use crate::state::AuthorizedBufferHeader;
use crate::state::VendingMachineBufferHeader;

pub struct Processor {}

impl Processor {
    pub fn process_instruction(
        _program_id: &Pubkey,
        _accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        let instruction = EchoInstruction::try_from_slice(instruction_data)
            .map_err(|_| ProgramError::InvalidInstructionData)?;

        let mut account_info_iter = _accounts.iter();

        match instruction {
            EchoInstruction::Echo { data } => {
                msg!("Instruction: EchoInstruction");

                let echo_buffer = next_account_info(&mut account_info_iter)?;

                let mut data_dest = echo_buffer.try_borrow_mut_data()?;

                &msg!(&format!("{:?}", data_dest));

                let data_dest_len = data_dest.len();

                msg!(&data_dest_len.to_string());

                if data_dest[0] != 0 {
                    return Err(EchoError::AccountAlreadyWritten.into());
                }

                let write_flag: &[u8] = &[1];
                let data_to_write = [write_flag, &data[0..data_dest_len - 1]].concat();
                data_dest[..].copy_from_slice((&data_to_write[..]));

                &msg!(&format!("{:?}", data_dest));

                Ok(())
            }
            EchoInstruction::InitializeAuthorizedEcho {
                buffer_seed,
                buffer_size,
            } => {
                msg!("Instruction: InitializeAuthorizedEcho");
                let authorized_buffer = next_account_info(&mut account_info_iter)?;
                let authority = next_account_info(&mut account_info_iter)?;
                let system_program = next_account_info(&mut account_info_iter)?;
                let (expected_buffer_address, bump_seed) = Pubkey::find_program_address(
                    &[
                        b"authority",
                        authority.key.as_ref(),
                        &buffer_seed.to_le_bytes(),
                    ],
                    _program_id,
                );

                if (expected_buffer_address != *authorized_buffer.key) {
                    return Err(ProgramError::InvalidArgument);
                }

                let rent = Rent::get()?;
                let rent_lamports = rent.minimum_balance(buffer_size);

                let initial_data =
                    AuthorizedBufferHeader::new(bump_seed, buffer_seed, vec![0; buffer_size - 13]);

                invoke_signed(
                    &system_instruction::create_account(
                        authority.key,
                        authorized_buffer.key,
                        rent_lamports,
                        buffer_size as u64,
                        _program_id,
                    ),
                    &[
                        system_program.clone(), // program being invoked also needs to be included
                        authorized_buffer.clone(),
                        authority.clone(),
                    ],
                    &[&[
                        b"authority",
                        authority.key.as_ref(),
                        &buffer_seed.to_le_bytes(),
                        &bump_seed.to_le_bytes(),
                    ]],
                )?;

                // let initial_data = [&[bump_seed], &buffer_seed.to_le_bytes()[..]].concat();

                // let mut buffer_data = authorized_buffer.try_borrow_mut_data()?;
                initial_data.serialize(&mut &mut authorized_buffer.data.borrow_mut()[..])?;
                // buffer_data[..].copy_from_slice();

                // buffer_data[0..9].copy_from_slice(&initial_data[..]);

                Ok(())
            }
            EchoInstruction::AuthorizedEcho { data } => {
                msg!("Instruction: AuthorizedEcho");
                let authorized_buffer_info = next_account_info(&mut account_info_iter)?;
                let authority_info = next_account_info(&mut account_info_iter)?;

                msg!(&format!("{:?}", authorized_buffer_info.data));

                let mut authorized_buffer = AuthorizedBufferHeader::try_from_slice(
                    &authorized_buffer_info.try_borrow_mut_data()?,
                )?;

                // Verify authority has signed the tx
                if (!authority_info.is_signer) {
                    return Err(ProgramError::MissingRequiredSignature);
                }

                let (expected_buffer_address, _) = Pubkey::find_program_address(
                    &[
                        b"authority",
                        authority_info.key.as_ref(),
                        &authorized_buffer.buffer_seed.to_le_bytes(),
                    ],
                    _program_id,
                );

                if (expected_buffer_address != *authorized_buffer_info.key) {
                    msg!(&expected_buffer_address.to_string());
                    return Err(ProgramError::InvalidArgument);
                }

                authorized_buffer.buffer = data;
                authorized_buffer.serialize(&mut *authorized_buffer_info.data.borrow_mut())?;

                Ok(())
            }
            EchoInstruction::InitializeVendingMachineEcho { price, buffer_size } => {
                msg!("Instruction: InitializeVendingMachineEcho");
                let vending_machine_buffer_info = next_account_info(&mut account_info_iter)?;
                let vending_machine_token_mint_info = next_account_info(&mut account_info_iter)?;
                let payer_info = next_account_info(&mut account_info_iter)?;
                let system_program_info = next_account_info(&mut account_info_iter)?;

                let (expected_buffer_address, bump_seed) = Pubkey::find_program_address(
                    &[
                        b"authority",
                        vending_machine_token_mint_info.key.as_ref(),
                        &price.to_le_bytes(),
                    ],
                    _program_id,
                );

                if (expected_buffer_address != *vending_machine_buffer_info.key) {
                    return Err(ProgramError::InvalidArgument);
                }

                let vending_buffer_size = buffer_size
                    .checked_add(VendingMachineBufferHeader::BUFFER_OFFSET_BYTES as usize)
                    .ok_or(ProgramError::InvalidArgument)?;

                let rent = Rent::get()?;
                let rent_lamports = rent.minimum_balance(vending_buffer_size);

                // msg!(&vending_buffer_size.to_string());
                // msg!(&vending_machine_buffer_info.data_len().to_string()); // is 0 before creation

                invoke_signed(
                    &system_instruction::create_account(
                        payer_info.key,
                        vending_machine_buffer_info.key,
                        rent_lamports,
                        vending_buffer_size as u64,
                        _program_id,
                    ),
                    &[
                        system_program_info.clone(), // program being invoked also needs to be included
                        vending_machine_buffer_info.clone(),
                        payer_info.clone(),
                    ],
                    &[&[
                        b"authority",
                        vending_machine_token_mint_info.key.as_ref(),
                        &price.to_le_bytes(),
                        &bump_seed.to_le_bytes(),
                    ]],
                )?;

                let initial_data = VendingMachineBufferHeader::new(bump_seed, price, buffer_size);

                // msg!(&format!("{:?}", initial_data.try_to_vec()));
                // msg!(&format!("{:?}", initial_data.try_to_vec()?.len()));
                // msg!(&vending_machine_buffer_info.data_len().to_string()); // is now = vending_buffer_size

                initial_data
                    .serialize(&mut &mut vending_machine_buffer_info.data.borrow_mut()[..])?;
                Ok(())
            }
            EchoInstruction::VendingMachineEcho { data } => {
                msg!("Instruction: VendingMachineEcho");
                let vending_machine_buffer_info = next_account_info(&mut account_info_iter)?;
                let user_info = next_account_info(&mut account_info_iter)?;
                let user_ata_info = next_account_info(&mut account_info_iter)?;
                let vending_machine_token_mint_info = next_account_info(&mut account_info_iter)?;
                let token_program_info = next_account_info(&mut account_info_iter)?;

                let mut vending_machine_buffer = VendingMachineBufferHeader::try_from_slice(
                    &vending_machine_buffer_info.try_borrow_mut_data()?,
                )?;

                let (expected_buffer_address, bump_seed) = Pubkey::find_program_address(
                    &[
                        b"authority",
                        vending_machine_token_mint_info.key.as_ref(),
                        &vending_machine_buffer.price.to_le_bytes(),
                    ],
                    _program_id,
                );

                if (expected_buffer_address != *vending_machine_buffer_info.key) {
                    return Err(ProgramError::InvalidArgument);
                }

                // let token_mint = spl_token::state::Mint::try_from(
                //     &vending_machine_token_mint_info.data.try_borrow()?,
                // )?;

                invoke(
                    &spl_token::instruction::burn(
                        token_program_info.key,
                        user_ata_info.key,
                        vending_machine_token_mint_info.key,
                        user_info.key,
                        &[user_info.key],
                        vending_machine_buffer.price,
                    )?,
                    &[
                        token_program_info.clone(),
                        user_ata_info.clone(),
                        vending_machine_token_mint_info.clone(),
                        user_info.clone(),
                    ],
                )?;

                let buffer_len = vending_machine_buffer.buffer.len();

                vending_machine_buffer.buffer = vec![0; buffer_len];
                vending_machine_buffer.buffer = data[0..buffer_len.min(data.len())].try_to_vec()?;

                vending_machine_buffer
                    .serialize(&mut &mut vending_machine_buffer_info.data.borrow_mut()[..])?;

                Ok(())
            }
        }
    }
}
