# src/data/ — Test Data Generation

## Purpose

Generates deterministic, realistic transaction data for FashionForward's payment routing engine. Used by performance reports, demos, and integration tests. All output is fully reproducible across runs via seeded RNG.

## Files

| File | Role |
|------|------|
| `mod.rs` | Data generator — public API (`generate_test_data`, `get_test_dataset`), internal helpers, constants, and unit tests |

## Public API

| Function | Description |
|----------|-------------|
| `generate_test_data(count: usize)` | Generate `count` transactions with seeded RNG |
| `get_test_dataset()` | Returns the canonical 210-transaction dataset (calls `generate_test_data(210)`) |

## Data Distribution

### Countries (equal thirds)

| Country | Currency | Transactions (in 210 set) | BINs |
|---------|----------|---------------------------|------|
| Brazil | BRL | 70 | `411111`, `510510`, `376411` |
| Mexico | MXN | 70 | `424242`, `551234`, `371449` |
| Colombia | COP | 70 | `431940`, `520082`, `378282` |

BINs are realistic but fake. Each transaction gets a randomly selected BIN from its country's set.

### Customers (15 unique, weighted)

| Tier | IDs | % of transactions |
|------|-----|-------------------|
| Heavy users | `cust_001`–`cust_003` | ~30% |
| Mid-tier | `cust_004`–`cust_008` | ~30% |
| Long-tail | `cust_009`–`cust_015` | ~40% |

### Amounts ($10–$500, rounded to cents)

| Bracket | Range | Weight |
|---------|-------|--------|
| Small | $10–$100 | 40% |
| Medium | $100–$300 | 35% |
| Large | $300–$500 | 25% |

### Timestamps

All transactions fall on `2025-01-15`, spread linearly across the 08:00–20:00 UTC business window. Minutes and seconds are randomized.

## Deterministic Seeding

- **Seed:** `42` (constant `DATA_SEED`)
- **RNG:** `rand::rngs::StdRng::seed_from_u64`
- Calling any generator function multiple times always produces identical output. This is verified by the `test_is_deterministic` test.
- **Do not change the seed or generation order** without updating all dependent snapshot-based tests and reports.

## Conventions

- Transaction IDs follow the format `txn_NNNN` (1-indexed, zero-padded to 4 digits).
- Country assignment uses round-robin (`i % 3`), not random selection — this guarantees exact equal distribution.
- All amounts are `f64` rounded to 2 decimal places via `round_to_cents`.
- Keep this module free of I/O or network calls — it is pure in-memory generation.
