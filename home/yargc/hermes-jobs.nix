{
  unknown-frontier-github = {
    schedule = "30 8 * * *";
    mode = "dispatch";
    task = "unknown-frontier-github";
    deliver = "local";
    description = "GitHub unknown-frontier scout with broad low-attention candidate collection";
  };

  unknown-frontier-reddit = {
    schedule = "35 8 * * *";
    mode = "dispatch";
    task = "unknown-frontier-reddit";
    deliver = "local";
    description = "Reddit unknown-frontier scout with broad recent and niche candidate collection";
  };

  unknown-frontier-x = {
    schedule = "40 8 * * *";
    mode = "dispatch";
    task = "unknown-frontier-x";
    deliver = "local";
    description = "X unknown-frontier scout using native x_search";
  };

  free-ai-radar = {
    schedule = "45 8 * * *";
    mode = "dispatch";
    task = "free-ai-radar";
    deliver = "local";
    description = "Linux.do-first legitimate free AI and cost-saving radar";
  };

  unknown-frontier-synthesis = {
    schedule = "0 9 * * *";
    mode = "dispatch";
    task = "unknown-frontier-synthesis";
    deliver = "local";
    description = "bounded fan-in of fresh GitHub Reddit and X scout state";
  };

  agenda = {
    schedule = "30 9 * * *";
    mode = "dispatch";
    task = "agenda";
    deliver = "local";
    description = "compact current agenda before Morning Check";
  };

  morning-check = {
    schedule = "0 10 * * *";
    mode = "dispatch";
    task = "morning-check";
    deliver = "local";
    description = "daily Telegram brief built from local project data and persistent Hermes findings";
  };

  upstream-edge-radar = {
    schedule = "0 15 * * *";
    mode = "dispatch";
    task = "upstream-edge-radar";
    deliver = "local";
    description = "zero-model upstream head gate followed by research only when tracked sources move";
  };

  vesper-health-watch = {
    schedule = "7 */3 * * *";
    mode = "watchdog";
    task = "vesper-health-watch";
    deliver = "telegram";
    description = "zero-token workstation health alert that stays silent when healthy";
  };

  cron-skill-integrity-watch = {
    schedule = "17 */6 * * *";
    mode = "watchdog";
    task = "cron-skill-integrity-watch";
    deliver = "telegram";
    description = "zero-token scheduler registry script and skill integrity check";
  };

  cron-retention = {
    schedule = "15 3 * * 1";
    mode = "dispatch";
    task = "cron-retention";
    deliver = "local";
    description = "deterministic cleanup of ephemeral cron sessions outputs candidate pools and old run records";
  };

  second-brain-dream = {
    schedule = "30 23 * * *";
    mode = "dispatch";
    task = "second-brain-dream";
    deliver = "local";
    description = "nightly durable research consolidation into the Obsidian second brain";
  };

  user-pain-miner = {
    schedule = "0 11 * * 0";
    mode = "dispatch";
    task = "user-pain-miner";
    deliver = "local";
    description = "weekly recurring-problem and project-opportunity miner";
  };

  project-archaeologist = {
    schedule = "30 12 * * 0";
    mode = "dispatch";
    task = "project-archaeologist";
    deliver = "local";
    description = "weekly local repository archaeology for forgotten unfinished work";
  };

  skill-evolution-review = {
    schedule = "0 14 * * 0";
    mode = "dispatch";
    task = "skill-evolution-review";
    deliver = "local";
    description = "weekly evidence-based review of skill drafts and research heuristics";
  };

  ai-usage-economist = {
    schedule = "30 15 * * 0";
    mode = "dispatch";
    task = "ai-usage-economist";
    deliver = "local";
    description = "weekly local agent usage and model-cost workflow review";
  };

  weekly-intelligence-review = {
    schedule = "0 17 * * 0";
    mode = "dispatch";
    task = "weekly-intelligence-review";
    deliver = "local";
    description = "weekly decision-oriented synthesis across research projects upstream cost and skill learning";
  };
}
