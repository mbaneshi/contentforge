# Codeilus — Content Kit

## Project Summary
Turn any codebase into an interactive, gamified learning experience with 3D visualizations.

## Key Facts
- **Repo**: https://github.com/mbaneshi/codeilus
- **Docs**: https://mbaneshi.github.io/codeilus/
- **Version**: v0.1.0
- **Stack**: Rust (16 crates) + SvelteKit 5 + Three.js + SQLite
- **Install**: `brew tap mbaneshi/codeilus && brew install codeilus`
- **Run**: `codeilus ./any-repo`
- **Tests**: 81 passing, zero clippy warnings
- **Binary**: Single 40MB binary, cross-platform (macOS Intel/ARM, Linux x86/ARM)
- **License**: MIT

## Unique Selling Points (use in content)
1. **Only tool** that turns a codebase into a gamified learning experience
2. **3D force-directed graphs** of files, symbols, and dependencies
3. **AI-generated learning chapters** that explain how code works
4. **Single binary** — no Docker, no cloud, no config. Just `codeilus ./repo`
5. **16 Rust crates** — clean architecture, not a monolith
6. **MCP server mode** — integrates with Claude Code

## Content Hooks (angles that get engagement)
- "I turned [famous repo]'s source code into a 3D learning game"
- "Onboarding takes weeks. This tool makes it 30 seconds."
- "Your codebase has hidden complexity clusters. Here's how to find them."
- "I shipped a full SvelteKit app inside a Rust binary"
- "16 Rust crates, 81 tests, one binary — anatomy of a real Rust workspace"

## Screenshots
- `assets/screenshot-home.png` (copy from /Users/bm/codeilus/codeilus/docs/assets/)
- `assets/screenshot-explore.png` (copy from /Users/bm/codeilus/codeilus/docs/assets/)

## Launch Status
- [ ] README polished with GIFs
- [ ] Run against 3 famous repos for results data
- [ ] Draft launch post (Substack)
- [ ] Draft X thread
- [ ] Draft LinkedIn post
- [ ] Publish to Hacker News
- [ ] Publish to Reddit (r/rust, r/programming)
- [ ] Post on DEV.to
- [ ] Record 60-second demo video for YouTube

## Content Pipeline
| # | Type | Title Idea | Status |
|---|------|-----------|--------|
| 1 | results-post | "I analyzed 3 trending GitHub repos with Codeilus. Here's what the graphs reveal." | Draft |
| 2 | launch-post | "I built a tool that turns any codebase into a 3D learning game." | Planned |
| 3 | build-story | "16 Rust crates, SvelteKit 5, Three.js — shipped as a single binary." | Planned |
