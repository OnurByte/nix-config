def num_or_null:
  if . == null then null else (try tonumber catch null) end;

def clamp_percent:
  if . == null then null elif . < 0 then 0 elif . > 100 then 100 else . end;

def round1:
  if . == null then null else ((. * 10 | round) / 10) end;

def error_text:
  if . == null or . == false or . == "" then ""
  elif type == "string" then .[0:500]
  elif type == "object" then ((.message // .error // .detail // .description // (tojson)) | tostring | .[0:500])
  else (tostring | .[0:500]) end;

def normalise_window:
  . as $w
  | ($w.usedPercent | num_or_null | clamp_percent | round1) as $used
  | ($w.remainingPercent | num_or_null | clamp_percent | round1) as $remaining
  | {
      kind: ($w.kind // "" | tostring),
      label: ($w.label // $w.kind // "Usage" | tostring),
      usedPercent: $used,
      remainingPercent: $remaining,
      resetAt: ($w.resetAt // "" | tostring)
    };

def normalise_account:
  . as $a
  | ($a.identity | if type == "object" then . else {} end) as $identity
  | ($a.status | if type == "object" then . else {} end) as $status
  | [($a.windows // [])[] | select(type == "object") | normalise_window] as $windows
  | ([ $windows[].usedPercent | select(. != null) ] | if length > 0 then max else null end) as $maxUsed
  | {
      id: ($a.id // $a.accountId // $a.label // "" | tostring),
      label: ($a.label // $a.name // $identity.accountEmail // "" | tostring),
      plan: ($identity.plan // $a.plan // "" | tostring),
      account: ($identity.accountEmail // $a.account // "" | tostring),
      status: ($status.level // "unknown" | tostring | ascii_downcase),
      statusLabel: ($status.label // "" | tostring),
      windows: $windows,
      maxUsedPercent: $maxUsed,
      pace: ($a.pace // null),
      updatedAt: ($a.updatedAt // "" | tostring)
    };

def normalise_provider($staleAfterSeconds):
  . as $p
  | [($p.windows // [])[] | select(type == "object") | normalise_window] as $windows
  | [($p.accounts // [])[] | select(type == "object") | normalise_account] as $accounts
  | ($windows + [ $accounts[].windows[] ]) as $quotaWindows
  | ([ $quotaWindows[].usedPercent | select(. != null) ] | if length > 0 then max else null end) as $maxUsed
  | (if $maxUsed == null then null else (100 - $maxUsed | round1) end) as $remaining
  | ($p.identity | if type == "object" then . else {} end) as $identity
  | ($p.status | if type == "object" then . else {} end) as $status
  | ($p.display | if type == "object" then . else {} end) as $display
  | ($p.error | error_text) as $error
  | ($status.level // "unknown" | tostring | ascii_downcase) as $level
  | (if ($error != "" or $level == "critical" or ($remaining != null and $remaining <= 10)) then "critical"
     elif ($level == "warning" or ($remaining != null and $remaining <= 25)) then "warning"
     elif ($level == "unknown" and $maxUsed == null) then "unknown"
     elif ($level == "ok" or $level == "healthy" or $level == "normal" or $maxUsed != null) then "ok"
     else "unknown" end) as $health
  | {
      id: ($p.id // "unknown" | tostring),
      name: ($p.name // $p.id // "Unknown" | tostring),
      enabled: ($p.enabled // true | not | not),
      source: ($p.source // "" | tostring),
      plan: ($identity.plan // "" | tostring),
      account: ($identity.accountEmail // "" | tostring),
      accounts: $accounts,
      status: $level,
      statusLabel: ($status.label // "" | tostring),
      windows: $windows,
      maxUsedPercent: $maxUsed,
      remainingPercent: $remaining,
      credits: ($p.credits | if type == "object" then . else null end),
      cost: ($p.cost | if type == "object" then . else null end),
      pace: ($p.pace // null),
      sortKey: (try ($display.sortKey // 0 | tonumber | floor) catch 0),
      health: $health,
      error: $error,
      updatedAt: ($p.updatedAt // "" | tostring),
      staleAfterSeconds: $staleAfterSeconds
    };

.[0] as $raw
| (.[1] // {}) as $agents
| (.[2] // {}) as $hermes
| (.[3] // {}) as $privacy
| ($raw.staleAfterSeconds | num_or_null) as $staleAfterSeconds
| [($raw.providers // [])[] | select(type == "object") | normalise_provider($staleAfterSeconds) | select(.enabled)]
  | sort_by([.sortKey, (.name | ascii_downcase)]) as $providers
| [ $providers[] | select(.maxUsedPercent != null) ] as $constrained
| ($constrained | if length > 0 then max_by(.maxUsedPercent) else null end) as $worst
| ([ $providers[] | select(.health == "critical") ] | length) as $critical
| ([ $providers[] | select(.health == "warning") ] | length) as $warning
| ([ $providers[] | select(.health == "unknown") ] | length) as $unknown
| (if $critical > 0 then "critical"
   elif $warning > 0 then "warning"
   elif $unknown > 0 then "unknown"
   else "ok" end) as $overallHealth
| {
    schemaVersion: 2,
    generatedAt: ($raw.generatedAt // (now | todateiso8601) | tostring),
    normalisedAt: (now | todateiso8601),
    staleAfterSeconds: $staleAfterSeconds,
    stale: false,
    summary: {
      providerCount: ($providers | length),
      criticalCount: $critical,
      warningCount: $warning,
      unknownCount: $unknown,
      overallHealth: $overallHealth,
      mostConstrainedQuota: (if $worst == null then null else {
        providerId: $worst.id,
        providerName: $worst.name,
        usedPercent: $worst.maxUsedPercent,
        remainingPercent: $worst.remainingPercent
      } end),
      maxUsedPercent: (if $worst == null then -1 else ($worst.maxUsedPercent | round) end),
      maxProvider: (if $worst == null then "" else $worst.name end),
      class: $overallHealth
    },
    providers: $providers,
    agents: (if ($agents | type) == "object" then $agents else {} end),
    hermes: (if ($hermes | type) == "object" then $hermes else {} end),
    privacy: (if ($privacy | type) == "object" then $privacy else {} end),
    codexbar: {
      version: ($raw.host.codexBarVersion // "" | tostring),
      generatedAt: ($raw.generatedAt // "" | tostring),
      staleAfterSeconds: $staleAfterSeconds
    }
  }
