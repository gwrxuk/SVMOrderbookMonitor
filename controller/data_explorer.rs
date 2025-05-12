use solana_client::rpc_client::RpcClient;
use solana_program::{
    pubkey::Pubkey,
    borsh::try_from_slice_unchecked,
};
use solana_orderbook_monitor::{OrderbookMonitor, OrderbookEventType};
use std::str::FromStr;
use std::collections::HashMap;

/// This example demonstrates how to extract and analyze orderbook data from an
/// initialized monitor account
fn main() {
    // Connect to the Solana cluster
    let rpc_url = "https://api.devnet.solana.com".to_string();
    let client = RpcClient::new(rpc_url);
    
    // Replace with your actual monitor account address
    let monitor_address = Pubkey::from_str("YOUR_MONITOR_ACCOUNT_ADDRESS").expect("Invalid monitor address");
    
    // Fetch the account data
    let account = client.get_account(&monitor_address).expect("Failed to fetch monitor account");
    
    // Deserialize the account data
    let monitor: OrderbookMonitor = try_from_slice_unchecked(&account.data)
        .expect("Failed to deserialize monitor account data");
    
    println!("=== Orderbook Monitor Analysis ===");
    println!("Total events recorded: {}", monitor.event_count);
    println!("Authority: {}", monitor.authority);
    println!("");
    
    // Calculate market activity
    let mut markets = HashMap::new();
    for event in &monitor.events {
        let counter = markets.entry(event.market_name.clone()).or_insert(0);
        *counter += 1;
    }
    
    println!("=== Market Activity ===");
    for (market, count) in markets {
        println!("{}: {} events", market, count);
    }
    println!("");
    
    // Calculate event type distribution
    let mut event_types = HashMap::new();
    for event in &monitor.events {
        let event_type = match event.event_type {
            OrderbookEventType::OrderPlaced => "Order Placed",
            OrderbookEventType::OrderFilled => "Order Filled",
            OrderbookEventType::OrderCancelled => "Order Cancelled",
        };
        let counter = event_types.entry(event_type).or_insert(0);
        *counter += 1;
    }
    
    println!("=== Event Type Distribution ===");
    for (event_type, count) in event_types {
        println!("{}: {} events", event_type, count);
    }
    println!("");
    
    // Calculate bid/ask distribution
    let mut bids = 0;
    let mut asks = 0;
    for event in &monitor.events {
        if event.is_bid {
            bids += 1;
        } else {
            asks += 1;
        }
    }
    
    println!("=== Bid/Ask Distribution ===");
    println!("Bids: {} events ({}%)", bids, (bids as f64 / monitor.event_count as f64) * 100.0);
    println!("Asks: {} events ({}%)", asks, (asks as f64 / monitor.event_count as f64) * 100.0);
    println!("");
    
    // Calculate price statistics (for a specific market)
    if !monitor.events.is_empty() {
        let target_market = &monitor.events[0].market_name;
        let mut prices = vec![];
        
        for event in &monitor.events {
            if &event.market_name == target_market {
                prices.push(event.price);
            }
        }
        
        if !prices.is_empty() {
            let min_price = prices.iter().min().unwrap();
            let max_price = prices.iter().max().unwrap();
            let avg_price = prices.iter().sum::<u64>() as f64 / prices.len() as f64;
            
            println!("=== Price Statistics for {} ===", target_market);
            println!("Min price: {}", min_price);
            println!("Max price: {}", max_price);
            println!("Avg price: {:.2}", avg_price);
            println!("");
        }
    }
    
    // Display recent events (last 5)
    println!("=== Recent Events ===");
    for (i, event) in monitor.events.iter().rev().take(5).enumerate() {
        let event_type = match event.event_type {
            OrderbookEventType::OrderPlaced => "Order Placed",
            OrderbookEventType::OrderFilled => "Order Filled",
            OrderbookEventType::OrderCancelled => "Order Cancelled",
        };
        
        println!("Event #{}: {} {} on {} for {} at price {}", 
            monitor.event_count - i as u64,
            if event.is_bid { "BID" } else { "ASK" },
            event_type,
            event.market_name,
            event.size,
            event.price);
    }
} 