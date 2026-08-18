# Vesper Wellbeing

Use Vesper's local foreground-usage data as optional machine context.

## read contract

- Run `vesper-control wellbeing status` to learn whether collection is `on` or `off`.
- Run `vesper-control wellbeing-summary` for the read-only JSON summary.
- The summary contains today's total foreground time plus per-app foreground time.
- Treat an empty summary as valid data, not as an error.

## ownership

- Settings → Apps owns the Wellbeing toggle.
- Collection is enabled by default on a new profile.
- If the user turns it off, never turn it back on automatically.
- Agents are readers, not owners, of Wellbeing state.

## privacy

- Keep raw Wellbeing data local by default.
- Do not paste or sync raw app-usage history into external services, prompts, telemetry, issue reports or logs unless the user explicitly requests it.
- Prefer derived local decisions over reproducing the raw history.
- Never infer sensitive personal facts from application names alone.

## useful cases

Use the data only when it materially helps the current task, for example to identify which development tools are actually used, prioritize workflow integrations, or explain local app-usage patterns the user asked about.
