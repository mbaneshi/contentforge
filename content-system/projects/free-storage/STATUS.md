# FreeStorage — Content Kit

## Project Summary
Open-source disk storage analyzer and manager. CLI + TUI + Web Dashboard. "Know exactly what's eating your disk."

## Key Facts
- **Repo**: https://github.com/mbaneshi-labs/free-storage-app
- **Stack**: Rust (18 crates), SQLite FTS5, Ratatui, Axum, SvelteKit 5
- **Architecture**: Sync core, async presentations; zero inter-feature dependencies
- **Install**: `cargo install freestorage`
- **License**: MIT

## 18 Crates
`core`, `finder`, `grep`, `fuzzy`, `usage`, `preview`, `listing`, `system`, `procs`, `disks`, `codestats`, `cleanup`, `daemon`, `tui`, `server`, `cli` + more

## Unique Selling Points (use in content)
1. **Three interfaces** — CLI for scripting, TUI for power users, Web for visual exploration
2. **Three-phase dedup** — group by size → xxh3 partial hash → blake3 full hash (minimizes I/O)
3. **Feature-gated binary** — install only what you need: `--features find,grep`
4. **18 Rust crates** — clean workspace architecture, each crate is independent
5. **Embedded SvelteKit** — single binary serves the web dashboard
6. **Pipe-friendly** — one path per line, `--json` on all commands

## Content Hooks (angles that get engagement)
- "I built a disk analyzer with 18 Rust crates and a web UI in a single binary"
- "Three-phase dedup: how to find duplicates without hashing every file"
- "Feature-gated Rust binary — users install only the features they need"
- "From Rust workspace to embedded SvelteKit — shipping a full-stack app as one binary"
- "Why I chose SQLite FTS5 over everything else for file search"

## Launch Status
- [x] Core architecture and 18 crates scaffolded
- [x] Scanner, indexer, searcher, dedup implemented
- [ ] TUI implementation
- [ ] Web dashboard pages
- [ ] Filesystem watcher daemon
- [ ] Draft launch post (Substack)
- [ ] Draft X thread
- [ ] Record demo video

## Content Pipeline
| # | Type | Title Idea | Status |
|---|------|-----------|--------|
| 1 | build-story | "18 Rust crates, SvelteKit embedded, SQLite FTS5 — anatomy of FreeStorage." | Planned |
| 2 | launch-post | "Know exactly what's eating your disk — open-source, zero cloud." | Planned |
| 3 | results-post | "I scanned my 2TB drive. Here's what FreeStorage found." | Planned |
