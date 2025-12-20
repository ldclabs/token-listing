<script lang="ts">
  import type {
    AuctionConfig,
    AuctionInfo,
    AuctionSnapshot
  } from '$declarations/ic_auction/ic_auction.did'
  import type { TokenDisplay, TokenInfo } from '$lib/utils/token'

  // interface AuctionSnapshot {
  //   'c' : bigint, // clearing_price
  //   'd' : bigint, // cumulative_demand_raised
  //   's' : bigint, // cumulative_supply_released
  //   't' : bigint, // timestamp
  // }

  interface Props {
    snapshots: AuctionSnapshot[]
    auctionInfo: AuctionInfo | null
    auctionCfg: AuctionConfig | null
    currencyInfo: TokenInfo
    currencyDisplay: TokenDisplay
    priceUnitsPerToken: (priceAtomic: bigint) => string
  }

  let {
    snapshots,
    auctionInfo,
    auctionCfg,
    currencyInfo,
    currencyDisplay,
    priceUnitsPerToken
  }: Props = $props()

  const allSnapshots = $derived.by(() => {
    if (snapshots.length === 0) return []
    const list = [...snapshots]
    const lastS = list[list.length - 1]
    if (!lastS) return list

    if (auctionInfo && auctionCfg) {
      const now = BigInt(Date.now())
      const endTime = auctionCfg.end_time
      const targetTime = now > endTime ? endTime : now

      if (targetTime > lastS.t) {
        list.push({
          c: auctionInfo.clearing_price,
          d: auctionInfo.cumulative_demand_raised,
          s: auctionInfo.cumulative_supply_released,
          t: targetTime
        })
      }
    }

    // Ensure chronological order; some sources may provide out-of-order snapshots.
    list.sort((a, b) => (a.t < b.t ? -1 : a.t > b.t ? 1 : 0))

    // Deduplicate by timestamp (keep the latest entry per timestamp).
    const deduped: AuctionSnapshot[] = []
    for (const s of list) {
      const prev = deduped[deduped.length - 1]
      if (prev && prev.t === s.t) deduped[deduped.length - 1] = s
      else deduped.push(s)
    }

    return deduped
  })

  const chartData = $derived.by(() => {
    if (allSnapshots.length === 0) return []
    const maxPoints = 300
    if (allSnapshots.length <= maxPoints) return allSnapshots
    const step = Math.floor(allSnapshots.length / maxPoints)
    const result = []
    for (let i = 0; i < allSnapshots.length; i += step) {
      result.push(allSnapshots[i])
    }
    if (result[result.length - 1] !== allSnapshots[allSnapshots.length - 1]) {
      result.push(allSnapshots[allSnapshots.length - 1])
    }
    return result
  })

  const chartBounds = $derived.by(() => {
    if (allSnapshots.length === 0) return null
    const first = allSnapshots[0]
    const last = allSnapshots[allSnapshots.length - 1]
    if (!first || !last) return null

    let minP = first.c
    let maxP = first.c
    let minD = first.d
    let maxD = first.d
    const minT = first.t
    const maxT = last.t

    for (const s of allSnapshots) {
      if (!s) continue
      if (s.c < minP) minP = s.c
      if (s.c > maxP) maxP = s.c
      if (s.d < minD) minD = s.d
      if (s.d > maxD) maxD = s.d
    }

    return {
      minP,
      maxP: maxP > minP ? maxP : minP + 1n,
      minD,
      maxD: maxD > minD ? maxD : minD + 1n,
      minT,
      maxT: maxT > minT ? maxT : minT + 1n
    }
  })

  const svgW = 1000
  const svgH = 420
  const padL = 64
  const padR = 76
  const padT = 16
  const padB = 44
  const plotW = svgW - padL - padR
  const plotH = svgH - padT - padB

  const clamp = (n: number, min: number, max: number) =>
    Math.max(min, Math.min(max, n))

  const safeNumber = (v: bigint) => {
    const n = Number(v)
    return Number.isFinite(n) ? n : 0
  }

  const formatTime = (t: bigint) => {
    const ms = safeNumber(t)
    const d = new Date(ms)
    if (Number.isNaN(d.getTime())) return ''
    const hh = String(d.getHours()).padStart(2, '0')
    const mm = String(d.getMinutes()).padStart(2, '0')
    return `${hh}:${mm}`
  }

  type XY = { x: number; y: number }

  // Step-after path: value holds until next snapshot time, then jumps.
  const toStepPath = (points: XY[]) => {
    if (points.length === 0) return ''
    const first = points[0]!
    let d = `M ${first.x} ${first.y}`

    for (let i = 1; i < points.length; i++) {
      const prev = points[i - 1]!
      const curr = points[i]!

      // Move horizontally to the next time at the previous value, then jump.
      d += ` L ${curr.x} ${prev.y} L ${curr.x} ${curr.y}`
    }

    return d
  }

  const toStepFillPath = (points: XY[], bottomY: number) => {
    if (points.length === 0) return ''
    const first = points[0]!
    const last = points[points.length - 1]!
    let d = `M ${first.x} ${bottomY} L ${first.x} ${first.y}`

    for (let i = 1; i < points.length; i++) {
      const prev = points[i - 1]!
      const curr = points[i]!
      d += ` L ${curr.x} ${prev.y} L ${curr.x} ${curr.y}`
    }

    d += ` L ${last.x} ${bottomY} Z`
    return d
  }

  const scaled = $derived.by(() => {
    if (!chartBounds || chartData.length === 0) return []

    const t0 = safeNumber(chartBounds.minT)
    const t1 = safeNumber(chartBounds.maxT)
    const p0 = safeNumber(chartBounds.minP)
    const p1 = safeNumber(chartBounds.maxP)
    const d0 = safeNumber(chartBounds.minD)
    const d1 = safeNumber(chartBounds.maxD)

    const dt = t1 - t0 || 1
    const dp = p1 - p0 || 1
    const dd = d1 - d0 || 1

    return chartData
      .filter((s): s is AuctionSnapshot => Boolean(s))
      .map((s) => {
        const x = padL + ((safeNumber(s.t) - t0) * plotW) / dt
        const yP = padT + plotH - ((safeNumber(s.c) - p0) * plotH) / dp
        const yD = padT + plotH - ((safeNumber(s.d) - d0) * plotH) / dd
        return {
          t: s.t,
          c: s.c,
          d: s.d,
          x: clamp(x, padL, padL + plotW),
          yP: clamp(yP, padT, padT + plotH),
          yD: clamp(yD, padT, padT + plotH)
        }
      })
  })

  const pricePath = $derived.by(() => {
    if (scaled.length < 2) return ''
    return toStepPath(scaled.map((p) => ({ x: p.x, y: p.yP })))
  })

  const priceFillPath = $derived.by(() => {
    if (scaled.length < 2) return ''
    return toStepFillPath(
      scaled.map((p) => ({ x: p.x, y: p.yP })),
      padT + plotH
    )
  })

  const demandPath = $derived.by(() => {
    if (scaled.length < 2) return ''
    return toStepPath(scaled.map((p) => ({ x: p.x, y: p.yD })))
  })

  const ticks = $derived.by(() => {
    if (!chartBounds) return null
    const steps = 4
    const price = Array.from({ length: steps + 1 }, (_, i) => {
      const r = i / steps
      const v =
        chartBounds.maxP -
        ((chartBounds.maxP - chartBounds.minP) * BigInt(i)) / BigInt(steps)
      const y = padT + plotH * r
      return { v, y }
    })
    const demand = Array.from({ length: steps + 1 }, (_, i) => {
      const r = i / steps
      const v =
        chartBounds.maxD -
        ((chartBounds.maxD - chartBounds.minD) * BigInt(i)) / BigInt(steps)
      const y = padT + plotH * r
      return { v, y }
    })
    return {
      price,
      demand,
      t0: chartBounds.minT,
      t1: chartBounds.maxT
    }
  })

  let containerEl: HTMLDivElement | null = $state(null)
  let hoverIndex = $state<number | null>(null)
  let hoverX = $state<number | null>(null)
  let tooltipX = $state(0)
  let tooltipY = $state(0)

  const setHoverFromClientX = (clientX: number, clientY: number) => {
    if (!containerEl || scaled.length === 0) return
    const rect = containerEl.getBoundingClientRect()
    const localX = ((clientX - rect.left) / rect.width) * svgW
    const localY = ((clientY - rect.top) / rect.height) * svgH

    const x = clamp(localX, padL, padL + plotW)
    hoverX = x

    // nearest point by x (linear scan is ok for <=300, but keep it efficient)
    let lo = 0
    let hi = scaled.length - 1
    while (hi - lo > 1) {
      const mid = (lo + hi) >> 1
      if (scaled[mid]!.x < x) lo = mid
      else hi = mid
    }
    const pick =
      Math.abs(scaled[lo]!.x - x) <= Math.abs(scaled[hi]!.x - x) ? lo : hi
    hoverIndex = pick

    // tooltip position (keep inside container)
    const tipW = 220
    const tipH = 64
    const pad = 10
    const px = clamp(
      ((x / svgW) * rect.width) | 0,
      pad,
      rect.width - tipW - pad
    )
    const py = clamp(
      ((clamp(localY, padT, padT + plotH) / svgH) * rect.height) | 0,
      pad,
      rect.height - tipH - pad
    )
    tooltipX = px
    tooltipY = py
  }

  const clearHover = () => {
    hoverIndex = null
    hoverX = null
  }
</script>

{#if snapshots.length > 1 && chartBounds}
  <section class="glass-border rounded-xl p-4 md:p-6">
    <div class="mb-6 flex items-center justify-between">
      <div class="space-y-1">
        <div class="text-muted text-xs font-semibold tracking-wide uppercase"
          >Analytics</div
        >
        <div class="text-lg font-bold">Price Discovery</div>
      </div>
      <div class="text-muted text-xs">
        {snapshots.length} snapshots recorded
      </div>
    </div>

    <div class="relative w-full" bind:this={containerEl}>
      <div class="relative h-72 w-full">
        <svg
          class="h-full w-full overflow-visible"
          viewBox={`0 0 ${svgW} ${svgH}`}
          role="img"
          aria-label="Price discovery chart"
          onpointermove={(e) => setHoverFromClientX(e.clientX, e.clientY)}
          onpointerleave={clearHover}
        >
          <defs>
            <linearGradient id="priceGradient" x1="0" y1="0" x2="0" y2="1">
              <stop
                offset="0%"
                stop-color="rgb(168, 85, 247)"
                stop-opacity="0.2"
              />
              <stop
                offset="100%"
                stop-color="rgb(168, 85, 247)"
                stop-opacity="0"
              />
            </linearGradient>
          </defs>

          <!-- Plot background -->
          <rect
            x={padL}
            y={padT}
            width={plotW}
            height={plotH}
            rx="12"
            fill="currentColor"
            class="text-muted opacity-[0.03]"
          />

          <!-- Grid (horizontal) -->
          {#if ticks}
            {#each ticks.price as tk (tk.y)}
              <line
                x1={padL}
                y1={tk.y}
                x2={padL + plotW}
                y2={tk.y}
                stroke="currentColor"
                stroke-dasharray="4 4"
                class="text-muted opacity-10"
              />
            {/each}
          {/if}

          <!-- Grid (vertical) -->
          {#each [0, 0.25, 0.5, 0.75, 1] as r (r)}
            <line
              x1={padL + plotW * r}
              y1={padT}
              x2={padL + plotW * r}
              y2={padT + plotH}
              stroke="currentColor"
              stroke-dasharray="4 4"
              class="text-muted opacity-10"
            />
          {/each}

          <!-- Price Fill -->
          {#if priceFillPath}
            <path d={priceFillPath} fill="url(#priceGradient)" />
          {/if}

          <!-- Price line -->
          {#if pricePath}
            <path
              d={pricePath}
              fill="none"
              stroke="currentColor"
              stroke-width="2.5"
              stroke-linejoin="round"
              stroke-linecap="round"
              class="text-purple-500"
            />
          {/if}

          <!-- Demand line -->
          {#if demandPath}
            <path
              d={demandPath}
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-dasharray="6 4"
              stroke-linejoin="round"
              stroke-linecap="round"
              class="text-amber-500 opacity-50"
            />
          {/if}

          <!-- Last point highlight -->
          {#if scaled.length > 0}
            {@const last = scaled[scaled.length - 1]!}
            <circle
              cx={last.x}
              cy={last.yP}
              r="4"
              fill="currentColor"
              class="text-purple-500"
            />
            <circle
              cx={last.x}
              cy={last.yP}
              r="8"
              fill="currentColor"
              class="animate-pulse text-purple-500/20"
            />
          {/if}

          <!-- Hover crosshair + markers -->
          {#if hoverIndex !== null && hoverX !== null && scaled[hoverIndex]}
            {@const hp = scaled[hoverIndex]!}
            <line
              x1={hp.x}
              y1={padT}
              x2={hp.x}
              y2={padT + plotH}
              stroke="currentColor"
              stroke-width="1"
              class="text-muted opacity-40"
            />
            <circle
              cx={hp.x}
              cy={hp.yP}
              r="5"
              fill="white"
              stroke="rgb(168, 85, 247)"
              stroke-width="2"
            />
            <circle
              cx={hp.x}
              cy={hp.yD}
              r="4"
              fill="white"
              stroke="rgb(245, 158, 11)"
              stroke-width="2"
              class="opacity-80"
            />
          {/if}

          <!-- Axes labels (left: price) -->
          {#if ticks}
            {#each ticks.price as tk (tk.y)}
              <text
                x={padL - 12}
                y={tk.y + 4}
                text-anchor="end"
                class="fill-current text-purple-500/80"
                font-size="11"
                font-weight="500"
              >
                {priceUnitsPerToken(tk.v)}
              </text>
            {/each}

            <!-- Axes labels (right: demand) -->
            {#each ticks.demand as tk (tk.y)}
              <text
                x={padL + plotW + 12}
                y={tk.y + 4}
                text-anchor="start"
                class="fill-current text-amber-500/70"
                font-size="11"
                font-weight="500"
              >
                {currencyDisplay.displayValue(tk.v)}
              </text>
            {/each}

            <!-- X axis time range -->
            <text
              x={padL}
              y={padT + plotH + 28}
              text-anchor="start"
              class="text-muted fill-current font-medium"
              font-size="11"
              opacity="0.7"
            >
              {formatTime(ticks.t0)}
            </text>
            <text
              x={padL + plotW}
              y={padT + plotH + 28}
              text-anchor="end"
              class="text-muted fill-current font-medium"
              font-size="11"
              opacity="0.7"
            >
              {formatTime(ticks.t1)}
            </text>
          {/if}
        </svg>

        <!-- Tooltip -->
        {#if hoverIndex !== null && scaled[hoverIndex]}
          {@const hp = scaled[hoverIndex]!}
          <div
            class="pointer-events-none absolute rounded-xl border border-white/10 bg-slate-900/80 p-3 text-[12px] shadow-2xl backdrop-blur-md dark:bg-slate-950/90"
            style={`left:${tooltipX}px; top:${tooltipY}px; width:220px; z-index: 50;`}
          >
            <div
              class="mb-2 flex items-center justify-between border-b border-white/5 pb-1.5"
            >
              <div class="font-bold text-white">{formatTime(hp.t)}</div>
              <div
                class="text-muted text-[10px] font-bold tracking-wider uppercase"
                >{currencyInfo.symbol}</div
              >
            </div>
            <div class="space-y-2">
              <div class="flex items-center justify-between">
                <div class="flex items-center gap-1.5">
                  <div class="h-2 w-2 rounded-full bg-purple-500"></div>
                  <span class="text-muted">Clearing Price</span>
                </div>
                <span class="font-bold text-white"
                  >{priceUnitsPerToken(hp.c)}</span
                >
              </div>
              <div class="flex items-center justify-between">
                <div class="flex items-center gap-1.5">
                  <div class="h-2 w-2 rounded-full bg-amber-500"></div>
                  <span class="text-muted">Cumulative Demand</span>
                </div>
                <span class="font-bold text-white"
                  >{currencyDisplay.displayValue(hp.d)}
                  {currencyInfo.symbol}</span
                >
              </div>
            </div>
          </div>
        {/if}
      </div>
    </div>

    <div class="mt-6 flex flex-wrap items-center gap-6 text-xs">
      <div class="flex items-center gap-2">
        <div class="h-0.5 w-6 bg-purple-500"></div>
        <span class="text-muted font-semibold uppercase">Clearing Price</span>
      </div>
      <div class="flex items-center gap-2">
        <div class="h-0.5 w-6 border-t-2 border-dashed border-amber-500"></div>
        <span class="text-muted font-semibold uppercase">Cumulative Demand</span
        >
      </div>
    </div>
  </section>
{/if}
