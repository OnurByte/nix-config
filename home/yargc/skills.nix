{ ... }:
let
  # Keep every external skill source pinned to an immutable commit so agent
  # behavior cannot change underneath a Nix rebuild.
  fetchPinned =
    { url, rev }:
    builtins.fetchGit {
      inherit url rev;
      ref = "main";
    };

  sources = {
    anthropic = fetchPinned {
      url = "https://github.com/anthropics/skills.git";
      rev = "f6656c1256d5a8adfa37db9110046ef20bac644c";
    };
    shadcn = fetchPinned {
      url = "https://github.com/shadcn-ui/ui.git";
      rev = "d4fc45b1fbabfccb7a6a4333d8004cf19481caa9";
    };
    vercelSkills = fetchPinned {
      url = "https://github.com/vercel-labs/skills.git";
      rev = "c6f69c631292444cc541ac6d91e2226b0ff247da";
    };
    laravelBoost = fetchPinned {
      url = "https://github.com/laravel/boost.git";
      rev = "243045b97c4ea22f3838a35e79c496ed3df74cdb";
    };
    planetScaleSkills = fetchPinned {
      url = "https://github.com/planetscale/database-skills.git";
      rev = "af0ce0cfb65cca4cc21d18ca0d9cf270ca99d488";
    };
    phpSkills = fetchPinned {
      url = "https://github.com/AsyrafHussin/agent-skills.git";
      rev = "2631530e9d47c45f6d153ed9f245f073ccbbba30";
    };
    impeccable = fetchPinned {
      url = "https://github.com/pbakaus/impeccable.git";
      rev = "9ce0350054b0199bfd0ebbde95d9fd70c7c91741";
    };
  };

  skills = [
    # Anthropic
    {
      name = "frontend-design";
      source = "${sources.anthropic.outPath}/skills/frontend-design";
    }
    {
      name = "webapp-testing";
      source = "${sources.anthropic.outPath}/skills/webapp-testing";
    }
    {
      name = "web-artifacts-builder";
      source = "${sources.anthropic.outPath}/skills/web-artifacts-builder";
    }
    {
      name = "mcp-builder";
      source = "${sources.anthropic.outPath}/skills/mcp-builder";
    }
    {
      name = "skill-creator";
      source = "${sources.anthropic.outPath}/skills/skill-creator";
    }
    {
      name = "pdf";
      source = "${sources.anthropic.outPath}/skills/pdf";
    }
    {
      name = "docx";
      source = "${sources.anthropic.outPath}/skills/docx";
    }
    {
      name = "xlsx";
      source = "${sources.anthropic.outPath}/skills/xlsx";
    }
    {
      name = "pptx";
      source = "${sources.anthropic.outPath}/skills/pptx";
    }

    # Web / UI
    {
      name = "shadcn";
      source = "${sources.shadcn.outPath}/skills/shadcn";
    }
    {
      name = "tailwindcss-development";
      source = "${sources.laravelBoost.outPath}/.ai/tailwindcss/4/skill/tailwindcss-development";
    }
    {
      name = "impeccable";
      source = "${sources.impeccable.outPath}/.agent/skills/impeccable";
    }

    # Skill discovery
    {
      name = "find-skills";
      source = "${sources.vercelSkills.outPath}/skills/find-skills";
    }

    # Laravel / PHP / MySQL
    {
      name = "laravel-best-practices";
      source = "${sources.laravelBoost.outPath}/.ai/laravel/skill/laravel-best-practices";
    }
    {
      name = "php-best-practices";
      source = "${sources.phpSkills.outPath}/skills/php-best-practices";
    }
    {
      name = "mysql";
      source = "${sources.planetScaleSkills.outPath}/skills/mysql";
    }
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
          name = "${root}/${skill.name}";
          value = {
            source = skill.source;
          };
        }) skills
      ) roots
    )
  );
in
{
  home.file = links;
}
