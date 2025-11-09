# Stellar Project (Real Estate)
Smart contracts can be used to record property ownership of any structure. They can also optimize transaction speed and efficiency by reducing the need for lawyers or brokers, allowing sellers to take charge of the process.

Smart contracts have also been used in the industry to automate and secure processes like:

Property Transactions: Automatically execute property sales when conditions like payment confirmation are met, reducing the need for intermediaries.
Lease Agreements: Automate rental agreements and payments, ensuring timely transactions and reducing disputes.
Escrow Services: Securely hold funds until all terms of a real estate deal are fulfilled, releasing them automatically upon completion.
Title Management: Record property ownership and transfer titles securely on the blockchain, enhancing transparency and reducing fraud.

## ✅ Prerequisites
- **Rust (stable)** and `cargo`
- Recommended installation via `rustup`: https://rustup.rs
- Add Wasm target (component model):
```bash
rustup target add wasm32v1-none
```
- **Stellar CLI** (new version has command `stellar contract ...`)
- Test: `stellar --version`
- Used to build/deploy/invoke Soroban contract.
- (Optional) **Make** to run targets in `contracts/hello-world/Makefile`.
- **Testnet** account (CLI will have command to create key and fund faucet).

> Note: The project uses **`soroban-sdk`** in the workspace (original `Cargo.toml` file), compatible with the new toolchain `wasm32v1-none`. If your environment does not support it, update Rust to the latest stable version.

## 🛠️ Setup & Installation
1. Install Rust and target:
```bash
rustup update
rustup target add wasm32v1-none
```

2. Install Stellar CLI (depending on environment):
- macOS (Homebrew): `brew install stellar-cli` *(package name may vary by distro)*
- Or download the official binary for your operating system and add it to `PATH`.
- Confirm: `stellar --version`

3. (Optional) Prepare environment variables for Testnet:
```bash
export RPC_URL="https://soroban-testnet.stellar.org:443"
export NETWORK_PASSPHRASE="Test SDF Network ; September 2015"
```

## 🧱 Build
From the project root directory (the innermost layer containing `Cargo.toml` and `contracts/`):
```bash
cd contracts/hello-world
# Method 1: use Makefile
make build

# Method 2: use Stellar CLI directly
stellar contract build
# Artifact will be located at: target/wasm32v1-none/release/hello_world.wasm
```

Run test & format:
```bash
make test
make fmt
```

## 🚀 Deploy
Example deploy to **Testnet** using Stellar CLI:

1. Generate/import key and fund faucet:
```bash
# Generate key alias for deployer
stellar keys generate --alias deployer
# Fund test lumens (if CLI supports)
stellar account fund --alias deployer --network testnet
```

2. Deploy contract:
```bash
# Make sure you are in contracts/hello-world folder or specify .wasm path
WASM=target/wasm32v1-none/release/hello_world.wasm

# Deploy and save alias for contract
stellar contract deploy --wasm $WASM --alias realestate --network testnet --source deployer

# Get Contract ID (may differ depending on CLI version)
CONTRACT_ID=$(stellar contract id --alias realestate)
echo $CONTRACT_ID
```

> If your CLI uses other parameters (e.g. `--id` / `--wasm-hash`), adjust accordingly. Some CLIs support listing with `stellar contract list`.

## 📚 Usage Guide (Invoke)
The contract defines the main functions in `src/lib.rs`:

- `register_property(property_id: BytesN<32>, owner: Address, metadata: Symbol)`
- `get_property(property_id: BytesN<32>) -> Option<(Address, Symbol)>`
- `transfer_property(invoker: Address, property_id: BytesN<32>, new_owner: Address)`
- `create_lease(lease_id: BytesN<32>, property_id: BytesN<32>, tenant: Address, start_ts: i64, end_ts: i64, rent: i64)`
- `get_lease(lease_id: BytesN<32>) -> Option<(..., i32)>`
- `end_lease(invoker: Address, lease_id: BytesN<32>)`
- `create_escrow(escrow_id: BytesN<32>, property_id: BytesN<32>, buyer: Address, seller: Address, arbiter: Address, amount: i64)`
- `release_escrow(invoker: Address, escrow_id: BytesN<32>, transfer_ownership: bool)`
- `cancel_escrow(invoker: Address, escrow_id: BytesN<32>)`
- `get_escrow(escrow_id: BytesN<32>) -> Option<(..., u32)>`

> **Suggested parameters:**
> - `BytesN<32>`: transmit hex format `0x...` 32 bytes long.
> - `Address`: transmit the Stellar account address (starting with `G...`) or other contract.
> - `Symbol`: pass short string, for example `"HOUSE"`, `"APT"`, `"LAND"`…

### Get address to pass to `Address`
```bash
OWNER=$(stellar keys address --alias deployer)
echo $OWNER
```

### Example invoke command
Set common variable:
```bash
CONTRACT_ID=${CONTRACT_ID:-$(stellar contract id --alias realestate)}
```

1) **Register property**:
```bash
PROPERTY_ID=0x11223344556677889900aabbccddeeff11223344556677889900aabbccddeeff
stellar contract invoke --id "$CONTRACT_ID" --fn register_property --arg property_id="$PROPERTY_ID" --arg owner="$OWNER" --arg metadata="HOUSE" --network testnet --source deployer
```

2) **View property information**:
```bash
stellar contract invoke --id "$CONTRACT_ID" --fn get_property --arg property_id="$PROPERTY_ID" --network testnet
```

3) **Asset transfer**:
```bash
NEW_OWNER=$(stellar keys generate --alias buyer --silent && stellar keys address --alias buyer)
stellar account fund --alias buyer --network testnet

stellar contract invoke --id "$CONTRACT_ID" --fn transfer_property --arg invoker="$OWNER" --arg property_id="$PROPERTY_ID" --arg new_owner="$NEW_OWNER" --network testnet --source deployer
```

4) **Create lease**:
```bash
LEASE_ID=0xa1a2a3a4a5a6a7a8a9aaabacadaeaf00b1b2b3b4b5b6b7b8b9babbbcbdbebfc0
TENANT="$NEW_OWNER"
START_TS=1731000000 # Unix seconds
END_TS=1738680000 # Unix seconds
RENT=1000

stellar contract invoke --id "$CONTRACT_ID" --fn create_lease --arg lease_id="$LEASE_ID" --arg property_id="$PROPERTY_ID" --arg tenant="$TENANT" --arg start_ts="$START_TS" --arg end_ts="$END_TS" --arg rent="$RENT" --network testnet --source deployer
```

5) **ESCROW (create, release, destroy)**:
```bash
ESCROW_ID=0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee
BUYER="$NEW_OWNER"
SELLER="$OWNER"
ARBITER="$OWNER" # example: deployer as arbiter
AMOUNT=5000

# Create escrow
stellar contract invoke --id "$CONTRACT_ID" --fn create_escrow --arg escrow_id="$ESCROW_ID" --arg property_id="$PROPERTY_ID" --arg buyer="$BUYER" --arg seller="$SELLER" --arg arbiter="$ARBITER" --arg amount="$AMOUNT" --network testnet --source deployer

# Release escrow (arbiter confirmation), optionally transferring ownership property
stellar contract invoke --id "$CONTRACT_ID" --fn release_escrow --arg invoker="$ARBITER" --arg escrow_id="$ESCROW_ID" --arg transfer_ownership=true --network testnet --source deployer

# Cancel escrow (if needed)
stellar contract invoke --id "$CONTRACT_ID" --fn cancel_escrow --arg invoker="$ARBITER" --arg escrow_id="$ESCROW_ID" --network testnet --source deployer
```

> **Tip**: Some CLIs support printing output in JSON format with the `--output json` or `-o json` flags.

## 🔧 Build/Deploy Commands summary
- Build: `stellar contract build` or `make build`
- Test: `make test`
- Deploy: `stellar contract deploy --wasm target/wasm32v1-none/release/hello_world.wasm --alias realestate --network testnet --source deployer`
- Invoke: `stellar contract invoke --id $(stellar contract id --alias realestate) --fn <function> --arg <...>`

## ❓ Troubleshooting
- **Target `wasm32v1-none` not found**: run `rustup target add wasm32v1-none` and update Rust.
- **Stellar CLI missing**: install CLI and check `stellar --version`.
- **Network / RPC error**: make sure to use `--network testnet` correctly or set `RPC_URL`, `NETWORK_PASSPHRASE` to suit your environment.

- **Incorrect parameter type**: check the function signature in `src/lib.rs` and the value format `BytesN<32>`, `Address`, `Symbol`.

---


Overwriting existing alias "increment" that currently links to contract ID: CC3XSYJBRUXDH4VFL6QUV72OSNRJ6CVQLR47DFMSXORJNIEYNOS4J32O

Link: 🔗 https://stellar.expert/explorer/testnet/contract/CAXNAMUA7MENEN2BJQSBVWPTGX2RAJPKYZSYQAXVRKVFZIT7EZJZZT7T


# Thành Viên Nhóm 
* Trần Văn Khiên  MSV : 22014013
* Vũ Xuân Hoan    MSV : 22010404
* Trần Long Khánh MSV : 22010449
