# 0002. Channel-Based Crawl Engine with Dedicated Storage Writer

Date: 2026-04-07

## Status

Accepted

## Context

The crawl engine must handle concurrent HTTP fetching, CPU-intensive HTML parsing, and high-throughput SQLite writes without blocking any single component. SQLite supports only one concurrent writer in WAL mode, and mixing CPU-bound work with async I/O on the same runtime causes latency spikes and throughput degradation.

## Decision

We will use a channel-based actor model with three distinct runtimes:

1. **tokio** for async I/O (HTTP fetching, event emission, Tauri commands)
2. **rayon** for CPU-bound work (HTML parsing, rule evaluation)
3. A **dedicated OS thread** for all SQLite writes, receiving batched commands over a bounded `mpsc` channel

The orchestrator runs on tokio and coordinates the pipeline: dequeue URL from frontier -> fetch on tokio -> parse on rayon -> send StorageCommand to writer thread. The storage writer accumulates commands into batches of 200 and flushes them in a single SQLite transaction.

## Consequences

### Positive

- No WAL contention: a single writer thread eliminates lock conflicts
- tokio is never blocked by CPU-bound parsing or synchronous SQLite calls
- Batched transactions provide 10-50x write throughput over single-row inserts
- The bounded channel (capacity 5000) provides natural backpressure when the writer falls behind

### Negative

- `FlushAck` synchronization is required before any code that reads data written by the pipeline (e.g., post-crawl analysis). Forgetting this leads to stale reads.
- Three runtimes increase cognitive complexity for contributors
- The rayon-to-tokio boundary requires oneshot channels for result passing

## Alternatives Considered

### Alternative 1: Single tokio Runtime

Run everything on tokio, including parsing and SQLite writes.

Rejected because CPU-bound parsing starves the async executor, reducing fetch concurrency. SQLite's synchronous API would require `spawn_blocking` for every write, losing batching benefits.

### Alternative 2: In-Process Queue (e.g., crossbeam)

Use crossbeam channels instead of tokio mpsc.

Rejected because the orchestrator needs async select over both the frontier and the state watch channel. tokio's mpsc integrates with the async runtime; crossbeam would require a polling adapter.
