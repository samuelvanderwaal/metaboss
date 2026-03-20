use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output};
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{bail, Context, Result};
use regex::Regex;
use serde_json::json;
use solana_client::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::native_token::LAMPORTS_PER_SOL;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;

/// Programs loaded into the test validator.
const BPF_PROGRAMS: &[(&str, &str)] = &[
    (
        "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s",
        "tests/fixtures/programs/token_metadata.so",
    ),
    (
        "CoREENxT6tW1HoK8ypY1SxRMZTcVPm7R94rH4PZNhX7d",
        "tests/fixtures/programs/mpl_core.so",
    ),
    (
        "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
        "tests/fixtures/programs/spl_token.so",
    ),
    (
        "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb",
        "tests/fixtures/programs/spl_token_2022.so",
    ),
    (
        "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL",
        "tests/fixtures/programs/associated_token_account.so",
    ),
];

/// How long to wait for the validator to become healthy.
const VALIDATOR_STARTUP_TIMEOUT: Duration = Duration::from_secs(30);

/// How much SOL to airdrop to the test keypair.
const AIRDROP_SOL: u64 = 10;

/// Output captured from a metaboss command invocation.
pub struct CommandOutput {
    pub stdout: String,
    pub stderr: String,
    pub success: bool,
}

/// Integration test context that manages a local validator, funded keypair, and
/// temp directories. Cleans up on drop.
pub struct TestContext {
    pub rpc_url: String,
    pub client: RpcClient,
    pub keypair_path: String,
    pub keypair: Keypair,
    validator_process: Child,
    temp_dir: PathBuf,
}

impl TestContext {
    /// Spin up a test validator, generate and fund a keypair, and return a
    /// ready-to-use test context.
    pub fn new() -> Result<Self> {
        // Create a unique temp directory for the validator ledger and keypair file.
        // We use the process ID and a timestamp to avoid collisions.
        let temp_dir = std::env::temp_dir().join(format!(
            "metaboss-test-{}-{}",
            std::process::id(),
            Instant::now().elapsed().as_nanos()
        ));
        fs::create_dir_all(&temp_dir).context("Failed to create temp directory")?;

        let ledger_dir = temp_dir.join("ledger");
        let keypair_path = temp_dir.join("test-keypair.json");

        // Generate a fresh keypair and write it to disk in the format solana CLI expects
        // (a JSON array of bytes).
        let keypair = Keypair::new();
        let keypair_bytes: Vec<u8> = keypair.to_bytes().to_vec();
        let keypair_json = serde_json::to_string(&keypair_bytes)?;
        fs::write(&keypair_path, &keypair_json)?;

        let rpc_url = "http://localhost:8899".to_string();

        // Build validator command with all BPF programs.
        let mut cmd = Command::new("solana-test-validator");
        cmd.arg("--ledger")
            .arg(&ledger_dir)
            .arg("--reset")
            .arg("--quiet");

        for (address, so_path) in BPF_PROGRAMS {
            cmd.arg("--bpf-program").arg(address).arg(so_path);
        }

        // Redirect stdout/stderr so the validator doesn't pollute test output.
        cmd.stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null());

        let validator_process = cmd
            .spawn()
            .context("Failed to start solana-test-validator")?;

        let client = RpcClient::new_with_commitment(rpc_url.clone(), CommitmentConfig::confirmed());

        // Wait for the validator to be ready.
        Self::wait_for_validator(&client)?;

        // Fund the test keypair.
        Self::airdrop_and_confirm(&client, &keypair)?;

        Ok(Self {
            rpc_url,
            client,
            keypair_path: keypair_path.to_string_lossy().to_string(),
            keypair,
            validator_process,
            temp_dir,
        })
    }

    /// Poll the validator's RPC endpoint until it responds or the timeout is reached.
    fn wait_for_validator(client: &RpcClient) -> Result<()> {
        let start = Instant::now();
        loop {
            if start.elapsed() > VALIDATOR_STARTUP_TIMEOUT {
                bail!("Timed out waiting for solana-test-validator to start");
            }
            // get_latest_blockhash is a good readiness check: it requires the
            // validator to have produced at least one block.
            if client.get_latest_blockhash().is_ok() {
                return Ok(());
            }
            thread::sleep(Duration::from_millis(200));
        }
    }

    /// Airdrop SOL and wait for confirmation.
    fn airdrop_and_confirm(client: &RpcClient, keypair: &Keypair) -> Result<()> {
        let sig = client
            .request_airdrop(&keypair.pubkey(), AIRDROP_SOL * LAMPORTS_PER_SOL)
            .context("Airdrop request failed")?;

        // Poll until the airdrop transaction is confirmed.
        let start = Instant::now();
        loop {
            if start.elapsed() > Duration::from_secs(15) {
                bail!("Timed out waiting for airdrop confirmation");
            }
            if client.confirm_transaction(&sig).unwrap_or(false) {
                return Ok(());
            }
            thread::sleep(Duration::from_millis(200));
        }
    }

    /// Run metaboss with the given arguments. `--rpc` and the RPC URL are
    /// automatically prepended so tests don't need to specify them.
    pub fn run_metaboss(&self, args: &[&str]) -> CommandOutput {
        let metaboss_bin = Self::metaboss_bin_path();

        let mut full_args = vec!["--rpc", &self.rpc_url];
        full_args.extend_from_slice(args);

        let output: Output = Command::new(&metaboss_bin)
            .args(&full_args)
            .output()
            .unwrap_or_else(|e| panic!("Failed to run metaboss at {:?}: {}", metaboss_bin, e));

        CommandOutput {
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            success: output.status.success(),
        }
    }

    /// Resolve the path to the built metaboss binary.
    fn metaboss_bin_path() -> PathBuf {
        // In integration tests built by cargo, the binary lives alongside the
        // test executable in target/<profile>/deps/. We walk up from the test
        // binary to find target/<profile>/metaboss.
        let mut path = std::env::current_exe().expect("cannot determine test binary path");
        // current_exe -> target/debug/deps/test_binary
        path.pop(); // deps/
        path.pop(); // debug/ (or release/)
        path.push("metaboss");
        if path.exists() {
            return path;
        }
        // Fallback: try from project root.
        PathBuf::from("./target/debug/metaboss")
    }

    /// Write a test NFT metadata JSON file with the test keypair's address
    /// filled in as the creator.
    pub fn create_test_nft_json(&self, path: &Path) -> Result<()> {
        let creator_address = self.keypair.pubkey().to_string();
        let metadata = json!({
            "name": "Test NFT",
            "symbol": "TNFT",
            "uri": "https://arweave.net/FPGAv1XnyZidnqquOdEbSY6_ES735ckcDTdaAtI7GFw",
            "seller_fee_basis_points": 100,
            "creators": [
                {
                    "address": creator_address,
                    "verified": false,
                    "share": 100
                }
            ]
        });

        let mut file = fs::File::create(path)?;
        file.write_all(serde_json::to_string_pretty(&metadata)?.as_bytes())?;
        Ok(())
    }
}

impl Drop for TestContext {
    fn drop(&mut self) {
        // Kill the validator process. We use kill() rather than waiting for
        // graceful shutdown since this is test cleanup.
        let _ = self.validator_process.kill();
        let _ = self.validator_process.wait();
        // Clean up the temp directory. Best-effort; ignore errors.
        let _ = fs::remove_dir_all(&self.temp_dir);
    }
}

/// Panic with stderr contents if the command was not successful.
pub fn assert_success(output: &CommandOutput) {
    assert!(
        output.success,
        "Command failed.\nstdout:\n{}\nstderr:\n{}",
        output.stdout, output.stderr,
    );
}

/// Extract a mint pubkey from metaboss output matching "Mint account: <PUBKEY>".
pub fn parse_mint_from_output(output: &str) -> String {
    let re = Regex::new(r"Mint account: (\S+)").expect("invalid regex");
    re.captures(output)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().to_string())
        .expect("Could not find 'Mint account: <pubkey>' in output")
}
