{ ... }:
let
  fetchPinned =
    {
      url,
      rev,
    }:
    builtins.fetchGit {
      inherit url rev;
      ref = "main";
    };

  anthropicSkills = fetchPinned {
    url = "https://github.com/anthropics/skills.git";
    rev = "f6656c1256d5a8adfa37db9110046ef20bac644c";
  };

  vercelSkills = fetchPinned {
    url = "https://github.com/vercel-labs/skills.git";
    rev = "c6f69c631292444cc541ac6d91e2226b0ff247da";
  };

  shadcnSkills = fetchPinned {
    url = "https://github.com/shadcn-ui/ui.git";
    rev = "d4fc45b1fbabfccb7a6a4333d8004cf19481caa9";
  };

  laravelBoost = fetchPinned {
    url = "https://github.com/laravel/boost.git";
    rev = "243045b97c4ea22f3838a35e79c496ed3df74cdb";
  };

  mindrallySkills = fetchPinned {
    url = "https://github.com/Mindrally/skills.git";
    rev = "05a71308897983093248d719a2ffa1bca61d0768";
  };

  impeccableSkills = fetchPinned {
    url = "https://github.com/pbakaus/impeccable.git";
    rev = "9ce0350054b0199bfd0ebbde95d9fd70c7c91741";
  };

  skillSources = {
    "frontend-design" = "${anthropicSkills.outPath}/skills/frontend-design";
    "webapp-testing" = "${anthropicSkills.outPath}/skills/webapp-testing";
    "web-artifacts-builder" = "${anthropicSkills.outPath}/skills/web-artifacts-builder";
    "mcp-builder" = "${anthropicSkills.outPath}/skills/mcp-builder";
    "skill-creator" = "${anthropicSkills.outPath}/skills/skill-creator";
    pdf = "${anthropicSkills.outPath}/skills/pdf";
    docx = "${anthropicSkills.outPath}/skills/docx";
    xlsx = "${anthropicSkills.outPath}/skills/xlsx";
    pptx = "${anthropicSkills.outPath}/skills/pptx";

    "find-skills" = "${vercelSkills.outPath}/skills/find-skills";
    shadcn = "${shadcnSkills.outPath}/skills/shadcn";
    impeccable = "${impeccableSkills.outPath}/.agents/skills/impeccable";

    "laravel-best-practices" = "${laravelBoost.outPath}/.ai/laravel/skill/laravel-best-practices";
    "tailwindcss-development" = "${laravelBoost.outPath}/.ai/tailwindcss/4/skill/tailwindcss-development";

    "php-development" = "${mindrallySkills.outPath}/php-development";
    "mysql-best-practices" = "${mindrallySkills.outPath}/mysql-best-practices";
  };

  roots = [
    ".agents/skills"
    ".codex/skills"
    ".claude/skills"
    ".config/opencode/skills"
  ];

  skillNames = builtins.attrNames skillSources;

  links = builtins.listToAttrs (
    builtins.concatLists (
      map (
        root:
        map (skill: {
          name = "${root}/${skill}";
          value = {
            source = skillSources.${skill};
          };
        }) skillNames
      ) roots
    )
  );
in
{
  home.file = links;
}
