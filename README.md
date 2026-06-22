# Allotropy Finance

A full-stack CosmWasm token launch and curve-bonding application. It combines Rust-based CosmWasm smart contracts for token management and bonding curve logic with a SvelteKit frontend for interactive user interfaces.

## Tech Stack & Project Structure

- **Frontend**: Svelte 5, SvelteKit, Tailwind CSS v4, Skeleton, Bun (package manager), Vitest (testing)
- **Smart Contracts (CosmWasm / Rust)**:
  - `contracts/cw20-liquid-bond`: Main liquid bonding curve contract.


  Confio contracts that were ported to latest version, slightly modified and partially re-used
  - `contracts/cw20-bonding`: Standard CW20 token contract.
  - `contracts/cw20-base`: Standard CW20 token contract.

## Developer Workflows

### Prerequisites

- [Rust](https://rustup.rs/) (for contracts build & lint)
- [Bun](https://bun.sh/) (for SvelteKit web dev)
- [Docker](https://www.docker.com/) (to run a local chain environment)

### Smart Contracts

Compile the contracts:

```bash
cargo build
```

Run linter:

```bash
cargo clippy --all-targets -- -D warnings
```

Format code:

```bash
cargo fmt
```

Build optimized CosmWasm artifacts:

```bash
make optimize
```

### Local Dev Chain

Start the local CosmWasm Cosmos chain container:

```bash
make local-chain
```

This builds of a local-gaia image and runs it with test accounts configured in `config/accounts.json`.

Port mappings:

- RPC: `http://localhost:26657`
- gRPC: `http://localhost:9090`
- REST API: `http://localhost:1317`

### SvelteKit Web App

Install dependencies:

```bash
bun install
```

Start the Svelte development server:

```bash
bun dev
```

Run Unit/E2E Tests:

```bash
bun test
```

Check linting:

```bash
bun run lint
```

Format code:

```bash
bun run format
```

---