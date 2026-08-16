{ ... }:
let
  # Official Anthropic skills pinned to one immutable commit. Home Manager exposes
  # the same source tree to each coding agent instead of maintaining copies.
  anthropicSkills = builtins.fetchGit {
    url = "https://github.com/anthropics/skills.git";
    ref = "main";
    rev = "f6656c1256d5a8adfa37db9110046ef20bac644c";
  };

  skills = [
    "frontend-design"
    "webapp-testing"
    "web-artifacts-builder"
    "mcp-builder"
    "skill-creator"
    "pdf"
    "docx"
    "xlsx"
    "pptx"
  ];

  roots = [
    ".codex/skills"
    ".claude/skills"
    ".config/opencode/skills"
  ];

  links = builtins.listToAttrs (
    builtins.concatLists (
      map (
        root:
        map (skill: {
          name = "${root}/${skill}";
          value = {
            source = "${anthropicSkills.outPath}/skills/${skill}";
          };
        }) skills
      ) roots
    )
  );
in
{
  home.file = links;
}
