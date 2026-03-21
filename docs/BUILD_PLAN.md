# ContentForge — Prioritized Build Plan

> Based on Perplexity research (2026-03-21) and Windmill V2 architecture
> Total: ~2,100 LOC across 7 phases, targeting first revenue at week 6

## Phase Order (by business impact)

| Phase | What | Week | LOC | Why First |
|-------|------|------|-----|-----------|
| **1.1** | Wire MCP tools to real DB | 1 | ~150 | MCP = primary differentiator and discovery channel |
| **1.2** | Wire scheduler to real publish | 1 | ~200 | Completes basic end-to-end loop |
| **2** | Mastodon + Bluesky adapters | 2 | ~345 | Free/stable platforms devs use. 3 platforms = credible |
| **3** | Pipeline engine (Pro feature) | 3-4 | ~760 | Core paid feature. Enables hero workflow |
| **4** | MCP registry + Homebrew | 4-5 | ~100 | Free distribution to target audience |
| **5** | Encryption + license + Stripe | 5-6 | ~450 | First revenue |
| **6** | Cron-triggered pipelines | 6-7 | ~150 | Sticky Pro feature |
| **7** | Remove dead platforms | 7 | -50 | Reduce maintenance |

## Hero Workflow (what we're building toward)

```
git commit → Claude MCP calls contentforge.draft() →
contentforge.adapt(devto, mastodon, bluesky) →
contentforge.schedule(friday_8am) →
[Friday] auto-publish to all 3 platforms
```

## v1 Platform Priority (research-informed)

- ✅ DEV.to (done, working)
- 🔨 Mastodon (free API, 1.5M MAU, FOSS/dev community)
- 🔨 Bluesky (free API, growing indie/dev audience)
- ⏳ YouTube (quota-limited, Phase 3+)
- ⏳ Twitter/X (user brings own keys, $100+/mo, Phase 3+)
- ⏳ LinkedIn (partner gate required, Phase 3+)
- ❌ Medium (API deprecated in 2026)
- ❌ Substack (no API exists)
