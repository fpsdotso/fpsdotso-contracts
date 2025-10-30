# fpsdotso-contracts

Solana Anchor smart contracts for FPS.so. These programs back the real‑time, skill‑based FPS gameplay, lobby/matchmaking, and map registry. Designed to work with MagicBlock Ephemeral Rollups (ER) for low‑latency, gasless inputs.

## Prerequisites

- Rust + Anchor CLI
- Solana CLI
- Node.js / yarn or pnpm (workspace tooling)

## Build & Deploy

Build all:

```bash
anchor build
```

Deploy to local validator:

```bash
anchor deploy
```

Alternatively, deploy a single artifact (example map_registry):

```bash
solana program deploy target/deploy/map_registry.so \
  --url http://127.0.0.1:8899 \
  --keypair ~/.config/solana/id.json
```

Run tests:

```bash
anchor test
```

## MagicBlock (ER) Notes

- The `game` program is fed by MagicBlock Ephemeral Rollups for ~10ms input latency and gasless UX.
- The frontend initializes an ephemeral wallet and submits ER transactions against `game` using generated IDLs in `target/idl/`.

## Directory

```
programs/
  game/
  matchmaking/
  map_registry/
tests/
target/idl/
target/deploy/
```

## Environment

Point to your desired cluster:

```bash
solana config set --url http://127.0.0.1:8899     # local
solana config set --url https://api.devnet.solana.com  # devnet
```

## One‑liners (examples)

```bash
ANCHOR_PROVIDER_URL=http://127.0.0.1:8899 \
ANCHOR_WALLET=~/.config/solana/id.json \
yarn test:map-registry
```

## License

MIT (or your preferred license)
