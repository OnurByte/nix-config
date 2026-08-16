{ config, ... }:
let
  # Official Anthropic skills pinned to one immutable commit. Vesper keeps one
  # canonical skill tree and exposes agent-specific paths as links into it.
  anthropicSkills = builtins.fetchGit {
    url = "https://github.com/anthropics/skills.git";
    ref = "main";
    rev = "f6656c1256d5a8adfa37db9110046ef20bac644c";
  };

  skillSources = {
    frontend-design = "${anthropicSkills.outPath}/skills/frontend-design";
    webapp-testing = "${anthropicSkills.outPath}/skills/webapp-testing";
    web-artifacts-builder = "${anthropicSkills.outPath}/skills/web-artifacts-builder";
    mcp-builder = "${anthropicSkills.outPath}/skills/mcp-builder";
    skill-creator = "${anthropicSkills.outPath}/skills/skill-creator";
    pdf = "${anthropicSkills.outPath}/skills/pdf";
    docx = "${anthropicSkills.outPath}/skills/docx";
    xlsx = "${anthropicSkills.outPath}/skills/xlsx";
    pptx = "${anthropicSkills.outPath}/skills/pptx";

    vesper-maintainer = ./skills/vesper-maintainer;
    hermes-research-radar = ./skills/hermes-research-radar;
    vesper-obsidian-second-brain = ./skills/vesper-obsidian-second-brain;
  };

  skillNames = builtins.attrNames skillSources;
  canonicalRoot = ".agents/skills";
  agentRoots = [
    ".codex/skills"
    ".claude/skills"
    ".config/opencode/skills"
  ];

  # Hermes keeps its own bundled/agent-created skill tree. Expose only Vesper's
  # local workflow skills there, under a category directory, so Hermes retains
  # its native bundled skills while these shared definitions stay Nix-owned.
  hermesSkillNames = [
    "hermes-research-radar"
    "vesper-obsidian-second-brain"
    "vesper-maintainer"
  ];

  canonicalLinks = builtins.listToAttrs (
    map (skill: {
      name = "${canonicalRoot}/${skill}";
      value.source = skillSources.${skill};
    }) skillNames
  );

  agentLinks = builtins.listToAttrs (
    builtins.concatLists (
      map (
        root:
        map (skill: {
          name = "${root}/${skill}";
          value.source = config.lib.file.mkOutOfStoreSymlink "${config.home.homeDirectory}/${canonicalRoot}/${skill}";
        }) skillNames
      ) agentRoots
    )
  );

  hermesLinks = builtins.listToAttrs (
    map (skill: {
      name = ".hermes/skills/vesper/${skill}";
      value.source = config.lib.file.mkOutOfStoreSymlink "${config.home.homeDirectory}/${canonicalRoot}/${skill}";
    }) hermesSkillNames
  );
in
{
  home.file = canonicalLinks // agentLinks // hermesLinks // {
    # Hermes scheduled research writes durable output and proposed reusable
    # skills here. Drafts are intentionally separate from the active skill tree.
    ".local/share/vesper/briefings/.keep".text = "";
    ".local/share/vesper/skill-drafts/.keep".text = "";
  };
}
