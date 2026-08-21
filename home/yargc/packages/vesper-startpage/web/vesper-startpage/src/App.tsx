import {
  Activity,
  ArrowUpRight,
  BrainCircuit,
  Clock3,
  ExternalLink,
  Globe2,
  LockKeyhole,
  Radar,
  RefreshCw,
  Shield,
  TerminalSquare,
} from "lucide-react"
import { useCallback, useEffect, useMemo, useState } from "react"

import {
  Badge,
  Button,
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
  Empty,
  EmptyDescription,
  EmptyHeader,
  EmptyMedia,
  EmptyTitle,
  ScrollArea,
  Separator,
  Skeleton,
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui"
import {
  formatTime,
  isHistoryUrl,
  mergeHistory,
  type Briefing,
  type HermesResponse,
  type HistoryItem,
  type HistoryResponse,
  type ShortcutsResponse,
  type TorItem,
  type TorResponse,
} from "@/lib/dashboard"

type SourceState<T> = {
  data: T | null
  loading: boolean
  error: boolean
}

const emptyHistory: HistoryResponse = {
  available: false,
  reason: "missing",
  stats: { totalVisits: 0, uniqueUrls: 0, todayVisits: 0 },
  items: [],
}

const emptyShortcuts: ShortcutsResponse = {
  available: false,
  reason: "missing",
  items: [],
}

const emptyHermes: HermesResponse = {
  available: false,
  reason: "missing",
  stats: { totalBriefings: 0, unread: 0, highPriority: 0, sourceCount: 0 },
  briefings: [],
  tiers: [],
  coverage: {},
}

const emptyTor: TorResponse = { available: true, items: [] }

async function loadJson<T>(path: string, signal: AbortSignal): Promise<T> {
  const response = await fetch(path, { signal, headers: { Accept: "application/json" } })
  if (!response.ok) {
    throw new Error(`${path}: ${response.status}`)
  }
  return response.json() as Promise<T>
}

function StatCard({ label, value, detail }: { label: string; value: string; detail: string }) {
  return (
    <Card size="sm" className="bg-card/80">
      <CardHeader>
        <CardDescription>{label}</CardDescription>
        <CardTitle className="font-mono text-2xl tracking-tight">{value}</CardTitle>
      </CardHeader>
      <CardFooter className="text-xs text-muted-foreground">{detail}</CardFooter>
    </Card>
  )
}

function LoadingBlock({ className = "h-24" }: { className?: string }) {
  return <Skeleton className={className} aria-label="yükleniyor" />
}

function Unavailable({ title, description }: { title: string; description: string }) {
  return (
    <Empty className="min-h-48 border border-dashed border-border/70 bg-muted/20">
      <EmptyHeader>
        <EmptyMedia variant="icon">
          <Activity aria-hidden="true" />
        </EmptyMedia>
        <EmptyTitle>{title}</EmptyTitle>
        <EmptyDescription>{description}</EmptyDescription>
      </EmptyHeader>
    </Empty>
  )
}

function ShortcutGrid({ state }: { state: SourceState<ShortcutsResponse> }) {
  const items = state.data?.items ?? emptyShortcuts.items

  if (state.loading) {
    return (
      <section aria-labelledby="shortcuts-heading" className="flex flex-col gap-3" aria-busy="true">
        <LoadingBlock className="h-7 w-48" />
        <div className="grid grid-cols-2 gap-2 sm:grid-cols-3 lg:grid-cols-5">
          {Array.from({ length: 10 }, (_, index) => (
            <LoadingBlock key={index} className="h-16" />
          ))}
        </div>
      </section>
    )
  }

  if (!state.data?.available || items.length === 0) {
    return (
      <section aria-labelledby="shortcuts-heading" className="flex flex-col gap-3">
        <div>
          <p className="eyebrow">quick launch</p>
          <h2 id="shortcuts-heading" className="text-xl font-semibold tracking-tight">
            Helium shortcuts
          </h2>
        </div>
        <Unavailable
          title="Helium shortcuts kullanılamıyor"
          description="Helium Preferences dosyası bulunamadı veya okunamadı."
        />
      </section>
    )
  }

  return (
    <section aria-labelledby="shortcuts-heading" className="flex flex-col gap-3">
      <div className="flex items-end justify-between gap-4">
        <div>
          <p className="eyebrow">quick launch</p>
          <h2 id="shortcuts-heading" className="text-xl font-semibold tracking-tight">
            Helium shortcuts
          </h2>
        </div>
        <Badge variant="outline">{items.length} pinned</Badge>
      </div>
      <div className="grid grid-cols-2 gap-2 sm:grid-cols-3 lg:grid-cols-5">
        {items.map((shortcut) => (
          <Button
            key={shortcut.url}
            asChild
            variant="outline"
            className="h-auto min-h-16 justify-between gap-3 rounded-xl bg-card/70 px-3 py-3 text-left hover:bg-muted"
          >
            <a href={shortcut.url} aria-label={`${shortcut.title}: ${shortcut.url}`}>
              <span className="flex min-w-0 flex-col gap-1">
                <span className="truncate font-medium">{shortcut.title}</span>
                <span className="truncate text-xs text-muted-foreground">{shortcut.domain}</span>
              </span>
              <ArrowUpRight data-icon="inline-end" aria-hidden="true" />
            </a>
          </Button>
        ))}
      </div>
    </section>
  )
}

function HistoryRows({ items, loading }: { items: HistoryItem[]; loading: boolean }) {
  if (loading) {
    return (
      <div className="flex flex-col gap-2" aria-busy="true">
        {Array.from({ length: 6 }, (_, index) => (
          <LoadingBlock key={index} className="h-14" />
        ))}
      </div>
    )
  }

  if (items.length === 0) {
    return (
      <Unavailable
        title="Geçmiş boş"
        description="Zen ve Helium’un okunabilir web geçmişinde gösterilecek kayıt yok."
      />
    )
  }

  return (
    <ScrollArea className="h-[26rem] pr-3">
      <div className="flex flex-col">
        {items.map((item) => (
          <a
            key={`${item.browser}-${item.url}-${item.visitedAt}`}
            href={item.url}
            className="group flex min-w-0 items-center gap-3 border-b border-border/60 py-3 outline-none transition-colors first:pt-1 last:border-b-0 hover:bg-muted/40 focus-visible:bg-muted/60"
          >
            <span className="flex size-8 shrink-0 items-center justify-center rounded-lg bg-muted text-muted-foreground">
              <Globe2 aria-hidden="true" />
            </span>
            <span className="flex min-w-0 flex-1 flex-col gap-1">
              <span className="truncate text-sm font-medium group-hover:text-primary">{item.title}</span>
              <span className="truncate text-xs text-muted-foreground">{item.domain}</span>
            </span>
            <Badge variant={item.browser === "Zen" ? "secondary" : "outline"}>{item.browser}</Badge>
            <time className="shrink-0 font-mono text-xs text-muted-foreground" dateTime={item.visitedAt}>
              {formatTime(item.visitedAt)}
            </time>
          </a>
        ))}
      </div>
    </ScrollArea>
  )
}

function HistoryPanel({ helium, zen }: { helium: SourceState<HistoryResponse>; zen: SourceState<HistoryResponse> }) {
  const items = useMemo(
    () => mergeHistory(helium.data?.items ?? [], zen.data?.items ?? []).filter((item) => isHistoryUrl(item.url)),
    [helium.data?.items, zen.data?.items]
  )
  const heliumStats = helium.data?.stats ?? emptyHistory.stats
  const zenStats = zen.data?.stats ?? emptyHistory.stats
  const unavailable = !helium.loading && !zen.loading && !helium.data?.available && !zen.data?.available

  return (
    <div className="grid gap-4 xl:grid-cols-[minmax(0,1.5fr)_minmax(18rem,0.8fr)]">
      <Card>
        <CardHeader>
          <div className="flex items-start justify-between gap-3">
            <div>
              <CardTitle>Recent browsing</CardTitle>
              <CardDescription>Zen + Helium · son 16 web ziyareti</CardDescription>
            </div>
            <Badge variant="outline">local only</Badge>
          </div>
        </CardHeader>
        <CardContent>
          {unavailable ? (
            <Unavailable title="Browser geçmişi kullanılamıyor" description="Zen veya Helium profil veritabanı bulunamadı." />
          ) : (
            <HistoryRows items={items} loading={helium.loading || zen.loading} />
          )}
        </CardContent>
      </Card>
      <div className="grid gap-4 sm:grid-cols-2 xl:grid-cols-1">
        <StatCard label="Zen visits" value={zenStats.totalVisits.toLocaleString("tr-TR")} detail={`${zenStats.uniqueUrls.toLocaleString("tr-TR")} benzersiz URL`} />
        <StatCard label="Helium visits" value={heliumStats.totalVisits.toLocaleString("tr-TR")} detail={`${heliumStats.uniqueUrls.toLocaleString("tr-TR")} benzersiz URL`} />
        <StatCard label="Today" value={(zenStats.todayVisits + heliumStats.todayVisits).toLocaleString("tr-TR")} detail="iki browser · bugün" />
      </div>
    </div>
  )
}

function HermesPanel({ state }: { state: SourceState<HermesResponse> }) {
  const data = state.data ?? emptyHermes
  if (state.loading) {
    return <LoadingBlock className="h-72" />
  }
  if (!data.available) {
    return <Unavailable title="Hermes beklemede" description="Henüz briefing veya source registry çıktısı yok." />
  }

  return (
    <div className="flex flex-col gap-4">
      <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
        <StatCard label="Briefings" value={data.stats.totalBriefings.toLocaleString("tr-TR")} detail={`${data.stats.unread} okunmamış`} />
        <StatCard label="High priority" value={data.stats.highPriority.toLocaleString("tr-TR")} detail="öncelikli araştırma" />
        <StatCard label="Sources" value={data.stats.sourceCount.toLocaleString("tr-TR")} detail="registry kayıtları" />
        <StatCard label="Coverage" value={Object.values(data.coverage).reduce((sum, value) => sum + value, 0).toLocaleString("tr-TR")} detail="son rapor sinyalleri" />
      </div>
      <div className="grid gap-4 xl:grid-cols-[minmax(0,1.4fr)_minmax(18rem,0.6fr)]">
        <Card>
          <CardHeader>
            <CardTitle>Latest research</CardTitle>
            <CardDescription>Hermes’in yazdığı briefing index’inden okunur.</CardDescription>
          </CardHeader>
          <CardContent className="flex flex-col gap-3">
            {data.briefings.map((briefing) => (
              <BriefingCard key={briefing.id} briefing={briefing} />
            ))}
          </CardContent>
        </Card>
        <Card>
          <CardHeader>
            <CardTitle>Source tiers</CardTitle>
            <CardDescription>Hermes registry görünümü</CardDescription>
          </CardHeader>
          <CardContent className="flex flex-wrap gap-2">
            {data.tiers.length === 0 ? (
              <Badge variant="outline">tier verisi yok</Badge>
            ) : (
              data.tiers.map((tier) => (
                <Badge key={tier.tier} variant="secondary">
                  {tier.tier} · {tier.count}
                </Badge>
              ))
            )}
          </CardContent>
          <CardFooter className="flex flex-col items-start gap-2 text-xs text-muted-foreground">
            {Object.entries(data.coverage).map(([key, value]) => (
              <span key={key} className="flex w-full justify-between gap-4">
                <span>{key}</span>
                <span className="font-mono">{value.toLocaleString("tr-TR")}</span>
              </span>
            ))}
          </CardFooter>
        </Card>
      </div>
    </div>
  )
}

function BriefingCard({ briefing }: { briefing: Briefing }) {
  return (
    <Card size="sm" className="bg-muted/25">
      <CardHeader>
        <div className="flex items-start justify-between gap-3">
          <div className="min-w-0">
            <CardTitle className="truncate text-sm">{briefing.title}</CardTitle>
            <CardDescription className="mt-1">{briefing.lane} · {briefing.createdAt ? formatTime(briefing.createdAt) : "zaman yok"}</CardDescription>
          </div>
          <div className="flex shrink-0 gap-1">
            {briefing.unread ? <Badge>unread</Badge> : null}
            <Badge variant={briefing.priority === "high" || briefing.priority === "urgent" ? "destructive" : "outline"}>{briefing.priority}</Badge>
          </div>
        </div>
      </CardHeader>
      <CardContent>
        <p className="line-clamp-3 text-sm leading-relaxed text-muted-foreground">{briefing.summary || "Özet yok."}</p>
      </CardContent>
      <CardFooter className="justify-between gap-3 text-xs text-muted-foreground">
        <span>{briefing.sourceCount} kaynak</span>
        <span>confidence {briefing.confidence}</span>
      </CardFooter>
    </Card>
  )
}

function TorPanel({ state }: { state: SourceState<TorResponse> }) {
  const [opening, setOpening] = useState<string | null>(null)
  const [message, setMessage] = useState<string | null>(null)
  const items = state.data?.items ?? emptyTor.items

  const openInTor = useCallback(async (item: TorItem) => {
    setOpening(item.id)
    setMessage(null)
    try {
      const response = await fetch(`/api/tor/open/${encodeURIComponent(item.id)}`, {
        method: "POST",
        headers: { Accept: "application/json" },
      })
      if (!response.ok) {
        throw new Error(`Tor Browser açılamadı (${response.status})`)
      }
      setMessage(`${item.name} Tor Browser’a gönderildi.`)
    } catch (error) {
      setMessage(error instanceof Error ? error.message : "Tor Browser açılamadı.")
    } finally {
      setOpening(null)
    }
  }, [])

  if (state.loading) {
    return <LoadingBlock className="h-72" />
  }

  const pinnedCount = items.filter((item) => item.source === "pinned").length
  const hermesCount = items.filter((item) => item.source === "hermes").length

  return (
    <div className="flex flex-col gap-4">
      <div className="flex items-center gap-3 rounded-xl border border-border/70 bg-muted/20 p-4 text-sm text-muted-foreground">
        <Shield aria-hidden="true" />
        <span>Bu kaynaklar Zen veya Helium’da açılmaz. Buton yalnızca Nix tarafından yönetilen Tor Browser’ı çağırır.</span>
      </div>
      <div className="grid gap-4 sm:grid-cols-3">
        <StatCard label="Onion links" value={items.length.toLocaleString("tr-TR")} detail="gösterilen kaynak" />
        <StatCard label="Pinned" value={pinnedCount.toLocaleString("tr-TR")} detail="Dread + Pitch" />
        <StatCard label="Hermes" value={hermesCount.toLocaleString("tr-TR")} detail="registry’den gelen" />
      </div>
      <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-3">
        {items.map((item) => (
          <Card key={item.id}>
            <CardHeader>
              <div className="flex items-start justify-between gap-3">
                <div>
                  <CardTitle>{item.name}</CardTitle>
                  <CardDescription className="mt-1 font-mono text-xs">{item.displayHost}</CardDescription>
                </div>
                <Badge variant={item.source === "pinned" ? "secondary" : "outline"}>{item.source}</Badge>
              </div>
            </CardHeader>
            <CardContent className="flex flex-wrap gap-2 text-xs text-muted-foreground">
              {item.tier ? <Badge variant="outline">{item.tier}</Badge> : null}
              {typeof item.hits === "number" ? <Badge variant="outline">{item.hits} hits</Badge> : null}
              {item.lastUseful ? <span className="self-center">son useful {item.lastUseful}</span> : null}
            </CardContent>
            <CardFooter>
              <Button className="w-full" variant="outline" onClick={() => void openInTor(item)} disabled={opening !== null}>
                <LockKeyhole data-icon="inline-start" aria-hidden="true" />
                {opening === item.id ? "Tor Browser açılıyor…" : "Tor Browser’da aç"}
              </Button>
            </CardFooter>
          </Card>
        ))}
      </div>
      {message ? <p className="text-sm text-muted-foreground" role="status">{message}</p> : null}
      {items.length === 0 ? <Unavailable title="Tor kaynağı yok" description="Dread, Pitch veya Hermes registry içinde doğrulanmış onion kaydı yok." /> : null}
    </div>
  )
}

function App() {
  const [helium, setHelium] = useState<SourceState<HistoryResponse>>({ data: null, loading: true, error: false })
  const [zen, setZen] = useState<SourceState<HistoryResponse>>({ data: null, loading: true, error: false })
  const [shortcuts, setShortcuts] = useState<SourceState<ShortcutsResponse>>({ data: null, loading: true, error: false })
  const [hermes, setHermes] = useState<SourceState<HermesResponse>>({ data: null, loading: true, error: false })
  const [tor, setTor] = useState<SourceState<TorResponse>>({ data: null, loading: true, error: false })
  const [refreshedAt, setRefreshedAt] = useState<Date | null>(null)

  const refresh = useCallback(() => {
    const controller = new AbortController()
    setHelium((state) => ({ ...state, loading: true, error: false }))
    setZen((state) => ({ ...state, loading: true, error: false }))
    setShortcuts((state) => ({ ...state, loading: true, error: false }))
    setHermes((state) => ({ ...state, loading: true, error: false }))
    setTor((state) => ({ ...state, loading: true, error: false }))

    void Promise.all([
      loadJson<HistoryResponse>("/api/history/helium", controller.signal)
        .then((data) => setHelium({ data, loading: false, error: false }))
        .catch(() => setHelium({ data: emptyHistory, loading: false, error: true })),
      loadJson<HistoryResponse>("/api/history/zen", controller.signal)
        .then((data) => setZen({ data, loading: false, error: false }))
        .catch(() => setZen({ data: emptyHistory, loading: false, error: true })),
      loadJson<ShortcutsResponse>("/api/shortcuts", controller.signal)
        .then((data) => setShortcuts({ data, loading: false, error: false }))
        .catch(() => setShortcuts({ data: emptyShortcuts, loading: false, error: true })),
      loadJson<HermesResponse>("/api/hermes", controller.signal)
        .then((data) => setHermes({ data, loading: false, error: false }))
        .catch(() => setHermes({ data: emptyHermes, loading: false, error: true })),
      loadJson<TorResponse>("/api/tor", controller.signal)
        .then((data) => setTor({ data, loading: false, error: false }))
        .catch(() => setTor({ data: emptyTor, loading: false, error: true })),
    ]).finally(() => setRefreshedAt(new Date()))

    return () => controller.abort()
  }, [])

  useEffect(() => {
    let cleanup = () => {}
    const timer = window.setTimeout(() => {
      cleanup = refresh()
    }, 0)
    return () => {
      window.clearTimeout(timer)
      cleanup()
    }
  }, [refresh])

  return (
    <TooltipProvider>
      <main className="mx-auto flex min-h-svh w-full max-w-[1440px] flex-col gap-8 px-5 py-5 sm:px-8 sm:py-8">
        <header className="glass-panel flex flex-col gap-5 rounded-2xl p-5 sm:flex-row sm:items-end sm:justify-between sm:p-7">
          <div className="flex flex-col gap-3">
            <div className="flex items-center gap-3">
              <div className="flex size-10 items-center justify-center rounded-xl bg-primary text-primary-foreground">
                <TerminalSquare aria-hidden="true" />
              </div>
              <div>
                <p className="eyebrow">vesper / local startpage</p>
                <h1 className="text-3xl font-semibold tracking-[-0.04em] sm:text-4xl">Your surface for the unknown.</h1>
              </div>
            </div>
            <p className="max-w-2xl text-sm leading-relaxed text-muted-foreground">
              Browser akışı, Hermes araştırması ve Tor kaynakları tek yerel yüzeyde. Veriler gerçek profillerden okunur; boşsa boş görünür.
            </p>
          </div>
          <div className="flex items-center gap-2 text-xs text-muted-foreground">
            <Tooltip>
              <TooltipTrigger asChild>
                <span className="flex items-center gap-2 rounded-full border border-border/70 bg-background/50 px-3 py-2">
                  <span className="size-2 rounded-full bg-primary" aria-hidden="true" />
                  loopback · 3210
                </span>
              </TooltipTrigger>
              <TooltipContent>Vesper startpage yalnızca bu makinede dinlenir.</TooltipContent>
            </Tooltip>
            <Button variant="ghost" size="icon-sm" onClick={refresh} aria-label="Verileri yenile">
              <RefreshCw data-icon="inline-start" aria-hidden="true" />
            </Button>
            {refreshedAt ? <time dateTime={refreshedAt.toISOString()}>{formatTime(refreshedAt.toISOString())}</time> : null}
          </div>
        </header>

        <ShortcutGrid state={shortcuts} />
        <Separator />

        <Tabs defaultValue="recent" className="flex flex-col gap-5">
          <TabsList variant="line" aria-label="Vesper bölümleri">
            <TabsTrigger value="recent">
              <Clock3 data-icon="inline-start" aria-hidden="true" />
              recent
            </TabsTrigger>
            <TabsTrigger value="hermes">
              <BrainCircuit data-icon="inline-start" aria-hidden="true" />
              Hermes
            </TabsTrigger>
            <TabsTrigger value="tor">
              <Radar data-icon="inline-start" aria-hidden="true" />
              Tor
            </TabsTrigger>
          </TabsList>
          <TabsContent value="recent">
            <HistoryPanel helium={helium} zen={zen} />
          </TabsContent>
          <TabsContent value="hermes">
            <HermesPanel state={hermes} />
          </TabsContent>
          <TabsContent value="tor">
            <TorPanel state={tor} />
          </TabsContent>
        </Tabs>

        <footer className="flex flex-col gap-2 border-t border-border/60 pt-4 text-xs text-muted-foreground sm:flex-row sm:items-center sm:justify-between">
          <span>Vesper local surface · no remote sync</span>
          <span className="flex items-center gap-1"><ExternalLink aria-hidden="true" /> normal web links stay in the active browser</span>
        </footer>
      </main>
    </TooltipProvider>
  )
}

export default App
