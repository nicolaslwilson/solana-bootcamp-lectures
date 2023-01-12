use borsh::BorshDeserialize;
use borsh::BorshSerialize;
use solana_program::account_info::next_account_info;
use solana_program::program::invoke_signed;
use solana_program::rent::Rent;
use solana_program::system_instruction;
use solana_program::sysvar::Sysvar;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::error::EchoError;
use crate::instruction::EchoInstruction;
use crate::state::AuthorizedBufferHeader;

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

                // msg!(&format!("{:?}", authorized_buffer.data));

                // let initial_data = [&[bump_seed], &buffer_seed.to_le_bytes()[..]].concat();
                let initial_data = AuthorizedBufferHeader {
                    bump_seed: bump_seed,
                    buffer_seed: buffer_seed,
                    buffer: [].to_vec(),
                };
                // let mut buffer_data = authorized_buffer.try_borrow_mut_data()?;
                initial_data.serialize(&mut *authorized_buffer.data.borrow_mut());
                // buffer_data[..].copy_from_slice();

                // buffer_data[0..9].copy_from_slice(&initial_data[..]);

                // msg!(&format!("{:?}", authorized_buffer.data));

                Ok(())
            }
            EchoInstruction::AuthorizedEcho { data: _ } => {
                msg!("Instruction: AuthorizedEcho");
                Err(EchoError::NotImplemented.into())
            }
            EchoInstruction::InitializeVendingMachineEcho {
                price: _,
                buffer_size: _,
            } => {
                msg!("Instruction: InitializeVendingMachineEcho");
                Err(EchoError::NotImplemented.into())
            }
            EchoInstruction::VendingMachineEcho { data: _ } => {
                msg!("Instruction: VendingMachineEcho");
                Err(EchoError::NotImplemented.into())
            }
        }
    }
}
