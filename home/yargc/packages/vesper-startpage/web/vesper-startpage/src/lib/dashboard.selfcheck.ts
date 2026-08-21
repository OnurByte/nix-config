import { isHistoryUrl, mergeHistory, normalizeHistoryItem, visibleDomain } from "./dashboard"

const http = normalizeHistoryItem({
  title: "Example",
  url: "https://example.com/path",
  visitedAt: "2026-08-21T10:00:00.000Z",
  browser: "Zen",
})
const newer = normalizeHistoryItem({
  title: "Example newer",
  url: "https://example.com/new",
  visitedAt: "2026-08-21T11:00:00.000Z",
  browser: "Helium",
})

if (!isHistoryUrl(http.url) || isHistoryUrl("https://hiddenexample.onion/")) {
  throw new Error("history URL boundary failed")
}
if (visibleDomain(http.url) !== "example.com") {
  throw new Error("domain formatting failed")
}
if (mergeHistory([http], [http, newer])[0].title !== newer.title) {
  throw new Error("history merge ordering failed")
}
console.log("vesper startpage selfcheck: ok")
