# Verse — Phase 3 research: tokenization, networks, economics

Purpose
- Define an optional, experimental architecture for sharing and valuing report-like artifacts derived from user activity. This is research; it is not required for the MVP.

Token model (high level)
- Report token (NFT-style): immutable record of a report (URL, selector, metadata, timestamp). Portable and exportable; can remain private or be contributed to a network.
- Verse fungible token: per-verse utility token used for storage-backed issuance, query rights, rewards, and governance.

Peer roles (you can participate in multiple roles!)
- Users: create and optionally publish reports.
- Seeders/rebroadcasters: host report storage and serve data.
- Indexers/deduplicators: dedupe and index reports for efficient queries.
- Attesters/validators: provide attestations or integrity checks.
- Curators: create and govern Verses; stake tokens for governance privileges.

Storage and economic primitives
- Storage-backed fungible token issuance: token issuance rates tied to amount of storage provided in realtime (so very low rates tied to time thresholds) and host reputation (uptime + recent activity + peers/seeders).
- Access models: selling or trading data you create and own would be obvious, as is renting it cryptographically, providing access but not ownership, or hosting your data on someone else's storage generally.
- Decay model: data value would decline over time to favor recent contributions, with perhaps a few community defined exceptions.

Governance and portability
- Each Verse uses token-weighted governance; curators stake to create Verses and propose rate/rule changes.
- Users can fork a Verse and migrate data if governance diverges.
- Reports use a JSON schema and standard metadata to ensure portability across clients.

Research agenda (next steps)
- Specify on-disk JSON schemas for reports and Verse manifests.
- Design Merkle-based proofs of storage and simple proofs-of-history for auditability.
- Model storage-backed issuance and simulate adversarial scenarios.
- Prototype a minimal local Verse network (seeders + indexer + UI flows).

Notes
- Keep tokenization and P2P sync optional and separate from the core product. Prioritize privacy and portability in schema design.
