use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
    clock::Clock,
    sysvar::Sysvar,
};
use thiserror::Error;

// Define program errors
#[derive(Error, Debug, Copy, Clone)]
pub enum OrderbookError {
    #[error("Invalid instruction")]
    InvalidInstruction,
    #[error("Invalid account owner")]
    InvalidOwner,
    #[error("Account already initialized")]
    AlreadyInitialized,
}

impl From<OrderbookError> for ProgramError {
    fn from(e: OrderbookError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

// Define instruction types
#[derive(BorshSerialize, BorshDeserialize, Debug, PartialEq)]
pub enum OrderbookInstruction {
    /// Initialize a new orderbook monitor
    /// Accounts expected:
    /// 0. `[writable]` The orderbook monitor account to initialize
    Initialize,
    
    /// Record a new orderbook event
    /// Accounts expected:
    /// 0. `[writable]` The orderbook monitor account
    /// 1. `[]` Market account or other relevant account to monitor
    RecordEvent {
        market_name: String,
        price: u64,
        size: u64,
        is_bid: bool,
        event_type: OrderbookEventType,
    },
}

#[derive(BorshSerialize, BorshDeserialize, Debug, PartialEq, Clone)]
pub enum OrderbookEventType {
    OrderPlaced,
    OrderFilled,
    OrderCancelled,
}

// Define the orderbook monitor account structure
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct OrderbookMonitor {
    pub initialized: bool,
    pub authority: Pubkey,
    pub event_count: u64,
    pub events: Vec<OrderbookEvent>,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct OrderbookEvent {
    pub timestamp: i64,
    pub market_name: String,
    pub price: u64,
    pub size: u64,
    pub is_bid: bool,
    pub event_type: OrderbookEventType,
}

// Program entrypoint
entrypoint!(process_instruction);

// Program logic
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = OrderbookInstruction::try_from_slice(instruction_data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    match instruction {
        OrderbookInstruction::Initialize => {
            process_initialize(program_id, accounts)
        },
        OrderbookInstruction::RecordEvent { market_name, price, size, is_bid, event_type } => {
            process_record_event(program_id, accounts, market_name, price, size, is_bid, event_type)
        },
    }
}

fn process_initialize(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let monitor_account = next_account_info(account_info_iter)?;

    // Check account ownership
    if monitor_account.owner != program_id {
        msg!("Monitor account does not have the correct program id");
        return Err(OrderbookError::InvalidOwner.into());
    }

    // Get authority (the first account is also the authority in this simple case)
    let authority = monitor_account.key;

    // Initialize the monitor account
    let monitor = OrderbookMonitor {
        initialized: true,
        authority: *authority,
        event_count: 0,
        events: Vec::new(),
    };

    monitor.serialize(&mut *monitor_account.data.borrow_mut())?;
    
    msg!("Orderbook monitor initialized");
    Ok(())
}

fn process_record_event(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    market_name: String,
    price: u64,
    size: u64,
    is_bid: bool,
    event_type: OrderbookEventType,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let monitor_account = next_account_info(account_info_iter)?;
    let _market_account = next_account_info(account_info_iter)?;

    // Check account ownership
    if monitor_account.owner != program_id {
        msg!("Monitor account does not have the correct program id");
        return Err(OrderbookError::InvalidOwner.into());
    }

    // Get the current clock for timestamp
    let clock = Clock::get()?;
    
    // Load the monitor account data
    let mut monitor = OrderbookMonitor::try_from_slice(&monitor_account.data.borrow())?;

    // Create a new event
    let event = OrderbookEvent {
        timestamp: clock.unix_timestamp,
        market_name,
        price,
        size,
        is_bid,
        event_type,
    };

    // Record the event
    monitor.events.push(event.clone());
    monitor.event_count += 1;

    // Save the updated monitor account
    monitor.serialize(&mut *monitor_account.data.borrow_mut())?;

    // Log the event
    msg!("Orderbook event recorded: {:?}", event);
    Ok(())
}

// Client-side helpers
#[cfg(not(feature = "no-entrypoint"))]
pub mod client {
    use super::*;
    use solana_program::instruction::{AccountMeta, Instruction};

    pub fn initialize(
        program_id: &Pubkey,
        monitor_account: &Pubkey,
    ) -> Instruction {
        Instruction {
            program_id: *program_id,
            accounts: vec![
                AccountMeta::new(*monitor_account, true),
            ],
            data: OrderbookInstruction::Initialize.try_to_vec().unwrap(),
        }
    }

    pub fn record_event(
        program_id: &Pubkey,
        monitor_account: &Pubkey,
        market_account: &Pubkey,
        market_name: String,
        price: u64,
        size: u64,
        is_bid: bool,
        event_type: OrderbookEventType,
    ) -> Instruction {
        Instruction {
            program_id: *program_id,
            accounts: vec![
                AccountMeta::new(*monitor_account, true),
                AccountMeta::new_readonly(*market_account, false),
            ],
            data: OrderbookInstruction::RecordEvent {
                market_name,
                price,
                size,
                is_bid,
                event_type,
            }
            .try_to_vec()
            .unwrap(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_program::{
        program_pack::Pack,
        pubkey::Pubkey,
        rent::Rent,
        account_info::AccountInfo,
    };
    use solana_program::signer::keypair::Keypair;
    use std::mem::size_of;

    // A test helper function that creates a monitor account for testing
    fn create_monitor_account(lamports: u64, data_len: usize) -> (Keypair, AccountInfo) {
        let owner = Pubkey::new_unique();
        let key = Keypair::new();
        let mut lamports_ref = lamports;
        let mut data = vec![0; data_len];
        
        let account_info = AccountInfo::new(
            &key.pubkey(),
            false,
            true,
            &mut lamports_ref,
            &mut data,
            &owner,
            false,
            Rent::default().last_slot_of_epoch(0),
        );
        
        (key, account_info)
    }

    #[test]
    fn test_initialize() {
        let program_id = Pubkey::new_unique();
        
        // Create a monitor account
        let (_, monitor_account) = create_monitor_account(
            Rent::default().minimum_balance(size_of::<OrderbookMonitor>()),
            size_of::<OrderbookMonitor>(),
        );
        
        // Set the owner to the program id for this test
        // Note: This is a bit of a hack for testing, as we can't easily 
        // modify the account_info's owner field directly
        let mut owner = program_id;
        let monitor_account = AccountInfo::new(
            monitor_account.key,
            monitor_account.is_signer,
            monitor_account.is_writable,
            monitor_account.lamports,
            monitor_account.data,
            &owner,
            monitor_account.executable,
            monitor_account.rent_epoch,
        );
        
        let accounts = vec![monitor_account];
        
        // Test the initialize function
        let result = process_initialize(&program_id, &accounts);
        
        // Verify it worked
        assert!(result.is_ok());
        
        // Verify the account was initialized properly
        let monitor = OrderbookMonitor::try_from_slice(&accounts[0].data.borrow()).unwrap();
        assert!(monitor.initialized);
        assert_eq!(monitor.authority, *accounts[0].key);
        assert_eq!(monitor.event_count, 0);
        assert!(monitor.events.is_empty());
    }

    #[test]
    fn test_record_event() {
        let program_id = Pubkey::new_unique();
        
        // Create monitor account with pre-initialized data
        let (_, monitor_account) = create_monitor_account(
            Rent::default().minimum_balance(1000), // Larger to accommodate events
            1000, // Larger data size to accommodate events
        );
        
        // Create market account
        let market_key = Pubkey::new_unique();
        let market_account = AccountInfo::new(
            &market_key,
            false,
            false,
            &mut 100000,
            &mut vec![0; 10],
            &Pubkey::new_unique(),
            false,
            Rent::default().last_slot_of_epoch(0),
        );
        
        // Set the owner to the program id for this test
        let mut owner = program_id;
        let monitor_account = AccountInfo::new(
            monitor_account.key,
            monitor_account.is_signer,
            monitor_account.is_writable,
            monitor_account.lamports,
            monitor_account.data,
            &owner,
            monitor_account.executable,
            monitor_account.rent_epoch,
        );
        
        // Initialize the monitor first
        let accounts = vec![monitor_account.clone()];
        let _ = process_initialize(&program_id, &accounts);
        
        // Now test recording an event
        let accounts = vec![monitor_account, market_account];
        let result = process_record_event(
            &program_id,
            &accounts,
            "BTC/USDC".to_string(),
            50000_00000000, // $50,000.00 with 8 decimals
            1_00000000,     // 1 BTC
            true,           // Is bid
            OrderbookEventType::OrderPlaced,
        );
        
        // Verify it worked
        assert!(result.is_ok());
        
        // Verify the event was recorded
        let monitor = OrderbookMonitor::try_from_slice(&accounts[0].data.borrow()).unwrap();
        assert_eq!(monitor.event_count, 1);
        assert_eq!(monitor.events.len(), 1);
        assert_eq!(monitor.events[0].market_name, "BTC/USDC");
        assert_eq!(monitor.events[0].price, 50000_00000000);
        assert_eq!(monitor.events[0].size, 1_00000000);
        assert!(monitor.events[0].is_bid);
        assert!(matches!(monitor.events[0].event_type, OrderbookEventType::OrderPlaced));
    }
}
