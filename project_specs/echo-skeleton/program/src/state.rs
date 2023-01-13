use borsh::{BorshDeserialize, BorshSerialize};

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct AuthorizedBufferHeader {
    pub bump_seed: u8,
    pub buffer_seed: u64,
    pub buffer: Vec<u8>,
}

impl AuthorizedBufferHeader {
    pub fn new(bump_seed: u8, buffer_seed: u64, buffer: Vec<u8>) -> Self {
        AuthorizedBufferHeader {
            bump_seed: bump_seed,
            buffer_seed: buffer_seed,
            buffer: buffer,
        }
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct VendingMachineBufferHeader {
    pub bump_seed: u8,
    pub price: u64,
    pub buffer: Vec<u8>,
}

pub const BUMP_SEED_BYTES: i32 = 1;
pub const PRICE_BYTES: i32 = 8;
pub const BUFFER_SIZE_BYTES: i32 = 4;

impl VendingMachineBufferHeader {
    pub const BUFFER_OFFSET_BYTES: i32 = BUMP_SEED_BYTES + PRICE_BYTES + BUFFER_SIZE_BYTES;

    pub fn new(bump_seed: u8, price: u64, buffer_size: usize) -> Self {
        VendingMachineBufferHeader {
            bump_seed: bump_seed,
            price: price,
            buffer: vec![0; buffer_size],
        }
    }
}
