use solana_client::rpc_client::RpcClient;
use solana_program::{
    pubkey::Pubkey, 
    system_instruction::create_account, 
    system_program,
};
use solana_sdk::{
    signature::{Keypair, Signer},
    transaction::Transaction,
    commitment_config::CommitmentConfig,
};
use solana_orderbook_monitor::{
    client::{initialize, record_event},
    OrderbookEventType,
};
use std::str::FromStr;

// This example demonstrates how to interact with the orderbook monitor program
fn main() {
    // Connect to the Solana cluster
    let rpc_url = "https://api.devnet.solana.com".to_string();
    let client = RpcClient::new_with_commitment(rpc_url, CommitmentConfig::confirmed());
    
    // Program ID (must match deployed program)
    let program_id = Pubkey::from_str("YOUR_PROGRAM_ID_HERE").unwrap();
    
    // Create a new keypair for the client (this would be your wallet)
    let payer = Keypair::new();
    
    // Create a new keypair for the monitor account
    let monitor_account = Keypair::new();
    
    // Space needed for the monitor account (example estimation)
    let space = 8 + // Discriminator
                1 + // initialized: bool
                32 + // authority: Pubkey
                8 + // event_count: u64
                4 + // events.len() prefixed length
                0; // We start with 0 events
    
    // Calculate rent exemption
    let rent = client.get_minimum_balance_for_rent_exemption(space).unwrap();
    
    // Airdrop some SOL to the payer (for testing on devnet)
    let recent_blockhash = client.get_latest_blockhash().unwrap();
    let airdrop_signature = client.request_airdrop(&payer.pubkey(), 1_000_000_000).unwrap();
    client.confirm_transaction_with_spinner(&airdrop_signature, &recent_blockhash, CommitmentConfig::confirmed()).unwrap();
    
    println!("Airdrop complete!");
    
    // Create the monitor account
    let create_account_ix = create_account(
        &payer.pubkey(),
        &monitor_account.pubkey(),
        rent,
        space as u64,
        &program_id,
    );
    
    // Initialize the monitor
    let init_ix = initialize(
        &program_id,
        &monitor_account.pubkey(),
    );
    
    // Create and send the transaction
    let init_tx = Transaction::new_signed_with_payer(
        &[create_account_ix, init_ix],
        Some(&payer.pubkey()),
        &[&payer, &monitor_account],
        client.get_latest_blockhash().unwrap(),
    );
    
    match client.send_and_confirm_transaction_with_spinner(&init_tx) {
        Ok(sig) => println!("Monitor initialized! Signature: {}", sig),
        Err(e) => {
            eprintln!("Failed to initialize monitor: {}", e);
            return;
        }
    }
    
    // Define a fake market account (in a real scenario, this would be a real DEX market account)
    let market_account = Keypair::new();
    
    // Example: Record an order placed event
    let record_event_ix = record_event(
        &program_id,
        &monitor_account.pubkey(),
        &market_account.pubkey(),
        "SOL/USDC".to_string(),
        2500_000_000, // Price in lamports (e.g., $25.00 with 8 decimals)
        10_000_000,   // Size in lamports (e.g., 0.1 SOL)
        true,         // Is bid (buy order)
        OrderbookEventType::OrderPlaced,
    );
    
    let record_tx = Transaction::new_signed_with_payer(
        &[record_event_ix],
        Some(&payer.pubkey()),
        &[&payer],
        client.get_latest_blockhash().unwrap(),
    );
    
    match client.send_and_confirm_transaction_with_spinner(&record_tx) {
        Ok(sig) => println!("Event recorded! Signature: {}", sig),
        Err(e) => eprintln!("Failed to record event: {}", e),
    }
    
    // Example: Record an order filled event
    let record_filled_ix = record_event(
        &program_id,
        &monitor_account.pubkey(),
        &market_account.pubkey(),
        "SOL/USDC".to_string(),
        2500_000_000, // Price in lamports
        5_000_000,    // Partial fill size
        true,         // Is bid (buy order)
        OrderbookEventType::OrderFilled,
    );
    
    let record_filled_tx = Transaction::new_signed_with_payer(
        &[record_filled_ix],
        Some(&payer.pubkey()),
        &[&payer],
        client.get_latest_blockhash().unwrap(),
    );
    
    match client.send_and_confirm_transaction_with_spinner(&record_filled_tx) {
        Ok(sig) => println!("Fill event recorded! Signature: {}", sig),
        Err(e) => eprintln!("Failed to record fill event: {}", e),
    }
    
    println!("Client example completed successfully!");
} 