# Verse Design Documentation

Verse is the peer-to-peer, decentralized component of the Graphshell ecosystem. It enables optional sharing, storage, and economic models for reports and collaborative artifacts on top of Graphshell's core graph browser functionality.

## Project Description

Verse (Phase 3+) is an experimental architecture for tokenized reports and networks. It defines storage-backed issuance, governance models, and portability mechanisms for shared knowledge. **Verse is research and not required for the MVP** — it represents the long-term vision for collaborative, decentralized sense-making.

## Current Goals

1. **Specification Phase**: Define JSON schemas for reports and Verse manifests
2. **Economic Modeling**: Design storage-backed issuance and simulate scenarios
3. **Storage Proofs**: Develop Merkle-based proofs for auditability
4. **Prototype**: Build minimal local Verse network (seeders + indexer)
5. **Integration**: Enable Graphshell-to-Verse data portability

## Document Index

### Core Research
- **[VERSE.md](VERSE.md)** — High-level overview of tokenization, token models, peer roles, and storage primitives
- **[SEARCH_FINDINGS_SUMMARY.md](SEARCH_FINDINGS_SUMMARY.md)** — Literature scan and research findings summary

### Collaboration & Interaction
- **[GRAPHSHELL_P2P_COLLABORATION.md](GRAPHSHELL_P2P_COLLABORATION.md)** — P2P collaboration patterns, Graphshell-Verse interaction, and decentralized sync mechanisms

### Archive
See [archive_docs/](../archive_docs/) for superseded analysis and reference material.

## Build & Setup Guidelines

Verse is research-phase code and does not have standalone build guidelines at this time. It integrates with [graphshell_docs/](../graphshell_docs/) where Graphshell build instructions are located.

For Graphshell setup (required foundation):
- **Desktop (Windows)**: [graphshell_docs/BUILD.md](../graphshell_docs/BUILD.md)
- **Implementation Plan**: [graphshell_docs/IMPLEMENTATION_ROADMAP.md](../graphshell_docs/IMPLEMENTATION_ROADMAP.md)

## References

- **Graphshell Core**: [graphshell_docs/README.md](../graphshell_docs/README.md)
- **Graphshell-Verse Interaction**: [GRAPHSHELL_P2P_COLLABORATION.md](GRAPHSHELL_P2P_COLLABORATION.md)
- **Research Agenda**: [VERSE.md](VERSE.md)

## Research Stages

1. **Phase 1-2** (Graphshell MVP): Core graph browser — *Graphshell focus*
2. **Phase 3** (Verse MVP): Tokenization spec and minimal network — *Verse primary focus*
3. **Phase 4+** (Scale): Economic simulation and full governance — *Joint development*
