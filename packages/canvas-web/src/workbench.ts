/**
 * Workbench completo do DB Modeling na Web: reproduz o shell (`crates/app/src/shell.rs`)
 * e o canvas (`crates/app/src/canvas.rs`) do app desktop. Toda a geometria (layout, rotas,
 * grupos) vem do núcleo Rust/Wasm (`compute_scene`); aqui só há DOM, eventos e estado de UI.
 */

import {
  Command,
  Diagnostic,
  HostTransport,
  Schema,
  SceneGeometry,
  SidecarLayout,
  TextPatch,
  WorkspaceFileEntry,
} from './types';
import {
  CARD_W,
  HEADER_H,
  ROW_H,
  GROUP_PAD,
  MIN_ZOOM,
  MAX_ZOOM,
  TABLE_COLORS,
  STANDARD_TYPES,
  cardH,
  getGlass,
  hexWithAlpha,
  Mode,
} from './glass';
import { iconSvg, IconName } from './icons';
import { ActionId, catalog } from './actions';
import { WORKBENCH_CSS } from './workbench-css';
import { exportToMermaid, exportToSvg, exportSvgStringToRasterBlob } from './export';

type Pt = { x: number; y: number };
type Pane = 'files' | 'schema' | 'problems';
type Drag =
  | { kind: 'pan'; lx: number; ly: number }
  | { kind: 'cards'; last: Pt }
  | { kind: 'group'; name: string; last: Pt }
  | { kind: 'marquee'; start: Pt; now: Pt; base: string[] }
  | { kind: 'link'; table: string; col: string; now: Pt };

const esc = (s: string): string =>
  String(s).replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
const ICON = (n: IconName, s = 16, c = 'currentColor'): string => iconSvg(n, s, c);
const IDENT = /^[\p{L}\p{N}_]+$/u;
const UI_STEPS = [1, 1.15, 1.3];

interface Prefs {
  mode: Mode;
  uiScale: number;
  translucent: boolean;
  autoHide: boolean;
  inspector: boolean;
  sidebar: boolean;
  legend: boolean;
  colTypeW: number;
}

function loadPrefs(): Prefs {
  const d: Prefs = {
    mode: 'dark',
    uiScale: 1,
    translucent: false,
    autoHide: false,
    inspector: true,
    sidebar: true,
    legend: false,
    colTypeW: 110,
  };
  try {
    return { ...d, ...JSON.parse(localStorage.getItem('dbmodeling.prefs') || '{}') };
  } catch {
    return d;
  }
}

export class Workbench {
  private root: HTMLElement;
  private wb!: HTMLElement;
  private transport: HostTransport;
  private wasm: any;

  private source = '';
  private title = 'exemplo.dbml';
  private files: WorkspaceFileEntry[] = [];
  private schema: Schema = { tables: [], refs: [], groups: [] };
  private lastValid: Schema | null = null;
  private diags: Diagnostic[] = [];
  private enums: string[] = [];
  private scene: SceneGeometry | null = null;
  private positions: Record<string, Pt> = {};
  private extraLayout: Partial<SidecarLayout> = {};

  private selection: string[] = [];
  private top: string | null = null;
  private tool: 'select' | 'hand' = 'select';
  private space = false;
  private zoom = 1;
  private pan: Pt = { x: 60, y: 60 };
  private drag: Drag | null = null;
  private needsFit = true;

  private prefs: Prefs = loadPrefs();
  private pane: Pane = 'schema';
  private editorOpen = false;
  private paletteOpen = false;
  private shortcutsOpen = false;
  private paletteQuery = '';
  private paletteIdx = 0;
  private focusCol: [string, string] | null = null;
  private pendingNew: string | null = null;
  private colResize: { x: number; w: number } | null = null;
  private saveTimer: number | null = null;
  private raf: number | null = null;

  private els: Record<string, HTMLElement> = {};

  constructor(root: HTMLElement, transport: HostTransport, wasm?: any) {
    this.root = root;
    this.transport = transport;
    this.wasm = wasm || (window as any).DbModelingParser;
    this.build();
    this.bind();
    transport.onDocumentUpdated((src, layout, meta) => this.setDocument(src, layout, meta));
    transport.onWorkspaceFilesUpdated?.((files, title) => {
      this.files = files;
      if (title) this.title = title;
      this.renderChrome();
    });
  }

  // ───────────────────────── construção do DOM ─────────────────────────

  private build(): void {
    const style = document.createElement('style');
    style.textContent = WORKBENCH_CSS;
    document.head.appendChild(style);
    this.root.innerHTML = '';
    this.wb = document.createElement('div');
    this.wb.className = 'wb';
    this.wb.innerHTML = `
      <div class="titlebar ui" id="titlebar"></div>
      <div class="main">
        <div class="rail left ui" id="rail-left"></div>
        <div class="sidebar ui" id="sidebar"></div>
        <div class="editor ui" id="editor">
          <div class="eh"><span id="ed-icon"></span><span class="t" id="ed-title"></span>
            <button class="ib" data-act="FormatDocument" title="Formatar documento (Alt+Shift+F)">${ICON('Sparkles', 14)}</button>
            <button class="ib" data-act="ToggleEditor" title="Fechar editor (Ctrl+E)">${ICON('X', 14)}</button></div>
          <div class="body"><div class="gutter" id="gutter"></div><textarea class="code" id="code" spellcheck="false"></textarea></div>
        </div>
        <div class="surface" id="surface" tabindex="0">
          <div class="world" id="world"></div>
          <div class="marquee" id="marquee"></div>
          <div class="empty" id="empty">
            <div style="color:var(--faint)">${ICON('Table2', 28)}</div>
            <h3>Nenhuma tabela ainda</h3>
            <p>Escreva o esquema em DBML no editor ou crie uma nova tabela.</p>
            <button class="btn" data-act="ToggleEditor">${ICON('Braces', 14)} Abrir editor DBML</button>
            <button class="btn primary" data-act="NewTable">${ICON('Plus', 14)} Nova tabela</button>
          </div>
          <div class="island tools ui" id="tools"></div>
          <div class="island zoomi ui" id="zoomi"></div>
          <div class="island helpi ui" id="helpi"><button class="ib" data-act="ToggleLegend" title="Legenda (Ctrl+Shift+L)">${ICON('Info', 16)}</button></div>
          <div class="legend ui" id="legend">
            <h4>Legenda</h4>
            <div><span style="color:var(--accent)">${ICON('KeyRound', 13)}</span> PK · chave primária</div>
            <div><span>${ICON('Link2', 13)}</span> Relação (coluna → coluna)</div>
            <div><b>N · 1</b> Cardinalidade</div>
            <div><span style="color:var(--accent)">■</span> Tabela selecionada e suas relações</div>
          </div>
        </div>
        <div class="inspector ui" id="inspector"></div>
        <div class="rail right ui" id="rail-right"></div>
      </div>
      <div class="status ui" id="status"></div>
      <div class="toast" id="toast"></div>`;
    this.root.appendChild(this.wb);
    for (const id of [
      'titlebar', 'rail-left', 'sidebar', 'editor', 'surface', 'world', 'marquee', 'empty', 'tools', 'zoomi',
      'legend', 'inspector', 'rail-right', 'status', 'toast', 'code', 'gutter', 'ed-title', 'ed-icon',
    ]) {
      this.els[id] = this.wb.querySelector('#' + id) as HTMLElement;
    }
    this.els['ed-icon'].innerHTML = ICON('Braces', 14);
    this.applyTheme();
  }

  private applyTheme(): void {
    const g = getGlass(this.prefs.mode, this.prefs.uiScale);
    const v: Record<string, string> = {
      chrome: g.chrome, sidebar: g.sidebar, surface: g.surface, grid: g.grid, card: g.card,
      'card-translucent': g.cardTranslucent, island: g.island, edge: g.edge, 'edge-soft': g.edgeSoft,
      hover: g.hover, active: g.active, selected: g.selected, text: g.text, muted: g.textMuted,
      faint: g.textFaint, accent: g.accent, danger: g.danger, warning: g.warning, ok: g.ok,
      'card-shadow': g.cardShadow, 'card-shadow-sel': g.cardShadowSelected, 'island-shadow': g.islandShadow,
      ui: String(this.prefs.uiScale),
    };
    for (const k of Object.keys(v)) this.wb.style.setProperty('--' + k, v[k]);
    this.wb.classList.toggle('translucent', this.prefs.translucent);
  }

  private savePrefs(): void {
    try {
      localStorage.setItem('dbmodeling.prefs', JSON.stringify(this.prefs));
    } catch {
      /* ignorado */
    }
  }

  // ───────────────────────── documento / análise ─────────────────────────

  private setDocument(
    source: string,
    layout?: SidecarLayout,
    meta?: { title?: string; files?: WorkspaceFileEntry[] }
  ): void {
    if (meta?.title) this.title = meta.title;
    if (meta?.files) this.files = meta.files;
    const first = this.source === '' && !this.scene;
    if (layout) {
      this.extraLayout = layout;
      if (layout.tables && (first || Object.keys(this.positions).length === 0)) {
        this.positions = { ...layout.tables } as Record<string, Pt>;
      }
      if (layout.viewport && first) {
        this.zoom = layout.viewport.zoom;
        this.pan = { x: layout.viewport.pan_x, y: layout.viewport.pan_y };
        this.needsFit = false;
      }
    }
    if (source === this.source && !first) {
      this.renderChrome();
      return;
    }
    this.source = source;
    this.syncEditorValue();
    this.reanalyze();
    this.renderAll();
  }

  private reanalyze(): void {
    if (!this.wasm) return;
    try {
      const r = this.wasm.parse_and_analyze(this.source);
      const hasError = (r.diagnostics as Diagnostic[]).some((d) => d.severity === 'error');
      this.diags = r.diagnostics;
      (this.transport as any).sendDiagnostics?.(this.diags);
      this.enums = r.enums || [];
      // mantém o último esquema válido durante a digitação para não piscar o Empty State
      if (hasError && r.schema.tables.length === 0 && this.lastValid) {
        this.schema = this.lastValid;
      } else {
        this.schema = r.schema;
        if (!hasError || r.schema.tables.length > 0) this.lastValid = r.schema;
      }
    } catch (e) {
      console.error('Erro ao analisar DBML via Wasm:', e);
    }
    this.selection = this.selection.filter((n) => this.schema.tables.some((t) => t.name === n));
    this.computeScene();
  }

  private computeScene(): void {
    if (!this.wasm) return;
    const known: Record<string, Pt> = {};
    for (const t of this.schema.tables) {
      if (this.positions[t.name]) known[t.name] = this.positions[t.name];
    }
    if (this.pendingNew && !known[this.pendingNew]) {
      const c = this.viewCenter();
      known[this.pendingNew] = { x: Math.round(c.x - CARD_W / 2), y: Math.round(c.y - 60) };
      this.positions[this.pendingNew] = known[this.pendingNew];
    }
    try {
      this.scene = this.wasm.compute_scene(this.source, known);
      for (const k of Object.keys(this.scene!.positions)) {
        if (!this.positions[k]) this.positions[k] = this.scene!.positions[k];
      }
    } catch (e) {
      console.error('Erro em compute_scene:', e);
    }
    this.pendingNew = null;
  }

  private viewCenter(): Pt {
    const r = this.els['surface'].getBoundingClientRect();
    return { x: (r.width / 2 - this.pan.x) / this.zoom, y: (r.height / 2 - this.pan.y) / this.zoom };
  }

  // ───────────────────────── comandos / patches ─────────────────────────

  private fullReplace(updated: string): TextPatch {
    const old = this.source;
    const lines = old.split('\n');
    return {
      range: {
        start: { line: 0, character: 0 },
        end: { line: lines.length - 1, character: lines[lines.length - 1].length },
      },
      byte_start: 0,
      byte_end: new TextEncoder().encode(old).length,
      new_text: updated,
      updated_source: updated,
    };
  }

  private applyPatch(patch: TextPatch): void {
    if (!patch.new_text && patch.byte_start === patch.byte_end) return;
    this.source = patch.updated_source ?? this.source;
    this.transport.sendPatch(patch);
    this.syncEditorValue();
    this.reanalyze();
  }

  private dispatch(cmd: Command): TextPatch | null {
    if (!this.wasm) return null;
    try {
      const patch: TextPatch = this.wasm.apply_command(this.source, cmd);
      if (patch.renamed_table) {
        const [o, n] = patch.renamed_table;
        if (this.positions[o]) {
          this.positions[n] = this.positions[o];
          delete this.positions[o];
        }
        this.selection = this.selection.map((s) => (s === o ? n : s));
        if (this.top === o) this.top = n;
      }
      if (patch.created_table) {
        this.pendingNew = patch.created_table;
        this.selection = [patch.created_table];
        this.prefs.inspector = true;
        this.focusCol = null;
      }
      if (patch.created_column) {
        this.selection = [patch.created_column[0]];
        this.focusCol = patch.created_column;
        this.prefs.inspector = true;
      }
      this.applyPatch(patch);
      if (patch.should_relayout) this.relayout(false);
      this.scheduleRender();
      return patch;
    } catch (e: any) {
      this.toast(String(e?.message || e));
      return null;
    }
  }

  private relayout(fit = true): void {
    if (!this.wasm) return;
    try {
      const r = this.wasm.compute_layout(this.source, {});
      this.positions = { ...r.positions };
      this.computeScene();
      if (fit) this.fit();
      this.queueSaveLayout();
      this.scheduleRender();
    } catch (e) {
      console.error('Erro no auto-layout:', e);
    }
  }

  private queueSaveLayout(): void {
    if (this.saveTimer) window.clearTimeout(this.saveTimer);
    this.saveTimer = window.setTimeout(() => {
      const layout: SidecarLayout = {
        ...(this.extraLayout as SidecarLayout),
        tables: { ...this.positions },
        viewport: { zoom: this.zoom, pan_x: this.pan.x, pan_y: this.pan.y },
      };
      this.transport.saveLayout(layout).catch(console.error);
    }, 300);
  }

  // ───────────────────────── câmera ─────────────────────────

  private fit(): void {
    const r = this.els['surface'].getBoundingClientRect();
    if (!this.scene || r.width === 0 || this.schema.tables.length === 0) return;
    let x0 = Infinity, y0 = Infinity, x1 = -Infinity, y1 = -Infinity;
    for (const t of this.scene.tables) {
      const p = this.positions[t.name] || t;
      x0 = Math.min(x0, p.x); y0 = Math.min(y0, p.y);
      x1 = Math.max(x1, p.x + CARD_W); y1 = Math.max(y1, p.y + t.h);
    }
    for (const g of this.scene.groups) {
      x0 = Math.min(x0, g.x); y0 = Math.min(y0, g.y);
      x1 = Math.max(x1, g.x + g.w); y1 = Math.max(y1, g.y + g.h);
    }
    const w = x1 - x0, h = y1 - y0;
    const z = Math.min(Math.max(Math.min((r.width - 120) / w, (r.height - 140) / h, 1.2), MIN_ZOOM), MAX_ZOOM);
    this.zoom = z;
    this.pan = { x: (r.width - w * z) / 2 - x0 * z, y: (r.height - h * z) / 2 - y0 * z + 14 };
    this.needsFit = false;
  }

  private zoomAt(cx: number, cy: number, factor: number): void {
    const z = Math.min(Math.max(this.zoom * factor, MIN_ZOOM), MAX_ZOOM);
    const k = z / this.zoom;
    this.pan = { x: cx - (cx - this.pan.x) * k, y: cy - (cy - this.pan.y) * k };
    this.zoom = z;
  }

  private toWorld(e: { clientX: number; clientY: number }): Pt {
    const r = this.els['surface'].getBoundingClientRect();
    return { x: (e.clientX - r.left - this.pan.x) / this.zoom, y: (e.clientY - r.top - this.pan.y) / this.zoom };
  }

  private focusTable(name: string): void {
    const p = this.positions[name];
    const t = this.schema.tables.find((x) => x.name === name);
    if (!p || !t) return;
    const r = this.els['surface'].getBoundingClientRect();
    this.selection = [name];
    this.top = name;
    this.pan = {
      x: r.width / 2 - (p.x + CARD_W / 2) * this.zoom,
      y: r.height / 2 - (p.y + cardH(t.columns.length) / 2) * this.zoom,
    };
    this.prefs.inspector = true;
    this.savePrefs();
    this.scheduleRender();
  }

  // ───────────────────────── renderização ─────────────────────────

  private scheduleRender(): void {
    if (this.raf !== null) return;
    this.raf = requestAnimationFrame(() => {
      this.raf = null;
      this.renderAll();
    });
  }

  private renderAll(): void {
    if (this.needsFit && this.scene) {
      this.fit();
      if (this.els['surface'].getBoundingClientRect().width > 0) this.needsFit = false;
    }
    this.renderWorld();
    this.renderChrome();
  }

  private renderChrome(): void {
    this.applyTheme();
    this.renderTitlebar();
    this.renderRails();
    this.renderSidebar();
    this.renderInspector();
    this.renderIslands();
    this.renderStatus();
    this.els['editor'].style.display = this.editorOpen ? 'flex' : 'none';
    this.els['ed-title'].textContent = this.title;
    const sb = this.els['sidebar'];
    sb.style.display = this.prefs.sidebar ? 'block' : 'none';
    this.els['legend'].style.display = this.prefs.legend ? 'block' : 'none';
    this.els['empty'].style.display = this.schema.tables.length === 0 ? 'block' : 'none';
    this.updateGutter();
  }

  private tint(name: string): string | null {
    return this.scene?.tables.find((t) => t.name === name)?.tint || null;
  }

  private renderWorld(): void {
    const sc = this.scene;
    const w = this.els['world'];
    w.style.transform = `translate(${this.pan.x}px, ${this.pan.y}px) scale(${this.zoom})`;
    // grade de pontos adaptativa
    const g = getGlass(this.prefs.mode);
    let step = 24 * this.zoom;
    while (step < 20) step *= 2;
    const s = this.els['surface'];
    s.style.backgroundImage = `radial-gradient(circle, ${g.grid} 0.9px, transparent 1.2px)`;
    s.style.backgroundSize = `${step}px ${step}px`;
    s.style.backgroundPosition = `${this.pan.x}px ${this.pan.y}px`;
    s.style.cursor =
      this.drag?.kind === 'pan' ? 'grabbing' : this.tool === 'hand' || this.space ? 'grab' : 'default';
    if (!sc) {
      w.innerHTML = '';
      return;
    }
    const sel = new Set(this.selection);
    const hasSel = sel.size > 0;
    let svg = '';
    for (const e of sc.edges) {
      const lit = sel.has(e.from_table) || sel.has(e.to_table);
      const col = lit ? g.accent : g.textMuted;
      const op = lit ? 1 : hasSel ? 0.2 : 0.55;
      svg += `<path d="${e.svg_path}" fill="none" stroke="${col}" stroke-opacity="${op}" stroke-width="${lit ? 2 : 1.5}"/>`;
      svg += `<circle cx="${e.ax}" cy="${e.ay}" r="3" fill="${col}" fill-opacity="${op}"/><circle cx="${e.bx}" cy="${e.by}" r="3" fill="${col}" fill-opacity="${op}"/>`;
      svg += `<text x="${e.start_label_pos.x}" y="${e.start_label_pos.y + 10}" font-size="10" fill="${g.textMuted}" fill-opacity="${op}">${e.start_label}</text>`;
      svg += `<text x="${e.end_label_pos.x}" y="${e.end_label_pos.y + 10}" font-size="10" fill="${g.textMuted}" fill-opacity="${op}">${e.end_label}</text>`;
    }
    if (this.drag?.kind === 'link') {
      const d = this.drag;
      const p = this.positions[d.table];
      const t = this.schema.tables.find((x) => x.name === d.table);
      const i = t ? t.columns.findIndex((c) => c.name === d.col) : -1;
      if (p && i >= 0) {
        const ay = p.y + HEADER_H + 4 + ROW_H * (i + 0.5);
        svg += `<path d="M ${p.x + CARD_W} ${ay} L ${d.now.x} ${d.now.y}" stroke="${g.accent}" stroke-width="2" stroke-dasharray="6 4" fill="none"/><circle cx="${d.now.x}" cy="${d.now.y}" r="5" fill="${g.accent}"/>`;
      }
    }
    let html = `<svg class="edges">${svg}</svg>`;
    // molduras de grupo
    for (const gr of sc.groups) {
      const c = gr.color;
      // posição segue as tabelas (que podem estar sendo arrastadas)
      const mem = gr.tables.map((n) => ({ n, p: this.positions[n], t: this.schema.tables.find((x) => x.name === n) }))
        .filter((m) => m.p && m.t);
      if (!mem.length) continue;
      let x0 = Infinity, y0 = Infinity, x1 = -Infinity, y1 = -Infinity;
      for (const m of mem) {
        x0 = Math.min(x0, m.p.x); y0 = Math.min(y0, m.p.y);
        x1 = Math.max(x1, m.p.x + CARD_W); y1 = Math.max(y1, m.p.y + cardH(m.t!.columns.length));
      }
      const pad = GROUP_PAD - 8;
      html += `<div class="group" style="left:${x0 - pad}px;top:${y0 - pad - 8}px;width:${x1 - x0 + 2 * pad}px;height:${y1 - y0 + 2 * pad + 8}px;border-color:${hexWithAlpha(c, 0.4)};background:${hexWithAlpha(c, 0.06)}">
        <div class="gl" data-group="${esc(gr.name)}" style="background:${hexWithAlpha(c, 0.16)};color:${c}"><i style="background:${c}"></i>${esc(gr.name)}<b>${mem.length}</b></div></div>`;
    }
    // cartões
    const order = [...sc.tables].sort((a, b) => Number(a.name === this.top) - Number(b.name === this.top));
    for (const tv of order) {
      const t = this.schema.tables.find((x) => x.name === tv.name);
      const p = this.positions[tv.name];
      if (!t || !p) continue;
      const tint = tv.tint;
      const fk = new Set(tv.fk_columns);
      const rows = t.columns
        .map((c) => {
          const ic = c.pk
            ? `<span style="color:var(--accent)">${ICON('KeyRound', 12)}</span>`
            : fk.has(c.name)
              ? `<span style="color:var(--muted)">${ICON('Link2', 12)}</span>`
              : '';
          return `<div class="row" data-col="${esc(c.name)}" data-table="${esc(t.name)}"><div class="rail14">${ic}</div>
            <div class="cn ${c.pk ? 'pk' : ''}">${esc(c.name)}</div><div class="ty">${esc(c.ty)}</div>${c.not_null ? '<div class="nn">NN</div>' : ''}
            <div class="port" data-port="1" data-table="${esc(t.name)}" data-col="${esc(c.name)}"></div></div>`;
        })
        .join('');
      html += `<div class="card ${sel.has(t.name) ? 'sel' : ''}" data-table="${esc(t.name)}" style="left:${p.x}px;top:${p.y}px;height:${cardH(t.columns.length)}px">
        ${tint ? `<div class="bar" style="background:${tint}"></div>` : ''}
        <div class="head" style="${tint ? `background:${hexWithAlpha(tint, 0.08)}` : ''}"><div class="ic" style="${tint ? `color:${tint}` : ''}">${ICON('Table2', 14)}</div>
        <div class="nm">${esc(t.name)}</div><div class="cnt">${t.columns.length}</div></div>
        <div class="rows">${rows}</div>
        <div class="plus" data-addcol="${esc(t.name)}" title="Adicionar coluna">${ICON('Plus', 14)}</div></div>`;
    }
    w.innerHTML = html;
    // marquee
    const m = this.els['marquee'];
    if (this.drag?.kind === 'marquee') {
      const d = this.drag;
      const a = this.worldToScreen(d.start), b = this.worldToScreen(d.now);
      m.style.display = 'block';
      m.style.left = Math.min(a.x, b.x) + 'px';
      m.style.top = Math.min(a.y, b.y) + 'px';
      m.style.width = Math.abs(a.x - b.x) + 'px';
      m.style.height = Math.abs(a.y - b.y) + 'px';
    } else {
      m.style.display = 'none';
    }
  }

  private worldToScreen(p: Pt): Pt {
    return { x: p.x * this.zoom + this.pan.x, y: p.y * this.zoom + this.pan.y };
  }

  private btn(act: string, icon: IconName, tip: string, on = false, extra = ''): string {
    return `<button class="ib ${on ? 'on' : ''}" data-act="${act}" title="${esc(tip)}" ${extra}>${ICON(icon, 16)}</button>`;
  }

  private renderTitlebar(): void {
    this.els['titlebar'].innerHTML = `
      <span style="color:var(--accent)">${ICON('Database', 16)}</span>
      <div class="doc">${ICON('Braces', 13)}<span>${esc(this.title)}</span></div>
      ${this.btn('NewFile', 'Plus', 'Novo arquivo .dbml (Ctrl+N)')}
      <div class="spacer"></div>
      <button class="pill" data-act="OpenPalette">${ICON('Search', 14)}<span>DB Modeling — Ctrl+K comandos</span></button>
      <div class="spacer"></div>
      ${this.btn('ToggleTheme', this.prefs.mode === 'dark' ? 'Moon' : 'Sun', 'Alternar tema (Ctrl+T)')}`;
  }

  private errorCount(): number {
    return this.diags.filter((d) => d.severity === 'error').length;
  }

  private renderRails(): void {
    const errs = this.errorCount();
    const sb = this.prefs.sidebar;
    this.els['rail-left'].innerHTML = `
      ${this.btn('ShowFiles', 'Folder', 'Arquivos (Ctrl+Shift+E)', sb && this.pane === 'files')}
      ${this.btn('ShowSchema', 'Table2', 'Esquema (Ctrl+Shift+D)', sb && this.pane === 'schema')}
      <button class="ib ${sb && this.pane === 'problems' ? 'on' : ''}" data-act="ShowProblems" title="Problemas (Ctrl+Shift+M)">${ICON('TriangleAlert', 16)}${errs ? `<span class="badge">${errs}</span>` : ''}</button>
      ${this.btn('ToggleEditor', 'Braces', 'Editor DBML (Ctrl+E)', this.editorOpen)}
      <div class="grow"></div>
      <button class="ib" data-act="menu:appearance" title="Aparência">${ICON('Palette', 16)}</button>
      ${this.btn('ShowShortcuts', 'Keyboard', 'Atalhos de teclado (Ctrl+/)')}
      ${this.btn('OpenPalette', 'Command', 'Paleta de comandos (Ctrl+K)')}`;
    this.els['rail-right'].innerHTML = `
      <button class="ib ${this.prefs.inspector ? 'on' : ''}" data-act="ToggleInspector" title="Inspector (Ctrl+I)">${ICON('SlidersHorizontal', 16)}${this.selection.length ? '<span class="badge">1</span>' : ''}</button>
      ${this.btn('ToggleLegend', 'Info', 'Legenda (Ctrl+Shift+L)', this.prefs.legend)}`;
  }

  private renderIslands(): void {
    this.els['tools'].innerHTML = `
      ${this.btn('ToolSelect', 'MousePointer2', 'Selecionar (V)', this.tool === 'select')}
      ${this.btn('ToolHand', 'Hand', 'Mão: mover a vista (H)', this.tool === 'hand')}
      <div class="sep"></div>
      ${this.btn('NewTable', 'Plus', 'Nova tabela (T)')}
      <div class="sep"></div>
      ${this.btn('Relayout', 'LayoutGrid', 'Reorganizar automaticamente (Ctrl+L)')}
      ${this.btn('FitView', 'Scan', 'Ajustar à tela (Shift+1)')}
      <div class="sep"></div>
      ${this.btn('ToggleEditor', 'Braces', 'Editor DBML (Ctrl+E)', this.editorOpen)}
      <button class="ib" data-act="menu:export" title="Exportar">${ICON('Download', 16)}</button>`;
    this.els['zoomi'].innerHTML = `${this.btn('ZoomOut', 'ZoomOut', 'Reduzir (Ctrl+-)')}
      <button class="pct" data-act="FitView" title="Ajustar à tela">${Math.round(this.zoom * 100)}%</button>
      ${this.btn('ZoomIn', 'ZoomIn', 'Ampliar (Ctrl+=)')}`;
  }

  private hint(): string {
    const d = this.drag;
    if (d?.kind === 'cards') return `Movendo ${this.selection.join(', ')}`;
    if (d?.kind === 'group') return `Movendo o grupo ${d.name}`;
    if (d?.kind === 'link') return `Relação a partir de ${d.table}.${d.col}: solte sobre a coluna de destino`;
    if (d?.kind === 'marquee') return 'Solte para selecionar as tabelas dentro do retângulo';
    if (this.selection.length > 1) return `${this.selection.length} tabelas selecionadas · Ctrl+G agrupa`;
    if (this.selection.length === 1) return `${this.selection[0]} · F2 renomeia · Del exclui`;
    return 'Clique numa tabela para ver detalhes · arraste o fundo para selecionar · roda para mover · Ctrl+roda para zoom';
  }

  private renderStatus(): void {
    const errs = this.errorCount();
    const warns = this.diags.filter((d) => d.severity === 'warning').length;
    this.els['status'].innerHTML = `
      <button data-act="ShowProblems"><span style="color:${errs ? 'var(--danger)' : 'var(--faint)'}">${ICON('CircleX', 12)}</span> ${errs}
        <span style="color:${warns ? 'var(--warning)' : 'var(--faint)'}">${ICON('TriangleAlert', 12)}</span> ${warns}</button>
      <span>${this.schema.tables.length} tabelas · ${this.schema.refs.length} relações</span>
      <span class="hint">${esc(this.hint())}</span>
      <span>Zoom ${Math.round(this.zoom * 100)}%</span><span>DBML</span><span>${this.prefs.mode === 'dark' ? 'Escuro' : 'Claro'}</span>`;
  }

  private renderSidebar(): void {
    const el = this.els['sidebar'];
    if (this.pane === 'files') {
      const list = this.files.length
        ? this.files
            .map(
              (f) => `<div class="srow ${f.active ? 'on' : ''}" data-act="open-file" data-path="${esc(f.path)}">
          <span style="color:${f.active ? 'var(--accent)' : 'var(--faint)'}">${ICON('Database', 14)}</span><span class="g">${esc(f.name)}</span><span class="c">DBML</span></div>`
            )
            .join('')
        : `<div class="pad">Nenhum arquivo .dbml encontrado no workspace.</div>`;
      el.innerHTML = `<div class="sec"><span>ARQUIVOS</span><span>${this.files.length}</span></div>${list}
        <div style="padding:12px"><button class="btn primary" data-act="NewFile">${ICON('Plus', 14)} Novo arquivo</button></div>`;
    } else if (this.pane === 'schema') {
      const row = (name: string, cnt: number, pad = 0): string => {
        const t = this.tint(name);
        return `<div class="srow ${this.selection.includes(name) ? 'on' : ''}" style="padding-left:${8 + pad}px" data-act="focus-table" data-table="${esc(name)}">
          <span style="color:${t || 'var(--faint)'}">${ICON('Table2', 14)}</span><span class="g">${esc(name)}</span><span class="c">${cnt}</span></div>`;
      };
      let body = '';
      const grouped = new Set<string>();
      for (const gr of this.scene?.groups || []) {
        body += `<div class="gh"><i style="background:${gr.color}"></i><span style="flex:1">${esc(gr.name)}</span><span style="font-weight:400;color:var(--faint)">${gr.tables.length}</span></div>`;
        for (const n of gr.tables) {
          const t = this.schema.tables.find((x) => x.name === n);
          if (t) { body += row(n, t.columns.length, 14); grouped.add(n); }
        }
      }
      const loose = this.schema.tables.filter((t) => !grouped.has(t.name));
      if (grouped.size && loose.length) body += `<div class="gh" style="color:var(--faint)">Sem grupo</div>`;
      for (const t of loose) body += row(t.name, t.columns.length);
      const refs = this.schema.refs
        .map((r) => `<div class="rel"><span style="color:var(--faint)">${ICON('Link2', 12)}</span><span class="tx">${esc(r.from.table)} → ${esc(r.to.table)}</span></div>`)
        .join('');
      el.innerHTML = `<div class="sec"><span>TABELAS</span><span>${this.schema.tables.length}</span></div>${body}
        <div class="sec"><span>RELAÇÕES</span><span>${this.schema.refs.length}</span></div>${refs}`;
    } else {
      const items = this.diags
        .map((d, i) => {
          const [ic, col] = d.severity === 'error' ? (['CircleX', 'var(--danger)'] as const)
            : d.severity === 'warning' ? (['TriangleAlert', 'var(--warning)'] as const)
            : (['Info', 'var(--faint)'] as const);
          return `<div class="prob" data-act="goto-diag" data-i="${i}"><span style="color:${col};padding-top:2px">${ICON(ic as IconName, 13)}</span>
            <div><div>${esc(d.message)}</div><div class="sub">${esc(this.title)} · linha ${d.range.start.line + 1}</div></div></div>`;
        })
        .join('');
      el.innerHTML = `<div class="sec"><span>PROBLEMAS</span><span>${this.diags.length}</span></div>${
        items || `<div class="pad" style="align-items:center"><span style="color:var(--ok)">${ICON('CircleCheck', 20)}</span>Nenhum problema encontrado.</div>`}`;
    }
  }

  private renderInspector(): void {
    const el = this.els['inspector'];
    const sel = this.selection;
    const show = this.prefs.inspector && !(this.prefs.autoHide && sel.length === 0);
    el.style.display = show ? 'block' : 'none';
    if (!show) return;
    // não recria o inspector enquanto o usuário digita num campo dele
    const ae = document.activeElement as HTMLElement | null;
    if (ae && el.contains(ae) && ae.tagName === 'INPUT' && !this.focusCol) return;

    if (sel.length > 1) {
      el.innerHTML = `<div class="ihead"><span class="eb">SELEÇÃO</span><span class="ti">${sel.length} tabelas</span><span class="su">Shift+clique adiciona ou remove</span></div>
        <div class="sec"><span>TABELAS</span></div>
        ${sel.map((n) => `<div class="rel"><span style="color:var(--faint)">${ICON('Table2', 12)}</span><span class="tx">${esc(n)}</span></div>`).join('')}
        <div style="padding:12px"><button class="btn primary" style="width:100%;justify-content:center" data-act="GroupSelection">${ICON('LayoutGrid', 14)} Agrupar seleção</button></div>`;
      return;
    }
    const t = sel.length === 1 ? this.schema.tables.find((x) => x.name === sel[0]) : undefined;
    if (!t) {
      const kb = (l: string, k: string) => `<div class="kb"><span>${l}</span><kbd>${k}</kbd></div>`;
      el.innerHTML = `<div class="pad"><div style="display:flex;justify-content:space-between;align-items:center">
        <b style="font-size:14.5px;color:var(--text)">Nada selecionado</b>
        <button class="ib" data-act="ToggleInspector" title="Ocultar inspector (Ctrl+I)">${ICON('PanelRightClose', 14)}</button></div>
        <div>Clique numa tabela para ver colunas, cor, grupo e relações. Shift+clique seleciona várias.</div>
        <div style="height:1px;background:var(--edge);margin:6px 0"></div>
        ${kb('Selecionar', 'V')}${kb('Mover a vista', 'H')}${kb('Ajustar à tela', 'Shift+1')}${kb('Agrupar seleção', 'Ctrl+G')}${kb('Editor DBML', 'Ctrl+E')}${kb('Todos os comandos', 'Ctrl+K')}</div>`;
      return;
    }
    const grp = this.schema.groups.find((g) => g.tables.includes(t.name));
    const refs = this.schema.refs.filter((r) => r.from.table === t.name || r.to.table === t.name);
    const cur = (t.color || '').toLowerCase();
    const sw = TABLE_COLORS.map(([l, h]) => `<button class="${cur === h ? 'on' : ''}" title="${l}" style="background:${h}" data-act="set-color" data-table="${esc(t.name)}" data-color="${h}"></button>`).join('') +
      `<button class="none" title="Sem cor" data-act="set-color" data-table="${esc(t.name)}" data-color="">${ICON('X', 12)}</button>`;
    const tw = this.prefs.colTypeW;
    const cols = t.columns.map((c) => `<div class="crow">
        <div class="n"><input class="fld" data-edit="colname" data-table="${esc(t.name)}" data-col="${esc(c.name)}" data-orig="${esc(c.name)}" value="${esc(c.name)}"></div>
        <div class="split" data-split="1"></div>
        <div class="t" style="width:${tw}px"><input class="fld muted" data-edit="coltype" data-table="${esc(t.name)}" data-col="${esc(c.name)}" data-orig="${esc(c.ty)}" value="${esc(c.ty)}"></div>
        ${[['PK', 'pk', c.pk], ['NN', 'not null', c.not_null], ['UQ', 'unique', c.unique]]
          .map(([l, f, on]) => `<button class="chip ${on ? 'on' : ''}" data-act="toggle-flag" data-table="${esc(t.name)}" data-col="${esc(c.name)}" data-flag="${f}" data-on="${on ? 1 : 0}">${l}</button>`).join('')}
        <button class="del" data-act="del-col" data-table="${esc(t.name)}" data-col="${esc(c.name)}" title="Excluir coluna">${ICON('Trash', 13)}</button></div>`).join('');
    const rels = refs.map((r) => {
      const card = { ManyToOne: 'N:1', OneToMany: '1:N', OneToOne: '1:1', ManyToMany: 'N:N' }[r.kind];
      return `<div class="rel"><span style="color:var(--faint)">${ICON('Link2', 12)}</span><span class="tx">${esc(r.from.table)}.${esc(r.from.column)} → ${esc(r.to.table)}.${esc(r.to.column)}</span><span class="tag">${card}</span></div>`;
    }).join('');
    el.innerHTML = `
      <div class="ihead"><span class="eb">TABELA</span>
        <input class="fld big" id="insp-name" data-edit="table" data-table="${esc(t.name)}" data-orig="${esc(t.name)}" value="${esc(t.name)}">
        <span class="su">${t.columns.length} colunas · F2 renomeia</span></div>
      <div class="sec"><span>COR</span></div><div class="sw">${sw}</div>
      <div class="sec"><span>GRUPO</span></div>
      ${grp ? `<div class="rel"><span style="color:var(--faint)">${ICON('LayoutGrid', 12)}</span><span class="tx" style="color:var(--text)">${esc(grp.name)}</span><button class="btn ghost" style="height:22px" data-act="UngroupSelection">Desagrupar</button></div>`
            : `<div class="rel" style="color:var(--faint)">Sem grupo · Shift+clique em outras tabelas e Ctrl+G para agrupar</div>`}
      <div class="sec"><span>COLUNAS</span></div>
      <div class="crow hd"><div class="n">NOME</div><div class="split" data-split="1" title="Arraste para redimensionar (duplo-clique restaura)"></div>
        <div class="t" style="width:${tw}px">TIPO</div><div style="width:62px">FLAGS</div><div style="width:20px"></div></div>
      ${cols}
      <div style="padding:8px 12px"><button class="btn ghost" data-act="add-col" data-table="${esc(t.name)}">${ICON('Plus', 14)} Adicionar coluna</button></div>
      <div class="sec"><span>RELAÇÕES</span></div>${rels}
      <div style="padding:12px;margin-top:8px;border-top:1px solid var(--edge-soft)"><button class="btn ghost" data-act="DeleteSelection">${ICON('Trash', 14)} Excluir tabela</button></div>`;
    if (this.focusCol) {
      const [ft, fc] = this.focusCol;
      this.focusCol = null;
      const inp = el.querySelector(`[data-edit="colname"][data-table="${CSS.escape(ft)}"][data-col="${CSS.escape(fc)}"]`) as HTMLInputElement | null;
      inp?.focus();
      inp?.select();
    }
  }

  // ───────────────────────── editor DBML ─────────────────────────

  private syncEditorValue(): void {
    const ta = this.els['code'] as HTMLTextAreaElement;
    if (ta.value !== this.source) {
      const s = ta.selectionStart, e = ta.selectionEnd, top = ta.scrollTop;
      ta.value = this.source;
      ta.setSelectionRange(Math.min(s, this.source.length), Math.min(e, this.source.length));
      ta.scrollTop = top;
    }
  }

  private updateGutter(): void {
    if (!this.editorOpen) return;
    const n = this.source.split('\n').length;
    let s = '';
    for (let i = 1; i <= n; i++) s += i + '\n';
    this.els['gutter'].textContent = s;
    this.els['gutter'].scrollTop = this.els['code'].scrollTop;
  }

  private offsetOf(line: number, ch: number): number {
    const lines = this.source.split('\n');
    let off = 0;
    for (let i = 0; i < line && i < lines.length; i++) off += lines[i].length + 1;
    return off + ch;
  }

  private moveLine(up: boolean): void {
    const ta = this.els['code'] as HTMLTextAreaElement;
    const line = ta.value.slice(0, ta.selectionStart).split('\n').length - 1;
    try {
      const patch: TextPatch = this.wasm.swap_dbml_lines(this.source, line, up);
      if (patch.updated_source && patch.updated_source !== this.source) {
        this.applyPatch(patch);
        ta.value = this.source;
        const nl = up ? line - 1 : line + 1;
        const p = this.offsetOf(nl, 0);
        ta.setSelectionRange(p, p);
        this.scheduleRender();
      }
    } catch (e) {
      console.error(e);
    }
  }

  // ───────────────────────── ações ─────────────────────────

  private runAction(id: ActionId): void {
    switch (id) {
      case 'NewFile': this.transport.createNewFile?.(); return;
      case 'ShowFiles': this.showPane('files'); return;
      case 'ShowSchema': this.showPane('schema'); return;
      case 'ShowProblems': this.showPane('problems'); return;
      case 'ToggleTheme': this.prefs.mode = this.prefs.mode === 'dark' ? 'light' : 'dark'; break;
      case 'ToggleEditor':
        this.editorOpen = !this.editorOpen;
        if (this.editorOpen) setTimeout(() => this.els['code'].focus(), 0);
        break;
      case 'ToggleSidebar': this.prefs.sidebar = !this.prefs.sidebar; break;
      case 'ToggleInspector': this.prefs.inspector = !this.prefs.inspector; break;
      case 'ToggleLegend': this.prefs.legend = !this.prefs.legend; break;
      case 'FitView': this.fit(); break;
      case 'Relayout': this.relayout(true); return;
      case 'ZoomIn': case 'ZoomOut': {
        const r = this.els['surface'].getBoundingClientRect();
        this.zoomAt(r.width / 2, r.height / 2, id === 'ZoomIn' ? 1.2 : 1 / 1.2);
        break;
      }
      case 'ToolSelect': this.tool = 'select'; break;
      case 'ToolHand': this.tool = 'hand'; break;
      case 'Deselect': this.selection = []; break;
      case 'ToggleCardTransparency': this.prefs.translucent = !this.prefs.translucent; break;
      case 'GroupSelection':
        if (this.selection.length) this.dispatch({ action: 'group_tables', payload: { tables: [...this.selection] } });
        return;
      case 'UngroupSelection':
        if (this.selection.length) this.dispatch({ action: 'ungroup_tables', payload: { tables: [...this.selection] } });
        return;
      case 'NewTable': this.dispatch({ action: 'add_table', payload: {} }); return;
      case 'DeleteSelection':
        if (this.selection.length) {
          this.dispatch({ action: 'delete_tables', payload: { names: [...this.selection] } });
          this.selection = [];
        }
        break;
      case 'RenameSelected': {
        if (this.selection.length !== 1) return;
        this.prefs.inspector = true;
        this.renderChrome();
        const i = this.els['inspector'].querySelector('#insp-name') as HTMLInputElement | null;
        i?.focus(); i?.select();
        return;
      }
      case 'FormatDocument': {
        const f = this.wasm.format_dbml(this.source);
        if (f !== this.source) this.applyPatch(this.fullReplace(f));
        break;
      }
      case 'MoveLineUp': this.moveLine(true); return;
      case 'MoveLineDown': this.moveLine(false); return;
      case 'ZoomInUi': this.stepUi(1); break;
      case 'ZoomOutUi': this.stepUi(-1); break;
      case 'ResetUiZoom': this.prefs.uiScale = 1; break;
      case 'ToggleAutoCollapseInspector': this.prefs.autoHide = !this.prefs.autoHide; break;
      case 'OpenPalette': this.openPalette(); return;
      case 'ShowShortcuts': this.openShortcuts(); return;
      case 'ExportSvg': this.doExport('svg'); return;
      case 'ExportPng': this.doExport('png'); return;
      case 'ExportJpeg': this.doExport('jpeg'); return;
      case 'ExportMermaid': this.doExport('mermaid'); return;
    }
    this.savePrefs();
    this.scheduleRender();
  }

  private stepUi(d: number): void {
    let i = UI_STEPS.findIndex((s) => Math.abs(s - this.prefs.uiScale) < 0.05);
    if (i < 0) i = 0;
    this.prefs.uiScale = UI_STEPS[Math.min(Math.max(i + d, 0), UI_STEPS.length - 1)];
  }

  private showPane(p: Pane): void {
    this.pane = p;
    this.prefs.sidebar = true;
    this.savePrefs();
    this.renderChrome();
  }

  private async doExport(kind: 'svg' | 'png' | 'jpeg' | 'mermaid'): Promise<void> {
    if (!this.transport.exportFile) return;
    try {
      if (kind === 'mermaid') {
        await this.transport.exportFile('diagrama.mmd', exportToMermaid(this.schema), false);
        return;
      }
      const svg = exportToSvg(this.schema, this.positions, this.scene, this.prefs.mode);
      if (kind === 'svg') {
        await this.transport.exportFile('diagrama.svg', svg, false);
        return;
      }
      const blob = await exportSvgStringToRasterBlob(svg, kind === 'png' ? 'image/png' : 'image/jpeg');
      const b64: string = await new Promise((res, rej) => {
        const fr = new FileReader();
        fr.onload = () => res(String(fr.result).split(',')[1] || '');
        fr.onerror = rej;
        fr.readAsDataURL(blob);
      });
      await this.transport.exportFile(kind === 'png' ? 'diagrama.png' : 'diagrama.jpg', b64, true);
    } catch (e) {
      this.toast('Falha ao exportar: ' + e);
    }
  }

  private toast(msg: string): void {
    const t = this.els['toast'];
    t.textContent = msg;
    t.style.display = 'block';
    window.setTimeout(() => (t.style.display = 'none'), 3200);
  }

  // ───────────────────────── menus / paleta / atalhos ─────────────────────────

  private closeMenus(): void {
    this.wb.querySelectorAll('.menu, .suggest').forEach((n) => n.remove());
  }

  private openMenu(anchor: HTMLElement, html: string, right = false): void {
    this.closeMenus();
    const m = document.createElement('div');
    m.className = 'menu';
    m.innerHTML = html;
    this.wb.appendChild(m);
    const r = anchor.getBoundingClientRect();
    m.style.top = Math.min(r.bottom + 6, window.innerHeight - m.offsetHeight - 8) + 'px';
    m.style.left = (right ? r.right - m.offsetWidth : r.left + 40) + 'px';
    if (!right && r.left < 60) m.style.top = Math.max(8, r.top - 40) + 'px';
  }

  private appearanceMenu(anchor: HTMLElement): void {
    const ck = (on: boolean) => `<span class="ck">${on ? '✓' : ''}</span>`;
    const p = this.prefs;
    const sc = (v: number) => Math.abs(p.uiScale - v) < 0.05;
    this.openMenu(anchor, `
      <div class="lb">Tema</div>
      <button class="mi" data-act="set-mode" data-mode="dark">${ck(p.mode === 'dark')}Escuro</button>
      <button class="mi" data-act="set-mode" data-mode="light">${ck(p.mode === 'light')}Claro</button><hr>
      <div class="lb">Tamanho da interface</div>
      ${UI_STEPS.map((s, i) => `<button class="mi" data-act="set-ui" data-ui="${s}">${ck(sc(s))}${['Padrão (100%)', 'Médio (115%)', 'Grande (130%)'][i]}</button>`).join('')}<hr>
      <div class="lb">Painéis</div>
      <button class="mi" data-act="ToggleAutoCollapseInspector">${ck(p.autoHide)}Recolher inspector quando nada selecionado</button>
      <button class="mi" data-act="ToggleCardTransparency">${ck(p.translucent)}Cartões translúcidos</button>`);
  }

  private exportMenu(anchor: HTMLElement): void {
    this.openMenu(anchor, `
      <button class="mi" data-act="ExportPng"><span class="ck"></span>PNG</button>
      <button class="mi" data-act="ExportJpeg"><span class="ck"></span>JPEG</button>
      <button class="mi" data-act="ExportSvg"><span class="ck"></span>SVG vetorial</button>
      <button class="mi" data-act="ExportMermaid"><span class="ck"></span>Mermaid erDiagram</button>`);
  }

  private modal(inner: string): HTMLElement {
    this.closeModal();
    const bg = document.createElement('div');
    bg.className = 'modal-bg';
    bg.dataset.modal = '1';
    bg.innerHTML = `<div class="modal">${inner}</div>`;
    bg.addEventListener('mousedown', (e) => {
      if (e.target === bg) this.closeModal();
    });
    this.wb.appendChild(bg);
    return bg;
  }

  private closeModal(): void {
    this.wb.querySelectorAll('[data-modal]').forEach((n) => n.remove());
    this.paletteOpen = false;
    this.shortcutsOpen = false;
  }

  private openPalette(): void {
    this.paletteOpen = true;
    this.paletteQuery = '';
    this.paletteIdx = 0;
    const bg = this.modal(`<input id="pal-q" placeholder="Digite um comando…" autocomplete="off"><div class="list" id="pal-list"></div>`);
    const q = bg.querySelector('#pal-q') as HTMLInputElement;
    const draw = () => {
      const items = this.paletteItems();
      this.paletteIdx = Math.min(this.paletteIdx, Math.max(items.length - 1, 0));
      let last = '';
      (bg.querySelector('#pal-list') as HTMLElement).innerHTML = items
        .map((a, i) => {
          const head = a.group !== last ? `<div class="grp">${a.group}</div>` : '';
          last = a.group;
          return `${head}<div class="it ${i === this.paletteIdx ? 'on' : ''}" data-act="palette-run" data-id="${a.id}"><span>${esc(a.label)}</span><span>${a.displayKeys.map((k) => `<kbd>${k}</kbd>`).join(' ')}</span></div>`;
        })
        .join('');
    };
    q.addEventListener('input', () => { this.paletteQuery = q.value; this.paletteIdx = 0; draw(); });
    q.addEventListener('keydown', (e) => {
      const items = this.paletteItems();
      if (e.key === 'ArrowDown') { this.paletteIdx = Math.min(this.paletteIdx + 1, items.length - 1); draw(); e.preventDefault(); }
      else if (e.key === 'ArrowUp') { this.paletteIdx = Math.max(this.paletteIdx - 1, 0); draw(); e.preventDefault(); }
      else if (e.key === 'Enter') { const a = items[this.paletteIdx]; this.closeModal(); if (a) this.runAction(a.id); e.preventDefault(); }
      else if (e.key === 'Escape') this.closeModal();
    });
    draw();
    q.focus();
  }

  private paletteItems() {
    const q = this.paletteQuery.trim().toLowerCase();
    const fuzzy = (s: string): boolean => {
      let i = 0;
      for (const ch of s.toLowerCase()) if (ch === q[i]) i++;
      return i >= q.length;
    };
    return catalog().filter((a) => a.id !== 'OpenPalette' && (!q || fuzzy(a.label) || fuzzy(a.group)));
  }

  private openShortcuts(): void {
    this.shortcutsOpen = true;
    const groups: Record<string, string> = {};
    for (const a of catalog()) {
      if (!a.displayKeys.length) continue;
      groups[a.group] = (groups[a.group] || '') +
        `<div class="it"><span>${esc(a.label)}</span><span>${a.displayKeys.map((k) => `<kbd>${k}</kbd>`).join(' ')}</span></div>`;
    }
    this.modal(`<div class="list">${Object.entries(groups).map(([g, h]) => `<div class="grp">${g}</div>${h}`).join('')}</div>`);
  }

  // ───────────────────────── eventos ─────────────────────────

  private bind(): void {
    const wb = this.wb;
    wb.addEventListener('click', (e) => {
      const el = (e.target as HTMLElement).closest('[data-act]') as HTMLElement | null;
      if (!el) { if (!(e.target as HTMLElement).closest('.menu')) this.closeMenus(); return; }
      this.handleAct(el.dataset.act!, el);
    });
    const surf = this.els['surface'];
    surf.addEventListener('mousedown', (e) => this.onDown(e));
    window.addEventListener('mousemove', (e) => this.onMove(e));
    window.addEventListener('mouseup', () => this.onUp());
    surf.addEventListener('wheel', (e) => this.onWheel(e), { passive: false });
    window.addEventListener('resize', () => this.scheduleRender());
    window.addEventListener('keydown', (e) => this.onKey(e));
    window.addEventListener('keyup', (e) => {
      if (e.code === 'Space') { this.space = false; this.scheduleRender(); }
    });
    // editor DBML
    const ta = this.els['code'] as HTMLTextAreaElement;
    ta.addEventListener('input', () => {
      if (ta.value === this.source) return;
      this.applyPatch(this.fullReplace(ta.value));
      this.scheduleRender();
    });
    ta.addEventListener('scroll', () => { this.els['gutter'].scrollTop = ta.scrollTop; });
    // inspector
    const insp = this.els['inspector'];
    insp.addEventListener('change', (e) => this.onInspectorChange(e.target as HTMLInputElement));
    insp.addEventListener('keydown', (e) => {
      const t = e.target as HTMLInputElement;
      if (t.tagName !== 'INPUT') return;
      if (e.key === 'Enter') t.blur();
      else if (e.key === 'Escape') { t.value = t.dataset.orig || ''; t.blur(); this.closeMenus(); e.stopPropagation(); }
    });
    insp.addEventListener('focusin', (e) => this.suggest(e.target as HTMLInputElement));
    insp.addEventListener('input', (e) => this.suggest(e.target as HTMLInputElement));
    insp.addEventListener('focusout', () => setTimeout(() => this.wb.querySelectorAll('.suggest').forEach((n) => n.remove()), 120));
    insp.addEventListener('mousedown', (e) => {
      const sp = (e.target as HTMLElement).closest('[data-split]');
      if (!sp) return;
      if ((e as MouseEvent).detail >= 2) { this.prefs.colTypeW = 110; this.savePrefs(); this.renderChrome(); return; }
      this.colResize = { x: (e as MouseEvent).clientX, w: this.prefs.colTypeW };
      this.wb.style.userSelect = 'none';
      e.preventDefault();
    });
  }

  private handleAct(act: string, el: HTMLElement): void {
    const d = el.dataset;
    switch (act) {
      case 'menu:appearance': this.appearanceMenu(el); return;
      case 'menu:export': this.exportMenu(el); return;
      case 'set-mode': this.prefs.mode = d.mode as Mode; break;
      case 'set-ui': this.prefs.uiScale = parseFloat(d.ui || '1'); break;
      case 'open-file': this.transport.openFile?.(d.path!); return;
      case 'focus-table': this.focusTable(d.table!); return;
      case 'goto-diag': {
        const g = this.diags[parseInt(d.i!, 10)];
        if (!g) return;
        this.editorOpen = true;
        this.renderChrome();
        const ta = this.els['code'] as HTMLTextAreaElement;
        const off = this.offsetOf(g.range.start.line, g.range.start.character);
        ta.focus();
        ta.setSelectionRange(off, off);
        ta.scrollTop = Math.max(0, (g.range.start.line - 3) * 18);
        return;
      }
      case 'palette-run': this.closeModal(); this.runAction(d.id as ActionId); return;
      case 'set-color':
        this.dispatch({ action: 'set_table_color', payload: { table: d.table!, color: d.color || null } });
        return;
      case 'toggle-flag':
        this.dispatch({ action: 'set_column_flag', payload: { table: d.table!, column: d.col!, flag: d.flag!, value: d.on !== '1' } });
        return;
      case 'del-col':
        this.dispatch({ action: 'delete_column', payload: { table: d.table!, column: d.col! } });
        return;
      case 'add-col':
        this.dispatch({ action: 'add_column', payload: { table: d.table! } });
        return;
      default:
        this.closeMenus();
        this.runAction(act as ActionId);
        return;
    }
    this.closeMenus();
    this.savePrefs();
    this.scheduleRender();
  }

  private onInspectorChange(t: HTMLInputElement): void {
    const kind = t.dataset.edit;
    if (!kind) return;
    const val = t.value.trim();
    const orig = t.dataset.orig || '';
    if (val === orig) return;
    if (kind !== 'coltype' && !IDENT.test(val)) {
      this.toast('Nome inválido: use apenas letras, números e _');
      t.value = orig;
      return;
    }
    if (!val) { t.value = orig; return; }
    const table = t.dataset.table!;
    if (kind === 'table') this.dispatch({ action: 'rename_table', payload: { old_name: table, new_name: val } });
    else if (kind === 'colname') this.dispatch({ action: 'rename_column', payload: { table, old_name: t.dataset.col!, new_name: val } });
    else this.dispatch({ action: 'set_column_type', payload: { table, column: t.dataset.col!, ty: val } });
  }

  private suggest(t: HTMLInputElement): void {
    if (!t || t.dataset?.edit !== 'coltype') return;
    this.wb.querySelectorAll('.suggest').forEach((n) => n.remove());
    const q = t.value.trim().toLowerCase();
    const opts: Array<[string, string]> = [];
    for (const [ty, d] of STANDARD_TYPES) if (!q || ty.toLowerCase().includes(q)) opts.push([ty, d]);
    for (const en of this.enums) if (!q || en.toLowerCase().includes(q)) opts.push([en, 'enumeração']);
    if (!opts.length) return;
    const box = document.createElement('div');
    box.className = 'suggest';
    box.innerHTML = `<div class="hh"><span>Sugestões de tipo (clique para aplicar)</span><span>Esc fecha</span></div>` +
      opts.slice(0, 8).map(([ty, d]) => `<div class="op" data-ty="${esc(ty)}"><span>${esc(ty)}</span><small>${esc(d)}</small></div>`).join('');
    const r = t.getBoundingClientRect();
    box.style.left = Math.min(r.left, window.innerWidth - 250) + 'px';
    box.style.top = r.bottom + 2 + 'px';
    // mousedown + preventDefault: aplica o tipo ANTES do blur do input salvar o texto incompleto
    box.addEventListener('mousedown', (e) => {
      const op = (e.target as HTMLElement).closest('.op') as HTMLElement | null;
      if (!op) return;
      e.preventDefault();
      box.remove();
      this.dispatch({ action: 'set_column_type', payload: { table: t.dataset.table!, column: t.dataset.col!, ty: op.dataset.ty! } });
    });
    this.wb.appendChild(box);
  }

  private onDown(e: MouseEvent): void {
    const t = e.target as HTMLElement;
    this.closeMenus();
    this.els['surface'].focus();
    if (t.closest('.island, .legend, .empty')) return;
    if (e.button === 1 || this.tool === 'hand' || this.space) {
      this.drag = { kind: 'pan', lx: e.clientX, ly: e.clientY };
      e.preventDefault();
      this.scheduleRender();
      return;
    }
    if (e.button !== 0) return;
    const wp = this.toWorld(e);
    const port = t.closest('[data-port]') as HTMLElement | null;
    if (port) {
      this.drag = { kind: 'link', table: port.dataset.table!, col: port.dataset.col!, now: wp };
      e.preventDefault();
      this.scheduleRender();
      return;
    }
    const plus = t.closest('[data-addcol]') as HTMLElement | null;
    if (plus) {
      this.selection = [plus.dataset.addcol!];
      this.dispatch({ action: 'add_column', payload: { table: plus.dataset.addcol! } });
      return;
    }
    const gl = t.closest('[data-group]') as HTMLElement | null;
    if (gl) {
      this.drag = { kind: 'group', name: gl.dataset.group!, last: wp };
      this.scheduleRender();
      return;
    }
    const card = t.closest('.card') as HTMLElement | null;
    if (card) {
      const name = card.dataset.table!;
      if (e.shiftKey) {
        this.selection = this.selection.includes(name) ? this.selection.filter((n) => n !== name) : [...this.selection, name];
      } else if (!this.selection.includes(name)) {
        this.selection = [name];
      }
      this.top = name;
      if (e.detail >= 2) {
        this.selection = [name];
        this.runAction('RenameSelected');
        return;
      }
      this.drag = { kind: 'cards', last: wp };
      this.scheduleRender();
      return;
    }
    const base = e.shiftKey ? [...this.selection] : [];
    this.selection = base;
    this.drag = { kind: 'marquee', start: wp, now: wp, base };
    this.scheduleRender();
  }

  private onMove(e: MouseEvent): void {
    if (this.colResize) {
      this.prefs.colTypeW = Math.min(Math.max(this.colResize.w - (e.clientX - this.colResize.x), 60), 360);
      this.renderChrome();
      return;
    }
    const d = this.drag;
    if (!d) return;
    const wp = this.toWorld(e);
    if (d.kind === 'pan') {
      this.pan = { x: this.pan.x + e.clientX - d.lx, y: this.pan.y + e.clientY - d.ly };
      d.lx = e.clientX; d.ly = e.clientY;
    } else if (d.kind === 'cards') {
      const dx = wp.x - d.last.x, dy = wp.y - d.last.y;
      d.last = wp;
      for (const n of this.selection) {
        const p = this.positions[n];
        if (p) { p.x += dx; p.y += dy; }
      }
      this.recomputeEdges();
    } else if (d.kind === 'group') {
      const dx = wp.x - d.last.x, dy = wp.y - d.last.y;
      d.last = wp;
      const g = this.schema.groups.find((x) => x.name === d.name);
      for (const n of g?.tables || []) {
        const p = this.positions[n];
        if (p) { p.x += dx; p.y += dy; }
      }
      this.recomputeEdges();
    } else if (d.kind === 'marquee') {
      d.now = wp;
    } else if (d.kind === 'link') {
      d.now = wp;
    }
    this.scheduleRender();
  }

  /** Recalcula rotas ortogonais com as posições atuais (Wasm: `layout::route`). */
  private recomputeEdges(): void {
    try {
      this.scene = this.wasm.compute_scene(this.source, this.positions);
    } catch { /* mantém a cena anterior */ }
  }

  private onUp(): void {
    if (this.colResize) {
      this.colResize = null;
      this.wb.style.userSelect = '';
      this.savePrefs();
      return;
    }
    const d = this.drag;
    if (!d) return;
    this.drag = null;
    if (d.kind === 'marquee') {
      const x0 = Math.min(d.start.x, d.now.x), x1 = Math.max(d.start.x, d.now.x);
      const y0 = Math.min(d.start.y, d.now.y), y1 = Math.max(d.start.y, d.now.y);
      if (x1 - x0 > 2 || y1 - y0 > 2) {
        const sel = [...d.base];
        for (const t of this.schema.tables) {
          const p = this.positions[t.name];
          if (p && p.x < x1 && p.x + CARD_W > x0 && p.y < y1 && p.y + cardH(t.columns.length) > y0 && !sel.includes(t.name)) sel.push(t.name);
        }
        this.selection = sel;
      }
    } else if (d.kind === 'link') {
      const el = document.elementFromPoint(this.lastMouse.x, this.lastMouse.y) as HTMLElement | null;
      const row = el?.closest('[data-col]') as HTMLElement | null;
      if (row && row.dataset.table && !(row.dataset.table === d.table && row.dataset.col === d.col)) {
        this.dispatch({ action: 'add_ref', payload: { from_table: d.table, from_col: d.col, to_table: row.dataset.table, to_col: row.dataset.col! } });
      }
    } else {
      this.queueSaveLayout();
    }
    this.scheduleRender();
  }

  private lastMouse: Pt = { x: 0, y: 0 };

  private onWheel(e: WheelEvent): void {
    e.preventDefault();
    const r = this.els['surface'].getBoundingClientRect();
    if (e.ctrlKey || e.metaKey) {
      this.zoomAt(e.clientX - r.left, e.clientY - r.top, Math.min(Math.max(1 - e.deltaY * 0.0025, 0.5), 2));
    } else if (e.shiftKey && e.deltaX === 0) {
      this.pan.x -= e.deltaY;
    } else {
      this.pan.x -= e.deltaX;
      this.pan.y -= e.deltaY;
    }
    this.queueSaveLayout();
    this.scheduleRender();
  }

  private keyOf(e: KeyboardEvent): string {
    let k = e.key.toLowerCase();
    if (e.code.startsWith('Digit')) k = e.code.slice(5);
    else if (e.code === 'Minus') k = '-';
    else if (e.code === 'Equal') k = '=';
    else if (e.code === 'Slash') k = '/';
    if (k === 'backspace') k = 'delete';
    if (k === 'arrowup') k = 'up';
    if (k === 'arrowdown') k = 'down';
    const mods = [(e.ctrlKey || e.metaKey) && 'ctrl', e.altKey && 'alt', e.shiftKey && 'shift'].filter(Boolean);
    return [...mods, k].join('-');
  }

  private onKey(e: KeyboardEvent): void {
    const ae = document.activeElement as HTMLElement | null;
    const typing = !!ae && (ae.tagName === 'INPUT' || ae.tagName === 'TEXTAREA' || ae.isContentEditable);
    if (e.code === 'Space' && !typing && !this.paletteOpen) {
      if (!this.space) { this.space = true; this.scheduleRender(); }
      e.preventDefault();
      return;
    }
    if (this.paletteOpen && e.key !== 'Escape') return;
    const key = this.keyOf(e);
    const entry = catalog().find((a) => a.key === key || (a.id === 'DeleteSelection' && key === 'delete'));
    if (!entry || !entry.key) {
      if (e.key === 'Escape' && (this.paletteOpen || this.shortcutsOpen)) this.closeModal();
      return;
    }
    if (entry.context === 'Canvas' && (typing || this.paletteOpen || this.shortcutsOpen)) {
      if (entry.id === 'Deselect' && (this.paletteOpen || this.shortcutsOpen)) this.closeModal();
      return;
    }
    // Alt+Up/Down só valem dentro do editor DBML
    if ((entry.id === 'MoveLineUp' || entry.id === 'MoveLineDown') && ae !== this.els['code']) return;
    e.preventDefault();
    this.closeMenus();
    if (entry.id === 'Deselect' && (this.paletteOpen || this.shortcutsOpen)) { this.closeModal(); return; }
    this.runAction(entry.id);
  }

  // rastreia posição do mouse para soltar relações sobre colunas
  public attachMouseTracking(): void {
    window.addEventListener('mousemove', (e) => { this.lastMouse = { x: e.clientX, y: e.clientY }; });
  }
}

/** Monta o Workbench em `root` e inicia o rastreamento de mouse. */
export function mountWorkbench(root: HTMLElement, transport: HostTransport, wasm?: any): Workbench {
  const wb = new Workbench(root, transport, wasm);
  wb.attachMouseTracking();
  return wb;
}
