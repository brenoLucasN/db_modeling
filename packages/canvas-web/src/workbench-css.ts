/** Estilos do Workbench: reproduz o Design System `Glass` de `crates/app/src/glass.rs`. */

export const WORKBENCH_CSS = `
.wb, .wb * { box-sizing: border-box; }
.wb { position: absolute; inset: 0; display: flex; flex-direction: column; overflow: hidden;
  background: var(--chrome); color: var(--text); font: 13.5px/1.35 Inter, var(--vscode-font-family, system-ui), sans-serif;
  user-select: none; -webkit-user-select: none; }
.wb button { font: inherit; color: inherit; background: none; border: 0; cursor: pointer; }
.wb input, .wb textarea { font: inherit; color: var(--text); }
.wb svg { flex-shrink: 0; }
.wb .ui { zoom: var(--ui, 1); }
.wb .titlebar { height: 38px; flex: none; display: flex; align-items: center; gap: 8px; padding: 0 10px;
  background: var(--chrome); border-bottom: 1px solid var(--edge); }
.wb .titlebar .doc { display: flex; align-items: center; gap: 6px; padding: 3px 10px; border-radius: 6px;
  background: var(--hover); color: var(--text); font-weight: 500; max-width: 260px; }
.wb .titlebar .doc span { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.wb .titlebar .spacer { flex: 1; }
.wb .pill { display: flex; align-items: center; gap: 8px; height: 26px; min-width: 320px; padding: 0 12px;
  border-radius: 8px; border: 1px solid var(--edge); background: var(--surface); color: var(--faint); }
.wb .pill:hover { background: var(--hover); color: var(--muted); }
.wb .ib { width: 30px; height: 30px; border-radius: 7px; display: inline-flex; align-items: center; justify-content: center;
  color: var(--faint); position: relative; }
.wb .ib:hover { background: var(--hover); color: var(--text); }
.wb .ib.on { background: var(--hover); color: var(--text); }
.wb .ib .badge { position: absolute; top: 2px; right: 2px; min-width: 14px; height: 14px; padding: 0 3px; border-radius: 99px;
  background: color-mix(in srgb, var(--accent) 25%, transparent); border: 1px solid color-mix(in srgb, var(--accent) 60%, transparent);
  color: var(--accent); font-size: 9px; display: flex; align-items: center; justify-content: center; }
.wb .main { flex: 1; min-height: 0; display: flex; }
.wb .rail { width: 44px; flex: none; display: flex; flex-direction: column; align-items: center; gap: 4px; padding: 8px 0; background: var(--chrome); }
.wb .rail.left { border-right: 1px solid var(--edge); }
.wb .rail.right { border-left: 1px solid var(--edge); }
.wb .rail .grow { flex: 1; }
.wb .sidebar { width: 260px; flex: none; background: var(--sidebar); border-right: 1px solid var(--edge); overflow-y: auto; }
.wb .editor { width: 420px; flex: none; display: flex; flex-direction: column; background: var(--surface); border-right: 1px solid var(--edge); }
.wb .editor .eh { height: 32px; display: flex; align-items: center; gap: 6px; padding: 0 12px; border-bottom: 1px solid var(--edge); font-size: 12px; color: var(--muted); }
.wb .editor .eh .t { flex: 1; }
.wb .editor .body { flex: 1; min-height: 0; display: flex; }
.wb .gutter { width: 44px; flex: none; overflow: hidden; padding: 8px 6px; text-align: right; color: var(--faint);
  font: 12px/18px ui-monospace, SFMono-Regular, Menlo, monospace; white-space: pre; border-right: 1px solid var(--edge-soft); }
.wb textarea.code { flex: 1; resize: none; border: 0; outline: 0; background: transparent; padding: 8px 10px; user-select: text; -webkit-user-select: text;
  font: 12px/18px ui-monospace, SFMono-Regular, Menlo, monospace; white-space: pre; overflow: auto; tab-size: 2; }
.wb .surface { flex: 1; min-width: 0; position: relative; overflow: hidden; background: var(--surface); outline: none; }
.wb .world { position: absolute; left: 0; top: 0; width: 0; height: 0; transform-origin: 0 0; }
.wb .world svg.edges { position: absolute; left: 0; top: 0; width: 1px; height: 1px; overflow: visible; pointer-events: none; }
.wb .group { position: absolute; border-radius: 14px; border: 1px solid; }
.wb .group .gl { position: absolute; top: 8px; left: 10px; height: 22px; padding: 0 8px; border-radius: 6px; display: flex; align-items: center; gap: 6px;
  font-size: 12px; font-weight: 600; cursor: move; }
.wb .group .gl i { width: 7px; height: 7px; border-radius: 99px; display: block; }
.wb .group .gl b { font-weight: 400; opacity: .7; }
.wb .card { position: absolute; width: 260px; background: var(--card); border: 1px solid var(--edge); border-radius: 12px;
  box-shadow: var(--card-shadow); }
.wb.translucent .card { background: var(--card-translucent); backdrop-filter: blur(8px); }
.wb .card.sel { box-shadow: var(--card-shadow-sel); border-color: var(--accent); }
.wb .card .bar { position: absolute; left: 0; right: 0; top: 0; height: 3px; border-radius: 12px 12px 0 0; }
.wb .card .head { height: 44px; display: flex; align-items: center; gap: 8px; padding: 0 12px; border-bottom: 1px solid var(--edge-soft);
  border-radius: 12px 12px 0 0; cursor: move; }
.wb .card .head .ic { width: 22px; height: 22px; border-radius: 6px; display: flex; align-items: center; justify-content: center; background: var(--hover); }
.wb .card .head .nm { flex: 1; font-weight: 600; font-size: 14px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.wb .card .head .cnt { font-size: 10px; padding: 1px 6px; border-radius: 99px; background: var(--hover); color: var(--muted); }
.wb .card .rows { padding: 4px 0; }
.wb .row { position: relative; height: 30px; padding: 0 12px; display: flex; align-items: center; gap: 8px; font-size: 13px; }
.wb .row:hover { background: var(--hover); }
.wb .row .rail14 { width: 14px; display: flex; justify-content: center; }
.wb .row .cn { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.wb .row .cn.pk { font-weight: 500; }
.wb .row .ty { font-size: 11.5px; color: var(--muted); }
.wb .row .nn { font-size: 10px; color: var(--muted); }
.wb .row .port { position: absolute; right: -6px; top: 9px; width: 12px; height: 12px; border-radius: 99px; border: 2px solid var(--accent);
  background: var(--card); cursor: crosshair; display: none; }
.wb .row:hover .port { display: block; }
.wb .card .plus { position: absolute; left: 119px; bottom: -11px; width: 22px; height: 22px; border-radius: 99px; background: var(--accent);
  color: var(--surface); display: none; align-items: center; justify-content: center; cursor: pointer; }
.wb .card:hover .plus { display: flex; }
.wb .marquee { position: absolute; border: 1px solid var(--accent); background: var(--selected); display: none; pointer-events: none; }
.wb .island { position: absolute; z-index: 20; display: flex; align-items: center; gap: 2px; padding: 4px; border-radius: 10px;
  background: var(--island); border: 1px solid var(--edge); box-shadow: var(--island-shadow); backdrop-filter: blur(12px); }
.wb .island .sep { width: 1px; height: 18px; margin: 0 4px; background: var(--edge); }
.wb .island .pct { min-width: 48px; height: 30px; border-radius: 7px; font-size: 12px; color: var(--muted); }
.wb .island .pct:hover { background: var(--hover); }
.wb .tools { top: 12px; left: 50%; transform: translateX(-50%); }
.wb .zoomi { bottom: 12px; left: 12px; }
.wb .helpi { bottom: 12px; right: 12px; }
.wb .legend { position: absolute; z-index: 20; right: 12px; bottom: 58px; width: 240px; padding: 12px; border-radius: 10px;
  background: var(--island); border: 1px solid var(--edge); box-shadow: var(--island-shadow); font-size: 12px; color: var(--muted); display: none; }
.wb .legend h4 { margin: 0 0 8px; color: var(--text); font-size: 13px; }
.wb .legend div { display: flex; align-items: center; gap: 8px; margin: 5px 0; }
.wb .empty { position: absolute; z-index: 10; left: 50%; top: 50%; transform: translate(-50%, -50%); width: 340px; padding: 20px; text-align: center;
  border-radius: 12px; border: 1px solid var(--edge); background: var(--island); display: none; }
.wb .empty h3 { margin: 6px 0; font-size: 15px; }
.wb .empty p { margin: 0 0 12px; color: var(--muted); }
.wb .btn { height: 28px; padding: 0 12px; border-radius: 7px; border: 1px solid var(--edge); display: inline-flex; align-items: center; gap: 6px; background: var(--hover); }
.wb .btn:hover { background: var(--active); }
.wb .btn.primary { background: var(--accent); color: var(--surface); border-color: transparent; font-weight: 600; }
.wb .btn.ghost { background: transparent; border-color: transparent; color: var(--muted); }
.wb .btn.ghost:hover { background: var(--hover); color: var(--text); }
.wb .inspector { width: 300px; flex: none; background: var(--sidebar); border-left: 1px solid var(--edge); overflow-y: auto; overflow-x: hidden; position: relative; }
.wb .sec { display: flex; justify-content: space-between; padding: 12px 12px 4px; font-size: 12px; font-weight: 500; color: var(--faint); }
.wb .ihead { padding: 12px; border-bottom: 1px solid var(--edge); display: flex; flex-direction: column; gap: 2px; }
.wb .ihead .eb { font-size: 12px; font-weight: 500; color: var(--faint); }
.wb .ihead .ti { font-size: 16.5px; font-weight: 600; }
.wb .ihead .su { color: var(--muted); }
.wb input.fld { width: 100%; height: 24px; padding: 0 4px; border-radius: 4px; border: 1px solid transparent; background: transparent; outline: 0; user-select: text; -webkit-user-select: text; }
.wb input.fld:hover { background: var(--hover); }
.wb input.fld:focus { border-color: var(--accent); background: var(--surface); }
.wb input.fld.muted { color: var(--muted); }
.wb input.fld.big { font-size: 16.5px; font-weight: 600; height: 28px; }
.wb .sw { display: flex; flex-wrap: wrap; gap: 6px; padding: 0 12px; }
.wb .sw button { width: 22px; height: 22px; border-radius: 99px; border: 2px solid transparent; display: flex; align-items: center; justify-content: center; }
.wb .sw button.on { border-color: var(--text); }
.wb .sw button.none { border: 1px solid var(--edge); color: var(--faint); }
.wb .crow { display: flex; align-items: center; gap: 6px; padding: 0 12px; height: 32px; border-bottom: 1px solid var(--edge-soft); }
.wb .crow.hd { height: 22px; font-size: 10px; font-weight: 500; color: var(--faint); }
.wb .crow .n { flex: 1; min-width: 60px; overflow: hidden; }
.wb .crow .t { flex: none; overflow: hidden; font-size: 12px; }
.wb .crow .split { width: 4px; height: 14px; margin: 0 -2px; cursor: col-resize; flex: none; border-radius: 1px; }
.wb .crow .split:hover { background: color-mix(in srgb, var(--accent) 40%, transparent); }
.wb .chip { padding: 0 4px; font-size: 10px; border-radius: 4px; border: 1px solid var(--edge-soft); color: var(--faint); height: 18px; }
.wb .chip:hover { border-color: var(--edge); }
.wb .chip.on { background: var(--selected); border-color: color-mix(in srgb, var(--accent) 50%, transparent); color: var(--accent); }
.wb .crow .del { visibility: hidden; width: 20px; height: 20px; border-radius: 4px; color: var(--faint); display: flex; align-items: center; justify-content: center; }
.wb .crow:hover .del { visibility: visible; }
.wb .crow .del:hover { background: var(--hover); color: var(--danger); }
.wb .rel { display: flex; align-items: center; gap: 6px; padding: 2px 12px; color: var(--muted); }
.wb .rel span.tx { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.wb .tag { padding: 0 5px; border-radius: 4px; border: 1px solid var(--edge); font-size: 12px; color: var(--muted); }
.wb .pad { padding: 16px; display: flex; flex-direction: column; gap: 6px; color: var(--muted); }
.wb .kb { display: flex; justify-content: space-between; align-items: center; height: 26px; }
.wb kbd { font: 11px ui-monospace, monospace; padding: 1px 5px; border-radius: 4px; border: 1px solid var(--edge); background: var(--hover); color: var(--muted); }
.wb .srow { display: flex; align-items: center; gap: 8px; height: 28px; margin: 0 6px; padding: 0 8px; border-radius: 6px; color: var(--muted); cursor: pointer; }
.wb .srow:hover { background: var(--hover); }
.wb .srow.on { background: var(--selected); color: var(--text); }
.wb .srow .g { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.wb .srow .c { font-size: 12px; color: var(--faint); }
.wb .gh { display: flex; align-items: center; gap: 6px; padding: 8px 12px 2px; font-size: 12px; font-weight: 600; color: var(--muted); }
.wb .gh i { width: 7px; height: 7px; border-radius: 99px; display: block; }
.wb .prob { display: flex; gap: 8px; margin: 0 6px; padding: 5px 8px; border-radius: 6px; cursor: pointer; }
.wb .prob:hover { background: var(--hover); }
.wb .prob .sub { font-size: 12px; color: var(--faint); }
.wb .status { height: 24px; flex: none; display: flex; align-items: center; gap: 14px; padding: 0 10px; background: var(--chrome);
  border-top: 1px solid var(--edge); font-size: 12px; color: var(--muted); }
.wb .status .hint { flex: 1; text-align: center; color: var(--faint); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.wb .status button { display: inline-flex; align-items: center; gap: 4px; color: var(--muted); }
.wb .status button:hover { color: var(--text); }
.wb .menu { position: fixed; z-index: 60; min-width: 230px; padding: 6px; border-radius: 10px; background: var(--island);
  border: 1px solid var(--edge); box-shadow: var(--island-shadow); backdrop-filter: blur(14px); font-size: 13px; }
.wb .menu .lb { padding: 4px 8px; font-size: 11px; color: var(--faint); }
.wb .menu .mi { display: flex; align-items: center; gap: 8px; width: 100%; height: 28px; padding: 0 8px; border-radius: 6px; text-align: left; }
.wb .menu .mi:hover { background: var(--hover); }
.wb .menu .mi .ck { width: 14px; color: var(--accent); }
.wb .menu hr { border: 0; border-top: 1px solid var(--edge-soft); margin: 4px 0; }
.wb .modal-bg { position: absolute; inset: 0; z-index: 70; background: rgba(0,0,0,.45); display: flex; justify-content: center; align-items: flex-start; padding-top: 90px; }
.wb .modal { width: 560px; max-height: 70%; display: flex; flex-direction: column; border-radius: 12px; background: var(--island);
  border: 1px solid var(--edge); box-shadow: var(--island-shadow); overflow: hidden; }
.wb .modal input { height: 40px; padding: 0 14px; border: 0; border-bottom: 1px solid var(--edge); background: transparent; outline: 0; font-size: 14px; user-select: text; -webkit-user-select: text; }
.wb .modal .list { overflow-y: auto; padding: 6px; }
.wb .modal .it { display: flex; justify-content: space-between; align-items: center; height: 30px; padding: 0 10px; border-radius: 6px; cursor: pointer; }
.wb .modal .it.on, .wb .modal .it:hover { background: var(--hover); }
.wb .modal .grp { padding: 8px 10px 2px; font-size: 11px; color: var(--faint); }
.wb .suggest { position: fixed; z-index: 65; width: 240px; padding: 6px; border-radius: 6px; background: var(--island); border: 1px solid var(--edge); box-shadow: var(--island-shadow); }
.wb .suggest .hh { display: flex; justify-content: space-between; padding: 0 4px 2px; font-size: 10px; color: var(--faint); border-bottom: 1px solid var(--edge-soft); margin-bottom: 2px; }
.wb .suggest .op { display: flex; justify-content: space-between; padding: 3px 6px; border-radius: 4px; cursor: pointer; font-size: 12px; }
.wb .suggest .op:hover { background: var(--hover); }
.wb .suggest .op small { color: var(--faint); font-size: 10px; }
.wb .toast { position: absolute; z-index: 80; left: 50%; bottom: 40px; transform: translateX(-50%); padding: 8px 14px; border-radius: 8px;
  background: var(--island); border: 1px solid var(--warning); color: var(--text); display: none; }
`;
