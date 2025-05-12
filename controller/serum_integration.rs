use solana_client::rpc_client::RpcClient;
use solana_program::{
    pubkey::Pubkey,
    system_instruction::create_account,
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

/// This example demonstrates how to integrate the orderbook monitor with Serum DEX
fn main() {
    // Connect to the Solana cluster
    let rpc_url = "https://api.devnet.solana.com".to_string();
    let client = RpcClient::new_with_commitment(rpc_url, CommitmentConfig::confirmed());
    
    // Program ID (must match deployed program)
    let program_id = Pubkey::from_str("YOUR_PROGRAM_ID_HERE").unwrap();
    
    // Create a new keypair for the client
    let payer = Keypair::new();
    
    // Create a new keypair for the monitor account
    let monitor_account = Keypair::new();
    
    // Space needed for the monitor account (larger to accommodate more events)
    let space = 8 + // Discriminator
                1 + // initialized: bool
                32 + // authority: Pubkey
                8 + // event_count: u64
                4 + // events.len() prefixed length
                (1000 * (8 + // timestamp
                      50 + // market_name (max length)
                      8 + // price
                      8 + // size
                      1 + // is_bid
                      1)); // event_type
    
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
    
    // Set up Serum market accounts for monitoring
    // Note: In a real implementation, you would use the actual Serum market accounts
    
    // For example purposes, we'll just use fake accounts
    let serum_market_sol_usdc = Keypair::new().pubkey();
    
    println!("Starting to monitor SOL/USDC Serum market: {}", serum_market_sol_usdc);
    
    // Simulating monitoring of orderbook events
    // In a real scenario, you would set up a websocket connection to monitor events
    
    // Simulate a bid order being placed
    record_orderbook_event(
        &client,
        &payer,
        &program_id,
        &monitor_account.pubkey(),
        &serum_market_sol_usdc,
        "SOL/USDC",
        2500_000_000, // $25.00
        10_000_000,   // 0.1 SOL
        true,         // bid (buy)
        OrderbookEventType::OrderPlaced,
    );
    
    // Simulate an ask order being placed
    record_orderbook_event(
        &client,
        &payer,
        &program_id,
        &monitor_account.pubkey(),
        &serum_market_sol_usdc,
        "SOL/USDC",
        2600_000_000, // $26.00
        20_000_000,   // 0.2 SOL
        false,        // ask (sell)
        OrderbookEventType::OrderPlaced,
    );
    
    // Simulate a partial fill of the bid
    record_orderbook_event(
        &client,
        &payer,
        &program_id,
        &monitor_account.pubkey(),
        &serum_market_sol_usdc,
        "SOL/USDC",
        2500_000_000, // $25.00
        5_000_000,    // 0.05 SOL
        true,         // bid (buy)
        OrderbookEventType::OrderFilled,
    );
    
    // Add more Serum markets as needed
    let serum_market_btc_usdc = Keypair::new().pubkey();
    println!("Starting to monitor BTC/USDC Serum market: {}", serum_market_btc_usdc);
    
    // Simulate a Bitcoin order
    record_orderbook_event(
        &client,
        &payer,
        &program_id,
        &monitor_account.pubkey(),
        &serum_market_btc_usdc,
        "BTC/USDC",
        50000_000_000, // $50,000.00
        1_000_000,     // 0.01 BTC
        true,          // bid (buy)
        OrderbookEventType::OrderPlaced,
    );
    
    println!("Serum orderbook monitoring example completed!");
    println!("Monitor account: {}", monitor_account.pubkey());
}

fn record_orderbook_event(
    client: &RpcClient,
    payer: &Keypair,
    program_id: &Pubkey,
    monitor_account: &Pubkey,
    market_account: &Pubkey,
    market_name: &str,
    price: u64,
    size: u64,
    is_bid: bool,
    event_type: OrderbookEventType,
) {
    let record_ix = record_event(
        program_id,
        monitor_account,
        market_account,
        market_name.to_string(),
        price,
        size,
        is_bid,
        event_type.clone(),
    );
    
    let record_tx = Transaction::new_signed_with_payer(
        &[record_ix],
        Some(&payer.pubkey()),
        &[payer],
        client.get_latest_blockhash().unwrap(),
    );
    
    match client.send_and_confirm_transaction_with_spinner(&record_tx) {
        Ok(sig) => {
            let event_type_str = match event_type {
                OrderbookEventType::OrderPlaced => "placed",
                OrderbookEventType::OrderFilled => "filled",
                OrderbookEventType::OrderCancelled => "cancelled",
            };
            println!(
                "Recorded {} {} order on {} for {} at price {}! Signature: {}", 
                if is_bid { "bid" } else { "ask" },
                event_type_str,
                market_name,
                size,
                price,
                sig
            );
        },
        Err(e) => eprintln!("Failed to record event: {}", e),
    }
} 