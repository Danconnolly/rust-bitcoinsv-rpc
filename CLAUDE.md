# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

### Build and Test
- `cargo build` - Build all workspace members
- `cargo build --verbose` - Build with verbose output
- `cargo test` - Run unit tests for all workspace members
- `cargo test --verbose` - Run tests with verbose output
- `cargo build --examples` - Build examples

### Run a Single Test
- `cargo test test_name` - Run a specific test by name
- `cargo test --package bitcoinsv-rpc test_name` - Run test in specific package
- `cargo test -- --nocapture` - Show print output during tests

### Integration Tests
- `cd integration_test && ./run.sh` - Run integration tests against local Bitcoin SV nodes
- `BSVVERSION=1.0.16 ./integration_test/test_all.sh` - Run integration tests with specific BSV version

### Development
- `cargo check` - Fast type checking without building
- `cargo clippy` - Run Rust linter
- `cargo fmt` - Format code according to Rust standards
- `cargo doc --open` - Generate and open documentation

## Architecture

This is a Rust workspace with three crates:

1. **bitcoinsv-rpc** (client/) - Main RPC client library
   - Provides `Client` struct implementing the `RpcApi` trait
   - Handles HTTP/JSON-RPC communication with Bitcoin SV nodes
   - Supports multiple authentication methods via `Auth` enum
   - Re-exports all types from bitcoinsv-rpc-json

2. **bitcoinsv-rpc-json** (json/) - JSON type definitions
   - Contains all request/response data structures for Bitcoin SV RPC APIs
   - Provides serde serialization/deserialization
   - Independent of the client implementation

3. **integration_test** - Integration testing harness
   - Spins up local Bitcoin SV nodes for testing
   - Tests actual RPC communication against real nodes

### Key Design Patterns

**Authentication**: The client supports flexible authentication through the `Auth` enum:
- `None` - No authentication
- `UserPass(String, String)` - Username/password
- `CookieFile(PathBuf)` - Cookie-based auth for local nodes

**Error Handling**: Custom `Error` type in client/src/error.rs wraps various failure modes including JSON-RPC errors, HTTP errors, and parsing errors.

**API Surface**: All RPC methods are defined in the `RpcApi` trait (client/src/client.rs). The trait uses a generic `call` method that handles JSON serialization/deserialization.

**Type Safety**: Strong typing throughout - Bitcoin SV types (TxHash, BlockHash, etc.) come from the `bitcoinsv` dependency, ensuring type safety across the API boundary.

### Integration Test Setup

The integration tests use a specific setup:
1. Downloads Bitcoin SV binary if not present
2. Starts two regtest nodes that connect to each other
3. Node 1: Basic node on port 12348
4. Node 2: Full node with txindex on port 12349 (used for RPC tests)
5. Uses cookie authentication from the datadir

When developing new RPC methods, ensure you add corresponding integration tests to verify against a real node.