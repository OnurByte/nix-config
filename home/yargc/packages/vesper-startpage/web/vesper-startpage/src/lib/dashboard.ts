export type BrowserName = "Zen" | "Helium"

export type Shortcut = {
  title: string
  url: string
  domain: string
}

export type ShortcutsResponse = {
  available: boolean
  reason?: "missing" | "unreadable" | "invalid"
  items: Shortcut[]
}

export type HistoryItem = {
  title: string
  url: string
  domain: string
  visitedAt: string
  browser: BrowserName
}

export type HistoryStats = {
  totalVisits: number
  uniqueUrls: number
  todayVisits: number
}

export type HistoryResponse = {
  available: boolean
  reason?: "missing" | "unreadable" | "invalid"
  stats: HistoryStats
  items: HistoryItem[]
}

export type Briefing = {
  id: string
  lane: string
  title: string
  summary: string
  priority: string
  confidence: string
  createdAt: string
  unread: boolean
  sourceCount: number
}

export type TierCount = { tier: string; count: number }

export type HermesResponse = {
  available: boolean
  reason?: "missing" | "unreadable" | "invalid"
  stats: {
    totalBriefings: number
    unread: number
    highPriority: number
    sourceCount: number
  }
  briefings: Briefing[]
  tiers: TierCount[]
  coverage: Record<string, number>
}

export type TorItem = {
  id: string
  name: string
  displayHost: string
  source: "pinned" | "hermes"
  tier?: string
  hits?: number
  lastUseful?: string
}

export type TorResponse = { available: boolean; items: TorItem[] }

export function isHistoryUrl(value: string): boolean {
  try {
    const url = new URL(value)
    return (url.protocol === "http:" || url.protocol === "https:") && !url.hostname.toLowerCase().endsWith(".onion")
  } catch {
    return false
  }
}

export function visibleDomain(value: string): string {
  try {
    return new URL(value).hostname
  } catch {
    return "unknown domain"
  }
}

export function mergeHistory(...sources: HistoryItem[][]): HistoryItem[] {
  const seen = new Set<string>()
  return sources
    .flat()
    .filter((item) => {
      const key = `${item.browser}:${item.url}:${item.visitedAt}`
      if (seen.has(key)) {
        return false
      }
      seen.add(key)
      return isHistoryUrl(item.url)
    })
    .sort((left, right) => Date.parse(right.visitedAt) - Date.parse(left.visitedAt))
    .slice(0, 16)
}

export function formatTime(value: string): string {
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) {
    return "--:--"
  }
  return new Intl.DateTimeFormat("tr-TR", { hour: "2-digit", minute: "2-digit" }).format(date)
}

export function normalizeHistoryItem(raw: Omit<HistoryItem, "domain">): HistoryItem {
  return { ...raw, domain: visibleDomain(raw.url) }
}
