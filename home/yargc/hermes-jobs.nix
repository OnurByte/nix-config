{
  frontier-daily = {
    schedule = "30 8 * * *";
    mode = "dispatch";
    task = "frontier-daily";
    deliver = "telegram";
    description = "parallel GitHub Reddit and X frontier scouts followed by one verified synthesis";
  };

  free-ai-radar = {
    schedule = "45 8 * * *";
    mode = "dispatch";
    task = "free-ai-radar";
    deliver = "telegram";
    description = "linux.do-first legitimate free AI and cost-saving radar";
  };

  agenda = {
    schedule = "30 9 * * *";
    mode = "dispatch";
    task = "agenda";
    deliver = "telegram";
    description = "compact current agenda before Morning Check";
  };

  morning-check = {
    schedule = "0 10 * * *";
    mode = "dispatch";
    task = "morning-check";
    deliver = "telegram";
    description = "daily Telegram brief built from local project data and persistent Hermes findings";
  };

  upstream-edge-radar = {
    schedule = "0 15 * * *";
    mode = "dispatch";
    task = "upstream-edge-radar";
    deliver = "telegram";
    description = "early warning for Vesper upstream changes";
  };

  vesper-health-watch = {
    schedule = "7 */3 * * *";
    mode = "watchdog";
    task = "vesper-health-watch";
    deliver = "telegram";
    description = "zero-token workstation health alert that stays silent when healthy";
  };

  cron-integrity-watch = {
    schedule = "17 */6 * * *";
    mode = "watchdog";
    task = "cron-integrity-watch";
    deliver = "telegram";
    description = "zero-token scheduler registry script and skill integrity check";
  };

  second-brain-dream = {
    schedule = "30 23 * * *";
    mode = "dispatch";
    task = "second-brain-dream";
    deliver = "telegram";
    description = "nightly durable research consolidation into the Obsidian second brain";
  };

  user-pain-miner = {
    schedule = "0 11 * * 0";
    mode = "dispatch";
    task = "user-pain-miner";
    deliver = "telegram";
    description = "weekly recurring-problem and project-opportunity miner";
  };

  project-archaeologist = {
    schedule = "30 12 * * 0";
    mode = "dispatch";
    task = "project-archaeologist";
    deliver = "telegram";
    description = "weekly local repository archaeology for forgotten unfinished work";
  };

  skill-evolution-review = {
    schedule = "0 14 * * 0";
    mode = "dispatch";
    task = "skill-evolution-review";
    deliver = "telegram";
    description = "weekly evidence-based review of skill drafts and research heuristics";
  };

  ai-usage-economist = {
    schedule = "30 15 * * 0";
    mode = "dispatch";
    task = "ai-usage-economist";
    deliver = "telegram";
    description = "weekly local agent usage and model-cost workflow review";
  };
}
