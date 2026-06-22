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

# Codebase Memory — Knowledge Graph Tools

This project uses codebase-memory-mcp to maintain a knowledge graph of the codebase.

## Quick Decision Matrix

| Question                | Tool call                                               |
| ----------------------- | ------------------------------------------------------- |
| Who calls X?            | `trace_path(direction="inbound")`                       |
| What does X call?       | `trace_path(direction="outbound")`                      |
| Full call context       | `trace_path(direction="both")`                          |
| Find by name pattern    | `search_graph(name_pattern="...")`                      |
| Dead code               | `search_graph(max_degree=0, exclude_entry_points=true)` |
| Cross-service edges     | `query_graph` with Cypher                               |
| Impact of local changes | `detect_changes()`                                      |
| Risk-classified trace   | `trace_path(risk_labels=true)`                          |
| Text search             | `search_code` or Grep                                   |

## Exploration Workflow

1. `list_projects` — check if project is indexed
2. `get_graph_schema` — understand node/edge types
3. `search_graph(label="Function", name_pattern=".*Pattern.*")` — find code
4. `get_code_snippet(qualified_name="project.path.FuncName")` — read source

## Tracing Workflow

1. `search_graph(name_pattern=".*FuncName.*")` — discover exact name
2. `trace_path(function_name="FuncName", direction="both", depth=3)` — trace
3. `detect_changes()` — map git diff to affected symbols

## Quality Analysis

- Dead code: `search_graph(max_degree=0, exclude_entry_points=true)`
- High fan-out: `search_graph(min_degree=10, relationship="CALLS", direction="outbound")`
- High fan-in: `search_graph(min_degree=10, relationship="CALLS", direction="inbound")`

## 14 MCP Tools

`index_repository`, `index_status`, `list_projects`, `delete_project`,
`search_graph`, `search_code`, `trace_path`, `detect_changes`,
`query_graph`, `get_graph_schema`, `get_code_snippet`, `get_architecture`,
`manage_adr`, `ingest_traces`

## Edge Types

CALLS, HTTP_CALLS, ASYNC_CALLS, IMPORTS, DEFINES, DEFINES_METHOD,
HANDLES, IMPLEMENTS, OVERRIDE, USAGE, FILE_CHANGES_WITH,
CONTAINS_FILE, CONTAINS_FOLDER, CONTAINS_PACKAGE

## Cypher Examples (for query_graph)

```
MATCH (a)-[r:HTTP_CALLS]->(b) RETURN a.name, b.name, r.url_path, r.confidence LIMIT 20
MATCH (f:Function) WHERE f.name =~ '.*Handler.*' RETURN f.name, f.file_path
MATCH (a)-[r:CALLS]->(b) WHERE a.name = 'main' RETURN b.name
```

## Gotchas

1. `search_graph(relationship="HTTP_CALLS")` filters nodes by degree — use `query_graph` with Cypher to see actual edges.
2. `query_graph` has a 200-row cap — use `search_graph` with degree filters for counting.
3. `trace_path` needs exact names — use `search_graph(name_pattern=...)` first.
4. `direction="outbound"` misses cross-service callers — use `direction="both"`.
5. Results default to 10 per page — check `has_more` and use `offset`.
