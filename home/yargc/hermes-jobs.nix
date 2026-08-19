{
  unknown-frontier-github = {
    schedule = "30 8 * * *";
    mode = "dispatch";
    task = "unknown-frontier-github";
    deliver = "local";
    freshnessMinutes = 2160;
    description = "GitHub unknown-frontier scout";
  };

  unknown-frontier-reddit = {
    schedule = "35 8 * * *";
    mode = "dispatch";
    task = "unknown-frontier-reddit";
    deliver = "local";
    freshnessMinutes = 2160;
    description = "Reddit unknown-frontier scout";
  };

  unknown-frontier-x = {
    schedule = "40 8 * * *";
    mode = "dispatch";
    task = "unknown-frontier-x";
    deliver = "local";
    freshnessMinutes = 2160;
    description = "X with direct/mirror fallback frontier scout";
  };

  unknown-frontier-web = {
    schedule = "45 8 * * *";
    mode = "dispatch";
    task = "unknown-frontier-web";
    deliver = "local";
    freshnessMinutes = 2160;
    description = "protected clearnet and Tor onion frontier scout";
  };

  free-ai-radar = {
    schedule = "50 8 * * *";
    mode = "dispatch";
    task = "free-ai-radar";
    deliver = "local";
    freshnessMinutes = 2160;
    description = "linux.do-first legitimate free AI and cost-saving radar";
  };

  unknown-frontier-synthesis = {
    schedule = "10 9 * * *";
    mode = "dispatch";
    task = "unknown-frontier-synthesis";
    deliver = "local";
    freshnessMinutes = 2160;
    description = "bounded fan-in of fresh GitHub Reddit X and web/onion scout state";
  };

  agenda = {
    schedule = "30 9 * * *";
    mode = "dispatch";
    task = "agenda";
    deliver = "local";
    freshnessMinutes = 2160;
    description = "compact current agenda before Morning Check";
  };

  morning-check = {
    schedule = "0 10 * * *";
    mode = "dispatch";
    task = "morning-check";
    deliver = "local";
    freshnessMinutes = 2160;
    description = "daily Telegram brief built from local project data and persistent Hermes findings";
  };

  upstream-edge-radar = {
    schedule = "0 15 * * *";
    mode = "dispatch";
    task = "upstream-edge-radar";
    deliver = "local";
    freshnessMinutes = 2160;
    description = "early warning for Vesper upstream changes";
  };

  communications-radar = {
    schedule = "4,19,34,49 * * * *";
    mode = "dispatch";
    task = "communications-radar";
    deliver = "local";
    freshnessMinutes = 90;
    description = "read-only Agent Messenger communications triage with high-signal local alerts";
  };

  vesper-health-watch = {
    schedule = "7,22,37,52 * * * *";
    mode = "watchdog";
    task = "vesper-health-watch";
    deliver = "telegram";
    description = "zero-token workstation health, Hermes freshness and external dead-man heartbeat";
  };

  cron-skill-integrity-watch = {
    schedule = "17 */6 * * *";
    mode = "watchdog";
    task = "cron-skill-integrity-watch";
    deliver = "telegram";
    description = "zero-token scheduler registry script and skill integrity check";
  };

  second-brain-dream = {
    schedule = "30 23 * * *";
    mode = "dispatch";
    task = "second-brain-dream";
    deliver = "local";
    freshnessMinutes = 2160;
    description = "nightly durable research and communications consolidation into the Obsidian second brain";
  };

  user-pain-miner = {
    schedule = "0 11 * * 0";
    mode = "dispatch";
    task = "user-pain-miner";
    deliver = "local";
    freshnessMinutes = 11520;
    description = "weekly recurring-problem and project-opportunity miner";
  };

  project-archaeologist = {
    schedule = "30 12 * * 0";
    mode = "dispatch";
    task = "project-archaeologist";
    deliver = "local";
    freshnessMinutes = 11520;
    description = "weekly local repository archaeology for forgotten unfinished work";
  };

  skill-evolution-review = {
    schedule = "0 14 * * 0";
    mode = "dispatch";
    task = "skill-evolution-review";
    deliver = "local";
    freshnessMinutes = 11520;
    description = "weekly evidence-based review of skill drafts and research heuristics";
  };

  ai-usage-economist = {
    schedule = "30 15 * * 0";
    mode = "dispatch";
    task = "ai-usage-economist";
    deliver = "local";
    freshnessMinutes = 11520;
    description = "weekly local agent usage and model-cost workflow review";
  };
}
