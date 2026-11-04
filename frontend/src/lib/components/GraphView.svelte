<script lang="ts">
  import { tick } from 'svelte';
  import { graphData, forceLayout, type NoteIndex, type GraphNode, type GraphEdge } from '$lib/utils/knowledge';

  interface Props {
    open: boolean;
    index: NoteIndex | null;
    currentKey: string;
    onOpen: (path: string) => void;
    onClose: () => void;
  }
  let { open, index, currentKey, onOpen, onClose }: Props = $props();

  const W = 1200;
  const H = 800;
  let canvas = $state<HTMLCanvasElement | null>(null);
  let localOnly = $state(false);
  let showUnresolved = $state(false);
  // Drawn imperatively on canvas; only the counts shown in the header are reactive.
  let nodes: GraphNode[] = $state.raw([]);
  let edges: GraphEdge[] = $state.raw([]);
  let view = { x: 0, y: 0, k: 1 };
  let hover: GraphNode | null = null;
  let drag: { sx: number; sy: number; vx: number; vy: number } | null = null;

  function compute() {
    if (!index) return;
    const g = graphData(index, showUnresolved);
    let ns = g.nodes;
    let es = g.edges;
    if (localOnly && currentKey) {
      const keep = new Set([currentKey]);
      for (const e of es) {
        if (e.source === currentKey) keep.add(e.target);
        if (e.target === currentKey) keep.add(e.source);
      }
      ns = ns.filter((n) => keep.has(n.id));
      es = es.filter((e) => keep.has(e.source) && keep.has(e.target));
    }
    forceLayout(ns, es, W, H, ns.length > 600 ? 120 : 300);
    nodes = ns;
    edges = es;
    view = { x: 0, y: 0, k: 1 };
    draw();
  }

  function draw() {
    const ctx = canvas?.getContext('2d');
    if (!ctx || !canvas) return;
    const dpr = window.devicePixelRatio || 1;
    const rect = canvas.getBoundingClientRect();
    canvas.width = rect.width * dpr;
    canvas.height = rect.height * dpr;
    const s = Math.min(rect.width / W, rect.height / H) * view.k;
    ctx.setTransform(dpr * s, 0, 0, dpr * s, dpr * view.x, dpr * view.y);
    ctx.clearRect(-view.x / s, -view.y / s, rect.width / s, rect.height / s);
    const pos = new Map(nodes.map((n) => [n.id, n]));
    ctx.lineWidth = 0.8 / view.k;
    ctx.strokeStyle = 'rgba(148,163,184,0.35)';
    for (const e of edges) {
      const a = pos.get(e.source);
      const b = pos.get(e.target);
      if (!a || !b) continue;
      ctx.beginPath();
      ctx.moveTo(a.x, a.y);
      ctx.lineTo(b.x, b.y);
      ctx.stroke();
    }
    for (const n of nodes) {
      const r = 3 + Math.min(10, Math.sqrt(n.degree) * 2);
      ctx.beginPath();
      ctx.arc(n.x, n.y, r, 0, Math.PI * 2);
      ctx.fillStyle = n.id === currentKey ? '#22d3ee' : n.path ? (n === hover ? '#e2e8f0' : '#94a3b8') : '#52525b';
      ctx.fill();
      if (n === hover || n.id === currentKey || nodes.length <= 60 || view.k > 1.6) {
        ctx.fillStyle = '#cbd5e1';
        ctx.font = `${11 / view.k}px ui-sans-serif, system-ui`;
        ctx.fillText(n.label, n.x + r + 2, n.y + 4 / view.k);
      }
    }
  }

  function toWorld(ev: MouseEvent) {
    const rect = canvas!.getBoundingClientRect();
    const s = Math.min(rect.width / W, rect.height / H) * view.k;
    return { x: (ev.clientX - rect.left - view.x) / s, y: (ev.clientY - rect.top - view.y) / s };
  }
  function hit(ev: MouseEvent): GraphNode | null {
    const p = toWorld(ev);
    let best: GraphNode | null = null;
    let bd = 14 / view.k;
    for (const n of nodes) {
      const d = Math.hypot(n.x - p.x, n.y - p.y);
      if (d < bd) {
        bd = d;
        best = n;
      }
    }
    return best;
  }
  function onMove(ev: MouseEvent) {
    if (drag) {
      view.x = drag.vx + (ev.clientX - drag.sx);
      view.y = drag.vy + (ev.clientY - drag.sy);
      draw();
      return;
    }
    const h = hit(ev);
    if (h !== hover) {
      hover = h;
      if (canvas) canvas.style.cursor = h ? 'pointer' : 'grab';
      draw();
    }
  }
  function onDown(ev: MouseEvent) {
    drag = { sx: ev.clientX, sy: ev.clientY, vx: view.x, vy: view.y };
  }
  function onUp(ev: MouseEvent) {
    const moved = drag && Math.hypot(ev.clientX - drag.sx, ev.clientY - drag.sy) > 4;
    drag = null;
    if (!moved) {
      const n = hit(ev);
      if (n?.path) onOpen(n.path);
    }
  }
  function onWheel(ev: WheelEvent) {
    ev.preventDefault();
    const rect = canvas!.getBoundingClientRect();
    const mx = ev.clientX - rect.left;
    const my = ev.clientY - rect.top;
    const f = ev.deltaY < 0 ? 1.15 : 1 / 1.15;
    const k = Math.min(8, Math.max(0.2, view.k * f));
    const r = k / view.k;
    view.x = mx - (mx - view.x) * r;
    view.y = my - (my - view.y) * r;
    view.k = k;
    draw();
  }

  $effect(() => {
    // re-run when inputs change
    void index;
    void localOnly;
    void showUnresolved;
    if (open) tick().then(compute);
  });
</script>

{#if open}
  <div class="fixed inset-0 z-[60] bg-black/70 backdrop-blur-xs flex items-center justify-center p-6" role="presentation" onclick={onClose}>
    <div
      class="relative w-full h-full max-w-6xl rounded-xl border border-neutral-800 bg-neutral-950 overflow-hidden flex flex-col"
      role="dialog"
      aria-modal="true"
      aria-label="Graph View"
      tabindex="-1"
      onclick={(e) => e.stopPropagation()}
      onkeydown={(e) => e.key === 'Escape' && onClose()}
    >
      <header class="flex items-center gap-4 px-4 py-2 border-b border-neutral-800 text-xs text-neutral-300">
        <span class="font-semibold">Graph View</span>
        <span class="text-neutral-500">{nodes.length} catatan · {edges.length} link</span>
        <label class="flex items-center gap-1"><input type="checkbox" bind:checked={localOnly} /> Hanya catatan aktif</label>
        <label class="flex items-center gap-1"><input type="checkbox" bind:checked={showUnresolved} /> Link belum ada</label>
        <span class="ml-auto text-neutral-500">scroll = zoom · drag = geser · klik = buka</span>
        <button type="button" class="text-neutral-400 hover:text-white" onclick={onClose} aria-label="Tutup">✕</button>
      </header>
      <canvas
        bind:this={canvas}
        class="flex-1 w-full"
        style="cursor: grab"
        onmousemove={onMove}
        onmousedown={onDown}
        onmouseup={onUp}
        onmouseleave={() => (drag = null)}
        onwheel={onWheel}
      ></canvas>
    </div>
  </div>
{/if}
