{
  config,
  lib,
  pkgs,
  ...
}:
let
  home = config.home.homeDirectory;

  morningCheckDeliver = pkgs.writeText "vesper-morning-check-deliver.sh" ''
    #!/usr/bin/env bash
    # Legacy Morning Check end-to-end runner for Hermes no_agent cron mode.
    # The automation fleet migrates the job to native agent mode; this remains
    # as a compatibility fallback until local cron state is reconciled.
    set -euo pipefail

    export PATH="${home}/.local/bin:${home}/.hermes/hermes-agent/venv/bin:$PATH"
    export GIT_TERMINAL_PROMPT=0
    export HERMES_YOLO_MODE=1

    CANONICAL_COLLECT="${home}/.hermes/scripts/morning-check-collect.sh"
    LEGACY_COLLECT="${home}/.hermes/scripts/sabah-check-collect.sh"
    HERMES_BIN="$(command -v hermes)"
    TMPDIR="''${TMPDIR:-/tmp}"
    DATA_FILE="$(mktemp "$TMPDIR/morning-check-data.XXXXXX")"
    PROMPT_FILE="$(mktemp "$TMPDIR/morning-check-prompt.XXXXXX")"
    OUT_FILE="$(mktemp "$TMPDIR/morning-check-out.XXXXXX")"
    FULL_PROMPT_FILE="''${PROMPT_FILE}.full"
    COLLECT_ERR="$TMPDIR/morning-check-collect.err"
    MODEL_ERR="$TMPDIR/morning-check-oneshot.err"
    trap 'rm -f "$DATA_FILE" "$PROMPT_FILE" "$OUT_FILE" "$FULL_PROMPT_FILE"' EXIT

    if [[ -x "$CANONICAL_COLLECT" || -f "$CANONICAL_COLLECT" ]]; then
      COLLECT="$CANONICAL_COLLECT"
    elif [[ -x "$LEGACY_COLLECT" || -f "$LEGACY_COLLECT" ]]; then
      COLLECT="$LEGACY_COLLECT"
    else
      printf '%s\n' "Morning Check collector not found." >&2
      exit 1
    fi

    if ! timeout 50 bash "$COLLECT" >"$DATA_FILE" 2>"$COLLECT_ERR"; then
      printf '%s\n' "Warning: data collection was partial or failed; continue with available data." >>"$DATA_FILE"
      tail -20 "$COLLECT_ERR" >>"$DATA_FILE" 2>/dev/null || true
    fi

    cat >"$PROMPT_FILE" <<'PROMPT'
    Morning Check — concise Telegram briefing.

    The DATA section below contains collected input. Use it as the primary source.
    If additional verification is genuinely useful, use at most 2 web_search/x_search calls.
    Otherwise, finish using the provided DATA.

    ## Final response

    Return only the final Telegram message.
    Do not include analysis, tool chatter, status messages, preambles, or filler.
    Write in English with a concise, neutral, information-dense tone.

    Sections:

    1) **Git / Projects**
    2) **Todos**
    3) **News**
    4) Optional: **Actions**

    ### Git / Projects

    For each relevant repository, use 1–3 lines maximum.
    Include only useful state such as clean/dirty status, important untracked files, meaningful recent changes, relevant PRs/issues, and blockers.
    Do not dump raw Git output.

    ### Todos

    Include only important items, maximum 3–5.
    Prioritize blockers, today/this week work, and unfinished work with loss or regression risk.
    If there is nothing important, write: `No important open todos.`

    ### News

    Include at least 10 items. Use 12–15 when there are enough genuinely important developments.

    Preferred topics:
    - privacy
    - payments
    - Monero / Zcash, excluding investment or price discussion
    - Tor / onion ecosystem
    - AI / coding agents / developer tooling
    - security / development
    - web and privacy technology
    - startups / business
    - major technology developments

    Allow at most one unusual or highly interesting off-topic item.
    Avoid price analysis, technical analysis, coupons, Polymarket, filler, and repeated stories.

    Format each item exactly like:

    **N. Title**
    One concise sentence explaining what happened.
    URL

    Do not add a "Why:" section. Never invent information or URLs.

    ### Actions

    Optional. Use only when there are 1–3 concrete actions worth taking based on the report.

    Forbidden content:
    - cron status
    - HEARTBEAT
    - internal execution details
    - model/tool commentary
    - filler
    PROMPT

    {
      cat "$PROMPT_FILE"
      echo
      echo '----- DATA -----'
      head -c 90000 "$DATA_FILE"
      echo
      echo '----- END DATA -----'
    } >"$FULL_PROMPT_FILE"

    set +e
    timeout 240 "$HERMES_BIN" -z "$(cat "$FULL_PROMPT_FILE")" \
      --provider xai-oauth \
      -m grok-4.5 \
      -t web \
      --yolo \
      >"$OUT_FILE" 2>"$MODEL_ERR"
    rc=$?
    set -e

    if [[ $rc -ne 0 || ! -s "$OUT_FILE" ]]; then
      {
        echo "Morning Check — fallback mode (model run failed, rc=$rc)"
        echo
        echo "**Git / Projects**"
        rg -n '^### |dirty_files:|^## feat|^## main|recent:' "$DATA_FILE" | head -40
        echo
        echo "**Todos**"
        echo "- Automatic summary failed; inspect PROGRESS or memory state."
        echo
        echo "**News**"
        rg -n '^\*\*|### [0-9]|https://' "$DATA_FILE" | head -40
        echo
        echo "Log: $MODEL_ERR"
      } >"$OUT_FILE"
    fi

    OUT_FILE="$OUT_FILE" python3 - <<'PY'
    import os
    import re
    import sys
    from pathlib import Path

    text = Path(os.environ["OUT_FILE"]).read_text(errors="replace").strip()
    markers = ["Morning Check", "**1)", "**Git", "1) **Git", "**Git / Projects**"]
    cut = -1
    for marker in markers:
        index = text.find(marker)
        if index != -1 and (cut == -1 or index < cut):
            cut = index
    if cut > 0:
        text = text[cut:]

    lines = []
    for line in text.splitlines():
        stripped = line.strip()
        if stripped.startswith("Session:") or stripped.startswith("session_id"):
            continue
        if (
            stripped.startswith("RSS fail")
            or stripped.startswith("Searching")
            or stripped.startswith("Running search")
        ):
            cleaned = re.sub(
                r"^.*?((?:Morning Check|\*\*1\)|\*\*Git).*)",
                r"\1",
                line,
            )
            if cleaned != line:
                line = cleaned
            else:
                continue
        lines.append(line)

    output = "\n".join(lines).strip()
    if lines:
        first_line = lines[0]
        match = re.search(r"(Morning Check.*)", first_line)
        if match and match.start() > 0:
            lines[0] = match.group(1)
            output = "\n".join(lines).strip()

    if len(output) < 40:
        sys.stderr.write("output too short\n")
        sys.exit(2)

    print(output)
    PY
  '';

  legacyMorningCheck = pkgs.writeText "vesper-sabah-check-deliver.sh" ''
    #!/usr/bin/env bash
    exec "${home}/.hermes/scripts/morning-check-deliver.sh" "$@"
  '';
in
{
  # Hermes resolves cron scripts with realpath and refuses symlinks that escape
  # ~/.hermes/scripts. Home Manager normally exposes home.file sources as Nix
  # store symlinks, so copy the Nix-owned scripts into Hermes' sandbox instead.
  home.activation.installHermesMorningCheckScripts = lib.hm.dag.entryAfter [ "writeBoundary" ] ''
    scripts_dir="${home}/.hermes/scripts"
    ${pkgs.coreutils}/bin/mkdir -p "$scripts_dir"

    ${pkgs.coreutils}/bin/rm -f \
      "$scripts_dir/morning-check-deliver.sh" \
      "$scripts_dir/sabah-check-deliver.sh"

    ${pkgs.coreutils}/bin/install -m 0755 ${morningCheckDeliver} "$scripts_dir/morning-check-deliver.sh"
    ${pkgs.coreutils}/bin/install -m 0755 ${legacyMorningCheck} "$scripts_dir/sabah-check-deliver.sh"
  '';
}
