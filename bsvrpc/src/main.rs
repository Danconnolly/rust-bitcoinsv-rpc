use anyhow::{Context, Result};
use bitcoinsv_rpc::{Auth, Client, RpcApi};
use bitcoinsv::bitcoin::{BlockHash, TxHash};
use hex::FromHex;
use clap::{Parser, Subcommand};
use serde_json::Value;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "bsvrpc")]
#[command(about = "Bitcoin SV RPC command-line client", long_about = None)]
struct Cli {
    #[arg(short, long, default_value = "http://localhost:8332")]
    url: String,

    #[arg(short = 'U', long)]
    user: Option<String>,

    #[arg(short = 'p', long)]
    password: Option<String>,

    #[arg(short = 'c', long)]
    cookie_file: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Get blockchain info")]
    GetBlockchainInfo,

    #[command(about = "Get best block hash")]
    GetBestBlockHash,

    #[command(about = "Get block count")]
    GetBlockCount,

    #[command(about = "Get difficulty")]
    GetDifficulty,

    #[command(about = "Get network info")]
    GetNetworkInfo,

    #[command(about = "Get peer info")]
    GetPeerInfo,

    #[command(about = "Get mining info")]
    GetMiningInfo,

    #[command(about = "Get block hash by height")]
    GetBlockHash {
        height: u64,
    },

    #[command(about = "Get block by hash")]
    GetBlock {
        hash: String,
        #[arg(short, long, default_value = "1")]
        verbosity: u8,
    },

    #[command(about = "Get raw transaction")]
    GetRawTransaction {
        txid: String,
        #[arg(short, long)]
        verbose: bool,
    },

    #[command(about = "Get transaction output")]
    GetTxOut {
        txid: String,
        vout: u32,
        #[arg(short, long)]
        include_mempool: bool,
    },

    #[command(about = "Get mempool info")]
    GetMempoolInfo,

    #[command(about = "Get connection count")]
    GetConnectionCount,


    #[command(about = "Send raw JSON-RPC command")]
    Raw {
        method: String,
        #[arg(help = "JSON array of parameters")]
        params: Option<String>,
    },
}

fn get_auth(cli: &Cli) -> Auth {
    if let Some(cookie_file) = &cli.cookie_file {
        Auth::CookieFile(cookie_file.clone())
    } else if let (Some(user), Some(pass)) = (&cli.user, &cli.password) {
        Auth::UserPass(user.clone(), pass.clone())
    } else {
        Auth::None
    }
}

fn print_json(value: impl serde::Serialize) -> Result<()> {
    let json = serde_json::to_string_pretty(&value)?;
    println!("{}", json);
    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Check if the URL contains credentials (has @ symbol after ://)
    let client = if cli.url.contains("://") && cli.url.split("://").nth(1).map_or(false, |s| s.contains('@')) {
        // Use new_from_uri for URLs with embedded credentials
        Client::new_from_uri(&cli.url, None).context("Failed to create RPC client")?
    } else {
        // Use the traditional method with separate auth
        let auth = get_auth(&cli);
        Client::new(&cli.url, auth, None).context("Failed to create RPC client")?
    };

    match &cli.command {
        Commands::GetBlockchainInfo => {
            let info = client.get_blockchain_info()?;
            print_json(info)?;
        }
        Commands::GetBestBlockHash => {
            let hash = client.get_best_block_hash()?;
            print_json(hash)?;
        }
        Commands::GetBlockCount => {
            let count = client.get_block_count()?;
            print_json(count)?;
        }
        Commands::GetDifficulty => {
            let difficulty = client.get_difficulty()?;
            print_json(difficulty)?;
        }
        Commands::GetNetworkInfo => {
            let info = client.get_network_info()?;
            print_json(info)?;
        }
        Commands::GetPeerInfo => {
            let peers = client.get_peer_info()?;
            print_json(peers)?;
        }
        Commands::GetMiningInfo => {
            let info = client.get_mining_info()?;
            print_json(info)?;
        }
        Commands::GetBlockHash { height } => {
            let hash = client.get_block_hash(*height)?;
            print_json(hash)?;
        }
        Commands::GetBlock { hash, verbosity } => {
            let hash_bytes = Vec::<u8>::from_hex(hash)
                .context("Invalid hex in block hash")?;
            let block_hash = BlockHash::from_slice(&hash_bytes);
            match verbosity {
                0 => {
                    let block_hex = client.get_block_hex(&block_hash)?;
                    print_json(block_hex)?;
                }
                1 => {
                    let block = client.get_block_info(&block_hash)?;
                    print_json(block)?;
                }
                _ => {
                    let block = client.get_block_info(&block_hash)?;
                    print_json(block)?;
                }
            }
        }
        Commands::GetRawTransaction { txid, verbose } => {
            let hash_bytes = Vec::<u8>::from_hex(txid)
                .context("Invalid hex in transaction ID")?;
            let tx_hash = TxHash::from_slice(&hash_bytes);
            if *verbose {
                let tx = client.get_raw_transaction_info(&tx_hash, None)?;
                print_json(tx)?;
            } else {
                let tx_hex = client.get_raw_transaction_hex(&tx_hash, None)?;
                print_json(tx_hex)?;
            }
        }
        Commands::GetTxOut { txid, vout, include_mempool } => {
            let hash_bytes = Vec::<u8>::from_hex(txid)
                .context("Invalid hex in transaction ID")?;
            let tx_hash = TxHash::from_slice(&hash_bytes);
            let result = client.get_tx_out(&tx_hash, *vout, Some(*include_mempool))?;
            print_json(result)?;
        }
        Commands::GetMempoolInfo => {
            let info = client.get_mempool_info()?;
            print_json(info)?;
        }
        Commands::GetConnectionCount => {
            let count = client.get_connection_count()?;
            print_json(count)?;
        }
        Commands::Raw { method, params } => {
            let params_value = if let Some(params_str) = params {
                serde_json::from_str::<Vec<Value>>(params_str)
                    .context("Failed to parse params as JSON array")?
            } else {
                vec![]
            };
            
            let result: Value = client.call(method, &params_value)?;
            print_json(result)?;
        }
    }

    Ok(())
}