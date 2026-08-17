pub const FRONTIER_TASKS: &[&str] = &[
    "unknown-frontier-github",
    "unknown-frontier-reddit",
    "unknown-frontier-x",
    "unknown-frontier-web",
];

pub const ALL_TASKS: &[&str] = &[
    "unknown-frontier-github",
    "unknown-frontier-reddit",
    "unknown-frontier-x",
    "unknown-frontier-web",
    "unknown-frontier-synthesis",
    "frontier-daily",
    "free-ai-radar",
    "agenda",
    "morning-check",
    "upstream-edge-radar",
    "vesper-health-watch",
    "cron-skill-integrity-watch",
    "second-brain-dream",
    "user-pain-miner",
    "project-archaeologist",
    "skill-evolution-review",
    "ai-usage-economist",
];

pub fn objective(task: &str) -> &'static str {
    match task {
        "unknown-frontier-github" => "Scout GitHub for overlooked coding-agent workflows, harnesses, MCP/skills/context-engineering techniques, and Monero/privacy engineering. Expand through repositories, issues, PRs, commits, forks, authors and organizations. Prefer working code and primary technical evidence over stars or hype.",
        "unknown-frontier-reddit" => "Scout Reddit for high-signal coding-agent and Monero/privacy techniques. Inspect niche communities and comment branches, treat central subreddits only as seeds, and verify important community claims against repositories, docs, issues, PRs, papers or other primary evidence.",
        "unknown-frontier-x" => "Scout X/Twitter for low-attention builders and researchers around coding agents and privacy. Inspect replies, quotes, demos and linked primary artifacts. Mirrors are transport, never independent corroboration.",
        "unknown-frontier-web" => "Scout clearnet and Tor/onion surfaces for coding-agent, Monero, privacy and OPSEC techniques. Use the machine Tor client for onion fetches when needed. Treat onion/community material as discovery or operational evidence and explicitly report access failures.",
        "unknown-frontier-synthesis" => "Synthesize the fresh GitHub, Reddit, X and web/onion scout reports. Deduplicate familiar items, counter-review strong claims, follow the best candidates to primary evidence, and rank agentic software-engineering plus Monero/privacy findings first.",
        "free-ai-radar" => "Find legitimate currently useful ways to reduce coding-agent and developer-workflow cost. Treat linux.do as a first-class discovery surface, then verify through official docs, repositories, releases or other primary sources. Reject stolen/shared credentials, mass-account abuse, payment bypasses and service-restriction evasion.",
        "agenda" => "Produce a compact current agenda biased toward coding agents/vibe coding/dev tooling and Monero/privacy. Secondary topics are Nix/Linux, Tor/onion, OPSEC, security, private communications, open source and consequential technology changes. Avoid generic benchmark chatter and filler.",
        "morning-check" => "Produce the Telegram-ready Morning Check from local project state and recent durable Hermes research. Sections: Git/Projects, Todos, important News, and only useful Actions. Prefer already verified findings and do not rediscover the same stories unnecessarily.",
        "upstream-edge-radar" => "Inspect meaningful recent upstream changes around Hermes, Codex, Claude Code, OpenCode, nixpkgs/NixOS, Hyprland, Caelestia, Zen, Helium, Tor, Monero and Cuprate. Surface breaking changes, new capabilities, deprecations, security/privacy implications and workarounds before they become surprises.",
        "second-brain-dream" => "Use the Vesper Obsidian second-brain workflow to consolidate durable recent research. Promote only durable facts, relationships, corrections, open questions and proven source paths. Stage reusable procedures as drafts and never auto-promote active Nix-owned skills.",
        "user-pain-miner" => "Mine recurring evidence-backed user pain across Hermes, Codex, Claude Code, OpenCode, NixOS, Hyprland and adjacent tooling. Require recurrence evidence before proposing a project or workflow opportunity.",
        "project-archaeologist" => "Inspect the user's bounded local Git roots and find forgotten but valuable unfinished work: stale dirty repositories, meaningful branches, abandoned experiments and concrete blockers. Prioritize a small set actually worth revisiting.",
        "skill-evolution-review" => "Review the active Vesper research skill, representative evals, accumulated source evidence, heuristics and skill drafts. Decide promote, keep-testing, merge, narrow, retire or rollback using evidence rather than persuasive wording. Do not mutate the active skill automatically.",
        "ai-usage-economist" => "Inspect local agent usage/accounting surfaces such as ccusage, CodexBar and TurnLens. Separate measured facts from recommendations, identify expensive low-value routing, and call out missing/unreliable measurements without inventing costs.",
        _ => "Run the declared Vesper Hermes task faithfully and preserve durable state.",
    }
}

pub fn research_contract(task: &str, skill: &str, durable: &str, extra: &str) -> String {
    format!(
        "Run Vesper's persistent Hermes workflow for task `{task}`.\n\nObjective:\n{}\n\nInstalled research contract:\n{skill}\n\nDurable state/context:\n{durable}\n\nTask-specific context:\n{extra}\n\nRules:\n- user-supplied and central sources are seeds, never an allowlist\n- preserve exploration outside known sources\n- prefer primary evidence for important claims\n- low attention is a discovery hint, not a quality score\n- do not fabricate URLs, page contents or numeric coverage\n- explicitly report blocked surfaces and real coverage shortfall\n- generic local-model/inference hobby content is out of scope unless it materially improves coding-agent quality/cost/privacy/deployment\n\nReturn exactly one JSON object and nothing else with this shape:\n{{\"title\":\"short title\",\"summary\":\"1-3 sentence summary\",\"body\":\"concise useful report\",\"priority\":\"low|normal|high|critical\",\"confidence\":0.0,\"sources\":[{{\"title\":\"source\",\"url\":\"https://...\"}}],\"coverage\":{{\"candidateTarget\":0,\"candidatesInspected\":0,\"canonicalCandidates\":0,\"deepReads\":0,\"primaryVerifications\":0,\"surfaces\":[],\"limitations\":[]}},\"statePatch\":{{\"knownConcepts\":[],\"candidateSources\":[],\"heuristics\":[],\"openQuestions\":[]}}}}",
        objective(task)
    )
}

pub fn adhoc_contract(query: &str, pages: usize, deep_reads: usize, skill: &str, sources: &str) -> String {
    format!(
        "Run Vesper's installed `hermes-research-radar` skill for this ad-hoc research request:\n\n{query}\n\nCandidate inspection target: {pages}\nDeep-read target: {deep_reads}\n\nResearch contract:\n{skill}\n\nCurrent adaptive source state:\n{sources}\n\nSearch broadly across GitHub, Reddit, X and relevant clearnet/onion surfaces. Existing sources are seeds, not an allowlist. Canonicalize duplicates conceptually, spend deep-reading effort on the strongest candidates, verify important claims against primary evidence, and report actual coverage instead of pretending targets were reached. Return exactly one JSON object with title, summary, body, priority, confidence, sources, coverage and statePatch."
    )
}
