# Solana Orderbook Monitor

A Solana program designed to monitor and record orderbook activity on Solana-based decentralized exchanges (DEXs).

## Overview

This project implements a Solana smart contract that can:

1. Monitor orderbook events (orders placed, filled, or cancelled)
2. Record event data on-chain for later analysis
3. Track prices, sizes, and timestamps for market activity

## Project Structure

- `src/lib.rs`: Core program logic
- `controller/client.rs`: Client for interacting with the program
- `controller/data_explorer.rs`: Utility to analyze recorded orderbook data

## Features

- Initialize an orderbook monitor account
- Record various types of orderbook events:
  - Order placed
  - Order filled
  - Order cancelled
- Store metadata about each event:
  - Timestamp
  - Market name
  - Price
  - Size
  - Direction (bid/ask)
- Analyze orderbook data:
  - Market activity distribution
  - Event type distribution
  - Bid/ask distribution
  - Price statistics

## Getting Started

### Prerequisites

- Rust and Cargo
- Solana CLI tools
- A Solana wallet with SOL (for deployment and testing)

### Building

```bash
cargo build-bpf
```

### Deploying

```bash
solana program deploy target/deploy/solana_orderbook_monitor.so
```

### Testing with the Client

1. Update the program ID in `controller/client.rs` with your deployed program ID
2. Run the example:

```bash
cargo run --example client
```

### Analyzing Recorded Data

To analyze orderbook data that has been recorded:

1. Update the monitor account address in `controller/data_explorer.rs`
2. Run the data explorer:

```bash
cargo run --example data_explorer
```

## Extending the Program

### Adding New Event Types

You can extend the `OrderbookEventType` enum in `src/lib.rs` to support additional event types specific to your requirements:

```rust
pub enum OrderbookEventType {
    OrderPlaced,
    OrderFilled,
    OrderCancelled,
    // Add your custom event types here
    LiquidityAdded,
    LiquidityRemoved,
    // etc.
}
```

## Use Cases

- Market data analytics
- Trading strategy development
- Price feed oracles
- Historical market data archive

## License

MIT

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. 