# Piggy Bank dApp

A decentralized savings jar built on **Stellar Soroban**. Lock your XLM with a condition — a target amount, a specific date, or both — and the funds are inaccessible until that condition is met. No early withdrawals, no exceptions.

Built as a practice project for the APAC Stellar Hackathon.

---

## Features

| Feature | Description |
|---|---|
| Create a piggy bank | Set a name, choose a lock condition, and deploy your personal savings jar |
| Deposit | Anyone can deposit XLM into a piggy bank |
| Withdraw | Only the owner can withdraw, and only once the condition is fully met |
| Multiple banks | One wallet can hold any number of piggy banks simultaneously |
| Progress tracking | See your balance vs. target and time remaining at a glance |

### Lock Conditions

- **Target Amount** — funds unlock when balance >= the set target
- **Target Time** — funds unlock when the current timestamp >= the set date
- **Both** — funds unlock only when *both* the amount and time conditions are satisfied

---

## Smart Contract

**Language:** Rust + Soroban SDK  
**Network:** Stellar Testnet

### Contract ID (Testnet)

> **Pending deployment** — contract ID will be updated here after deployment.

```
# Piggy Bank contract
CONTRACT_ID=<to be filled after deployment>

# Stellar Testnet RPC
RPC_URL=https://soroban-testnet.stellar.org
NETWORK_PASSPHRASE=Test SDF Network ; September 2015
```

### Contract Interface

```rust
fn create(env, owner, name, token, condition) -> u32   // returns bank ID
fn deposit(env, caller, bank_id, amount)
fn withdraw(env, owner, bank_id)
fn get_bank(env, bank_id) -> PiggyBank
fn get_owner_banks(env, owner) -> Vec<u32>
fn is_unlocked(env, bank_id) -> bool
```

### Data Structures

```rust
pub enum LockCondition {
    TargetAmount(i128),
    TargetTime(u64),
    Both { amount: i128, time: u64 },
}

pub struct PiggyBank {
    pub owner: Address,
    pub name: String,
    pub balance: i128,
    pub token: Address,
    pub condition: LockCondition,
    pub created_at: u64,
    pub is_withdrawn: bool,
}
```

---

## Tech Stack

| Layer | Tech |
|---|---|
| Smart Contract | Rust + Soroban SDK |
| Frontend | Next.js 14 (App Router) |
| Styling | Tailwind CSS v3 |
| Animation | Framer Motion |
| Wallet | Freighter browser extension |
| Stellar SDK | `@stellar/stellar-sdk` + `@stellar/freighter-api` |

---

## Project Structure


---

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) + `wasm32-unknown-unknown` target
- [Stellar CLI](https://developers.stellar.org/docs/smart-contracts/getting-started/setup)
- [Node.js](https://nodejs.org/) 18+
- [pnpm](https://pnpm.io/)
- [Freighter wallet](https://www.freighter.app/) browser extension

### 1. Build & Deploy the Contract

```bash
cd contracts

# Build
cargo build --target wasm32-unknown-unknown --release

# Deploy to testnet
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/piggy_bank.wasm \
  --network testnet \
  --source <your-keypair-alias>
```

Copy the output contract ID and set it in `frontend/.env.local`:

```env
NEXT_PUBLIC_CONTRACT_ID=
```

### 2. Fund Your Testnet Account

```bash
# Get testnet XLM from Friendbot
curl "https://friendbot.stellar.org?addr=<your-public-key>"
```

### 3. Run the Frontend

```bash
cd frontend
pnpm install
pnpm dev
```

Open `http://localhost:3000` and connect your Freighter wallet.

---

## Running Contract Tests

```bash
cd contracts
cargo test
```

Test coverage:

- `create_bank_success`
- `deposit_increases_balance`
- `withdraw_before_condition_fails`
- `withdraw_after_amount_target_met`
- `withdraw_after_time_target_met`
- `withdraw_after_both_conditions_met`
- `non_owner_cannot_withdraw`
- `deposit_to_nonexistent_bank_fails`
- `double_withdraw_fails`
- `get_owner_banks_returns_correct_ids`

---

## Resources

| Resource | Link |
|---|---|
| Soroban Docs | https://developers.stellar.org/docs/smart-contracts |
| Stellar SDK (JS) | https://github.com/stellar/js-stellar-sdk |
| Freighter Docs | https://docs.freighter.app |
| Stellar Testnet Explorer | https://stellar.expert/explorer/testnet |
| Testnet Friendbot | https://friendbot.stellar.org |
