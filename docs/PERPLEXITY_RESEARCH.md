# Perplexity Research Prompts for ContentForge

Use these prompts with Perplexity to validate assumptions and find opportunities.

---

## Prompt 1: Market Validation

```
I'm building an open-source, Rust-based CLI tool for developers who create content across multiple platforms (DEV.to, Twitter/X, LinkedIn, Medium, Substack, YouTube). It's a single binary with CLI + TUI + Web UI + MCP server for AI assistant integration.

Competitors: Buffer ($6-120/mo SaaS), Postiz (27k GitHub stars, Node.js, self-hosted), Mixpost (PHP/Laravel), Typefully (SaaS, X-only).

My differentiators: single binary (brew install), Rust performance, CLI-native, MCP protocol for Claude/AI integration, SQLite (zero infrastructure), MIT license.

I need to understand:

1. How big is the market of developers who actively create content across multiple platforms? Any data on how many devs post regularly on DEV.to, LinkedIn, Twitter, Medium?
2. What are the actual pain points developers face with cross-platform content publishing? Are there surveys, blog posts, or forum threads about this?
3. What pricing do developer tools in this space actually achieve? Show me real examples of CLI tools or developer productivity tools that charge $49 one-time or $9/mo and their reported revenue.
4. Has anyone tried a "developer-native content tool" before? What happened — did it succeed or fail, and why?
5. What's the current state of the "build in public" movement — how many developers are actively doing it, and what tools do they use?
```

---

## Prompt 2: Competitive Deep-Dive

```
Do a deep competitive analysis of these open-source social media management tools:

1. Postiz (https://github.com/gitroomhq/postiz-app) — 27k stars
2. Mixpost (https://github.com/inovector/mixpost) — 3k stars
3. Shoutify (https://github.com/TechSquidTV/Shoutify)
4. Typefully (typefully.com — closed source)

For each, tell me:
- Monthly active users or install numbers if available
- Revenue (if public — Postiz and Mixpost have paid tiers)
- What features users complain about missing (check GitHub issues, Reddit threads, HN comments)
- What their users love most
- How they handle platform API authentication (especially Twitter/X v2 API changes)
- Their deployment complexity (how hard is self-hosting?)
- Community activity (contributors, issue response time, release frequency)

Also: Are there any Rust-based content/publishing tools on GitHub? Any TUI-based social media tools? What's the landscape look like for CLI-first developer tools in content/social?
```

---

## Prompt 3: Pricing & Monetization Research

```
I need pricing research for developer CLI tools with open-source free tiers and paid upgrades:

1. Show me 10 examples of developer tools that use the "open core" model — free CLI tool + paid features. For each: what's free, what's paid, what's the price, and what's their estimated revenue?

2. Specifically for these successful open-core dev tools, what features do they gate behind payment?
   - Charm.sh (terminal tools)
   - Fig/Amazon Q (terminal autocomplete)
   - Warp (terminal)
   - Railway (deployment)
   - Supabase (database)
   - Windmill (workflow automation)

3. For a single-binary CLI tool targeting solo developers: is one-time payment ($49) or subscription ($9/mo) more effective? What does the data say about developer willingness to pay for CLI tools?

4. What distribution channels work best for developer tools? Rank these by effectiveness: Hacker News, Product Hunt, Reddit, DEV.to, Twitter/X, YouTube, crates.io, Homebrew, GitHub trending.

5. What's the typical GitHub stars → paid conversion funnel for open-source developer tools? Is 3-5% realistic?
```

---

## Prompt 4: Platform API Landscape (2026)

```
What is the current state (as of March 2026) of content publishing APIs for these platforms:

1. Twitter/X API v2 — current pricing tiers, rate limits for posting, any recent changes
2. LinkedIn API — what's required for programmatic posting? Any changes to their app review process?
3. DEV.to / Forem API — is it still stable and free?
4. Medium API — is it still deprecated? Any alternatives for programmatic posting?
5. YouTube Data API v3 — current quota limits for video uploads, any changes
6. Instagram Content Publishing API — current requirements for business verification
7. Substack — any official API yet? What's the state of unofficial approaches?
8. Bluesky AT Protocol — posting API, is it worth supporting?
9. Mastodon API — posting API, adoption numbers
10. Threads API — does Meta have a public posting API for Threads?

For each: authentication method, rate limits, cost, and any gotchas for a tool that publishes on behalf of users.
```

---

## Prompt 5: MCP Protocol Adoption & AI-Native Tools

```
The Model Context Protocol (MCP) by Anthropic is a standard for connecting AI assistants to external tools. I'm building a content publishing tool with an embedded MCP server so that Claude Code, Claude Desktop, and other AI assistants can draft/adapt/publish content through my tool.

I need to understand:

1. How many tools/servers currently support MCP? What's the adoption curve looking like in early 2026?
2. Are there any content creation or social media tools that already have MCP integration?
3. What's the developer sentiment around MCP vs alternatives (function calling, OpenAI plugins, LangChain tools)?
4. Is "MCP-native" a meaningful differentiator for a developer tool in 2026? Would developers choose a tool because it has MCP support?
5. What other AI assistant protocols should I consider supporting besides MCP? (A2A by Google, OpenAI function calling, etc.)
6. Are there examples of tools that gained traction specifically because of their AI assistant integration?
```

---

## Prompt 6: Launch Strategy for Rust Developer Tools

```
I'm launching an open-source Rust developer tool (content publishing CLI). I want to learn from successful Rust tool launches:

1. How did these Rust tools launch and get their first 1000 GitHub stars?
   - ripgrep (rg)
   - bat
   - fd
   - starship
   - zoxide
   - nushell
   - helix editor
   - ratatui
   - Zed editor

2. What's the typical timeline from first commit to 1000 stars for Rust CLI tools?

3. Which communities are most receptive to Rust tools? (r/rust, Hacker News, Rust newsletter, This Week in Rust, etc.)

4. What makes a Rust tool launch post go viral? Are there common patterns in successful Show HN posts for Rust projects?

5. Should I publish to crates.io early (for discovery) or wait until the tool is polished?

6. How important is a documentation site (GitHub Pages) for gaining stars and trust?

7. Any examples of Rust tools that successfully monetized? What was their path?
```

---

## How to Use These Results

After running each prompt through Perplexity:

1. Save key findings to `/Users/bm/contentforge/docs/research/` (create per-topic files)
2. Update PRODUCT_STRATEGY.md with validated/invalidated assumptions
3. Adjust pricing if research suggests different model
4. Identify the #1 launch channel based on research
5. Find the top 5 communities to engage with pre-launch
