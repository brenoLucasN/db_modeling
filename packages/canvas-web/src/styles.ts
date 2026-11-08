/**
 * Folha de estilos CSS dinâmicos baseada no Design System `Glass` (`crates/app/src/glass.rs`).
 * Reproduz fielmente as dimensões, sombras, bordas, ilhas em vidro, cartões, inspector e rails.
 */

import {
  Glass,
  CARD_W,
  HEADER_H,
  ROW_H,
  RADIUS_CARD,
  RADIUS_ISLAND,
  RAIL_W,
  STATUSBAR_H,
} from './glass';

export function buildStyles(g: Glass, translucentCards: boolean): string {
  const s = (px: number) => `${Math.round(px * g.uiScale * 10) / 10}px`;
  const cardBg = translucentCards ? g.cardTranslucent : g.card;

  return `
    .dbm-root {
      position: relative;
      width: 100%;
      height: 100%;
      display: flex;
      flex-direction: column;
      background: ${g.surface};
      color: ${g.text};
      font-family: Inter, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
      font-size: ${s(13.5)};
      user-select: none;
      overflow: hidden;
    }
    .dbm-root * {
      box-sizing: border-box;
    }
    .dbm-work {
      flex: 1;
      min-height: 0;
      display: flex;
      flex-direction: row;
      width: 100%;
      overflow: hidden;
    }
    /* Activity Rails (Left & Right) */
    .dbm-rail {
      width: ${RAIL_W}px;
      height: 100%;
      flex: none;
      display: flex;
      flex-direction: column;
      align-items: center;
      gap: 4px;
      padding: 8px 0;
      background: ${g.chrome};
      border-right: 1px solid ${g.edge};
      z-index: 20;
    }
    .dbm-rail-right {
      border-right: none;
      border-left: 1px solid ${g.edge};
    }
    .dbm-rail-btn {
      position: relative;
      width: 34px;
      height: 34px;
      border-radius: 7px;
      border: none;
      background: transparent;
      color: ${g.textFaint};
      display: flex;
      align-items: center;
      justify-content: center;
      cursor: pointer;
      padding: 0;
    }
    .dbm-rail-btn:hover {
      background: ${g.hover};
      color: ${g.text};
    }
    .dbm-rail-btn.active {
      background: ${g.hover};
      color: ${g.text};
    }
    .dbm-rail-badge {
      position: absolute;
      top: 3px;
      right: 3px;
      min-width: 14px;
      height: 14px;
      padding: 0 3px;
      border-radius: 999px;
      font-size: 9px;
      font-weight: 600;
      display: flex;
      align-items: center;
      justify-content: center;
    }
    /* Painéis laterais (Sidebar, Editor DBML, Inspector) */
    .dbm-sidebar {
      width: 260px;
      height: 100%;
      flex: none;
      background: ${g.sidebar};
      border-right: 1px solid ${g.edge};
      overflow-y: auto;
      display: flex;
      flex-direction: column;
      z-index: 15;
    }
    .dbm-editor-pane {
      width: 440px;
      height: 100%;
      flex: none;
      background: ${g.surface};
      border-right: 1px solid ${g.edge};
      display: flex;
      flex-direction: column;
      z-index: 15;
    }
    .dbm-editor-header {
      height: 32px;
      padding: 0 12px;
      border-bottom: 1px solid ${g.edge};
      display: flex;
      align-items: center;
      gap: 6px;
      font-size: ${s(12)};
      color: ${g.textMuted};
    }
    .dbm-editor-body {
      flex: 1;
      min-height: 0;
      position: relative;
      display: flex;
      overflow: hidden;
    }
    .dbm-editor-gutter {
      width: 40px;
      padding: 10px 6px 10px 0;
      text-align: right;
      font-family: "JetBrains Mono", ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
      font-size: ${s(12.5)};
      line-height: 20px;
      color: ${g.textFaint};
      background: ${g.surface};
      border-right: 1px solid ${g.edgeSoft};
      overflow: hidden;
      user-select: none;
    }
    .dbm-editor-textarea {
      flex: 1;
      height: 100%;
      padding: 10px 12px;
      background: ${g.surface};
      color: ${g.text};
      border: none;
      outline: none;
      resize: none;
      font-family: "JetBrains Mono", ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
      font-size: ${s(12.5)};
      line-height: 20px;
      white-space: pre;
      overflow: auto;
      tab-size: 2;
      user-select: text;
    }
    .dbm-inspector {
      width: 310px;
      height: 100%;
      flex: none;
      background: ${g.sidebar};
      border-left: 1px solid ${g.edge};
      overflow-y: auto;
      display: flex;
      flex-direction: column;
      z-index: 15;
    }
    .dbm-section-hdr {
      display: flex;
      justify-content: space-between;
      align-items: center;
      padding: 12px 12px 4px 12px;
      font-size: ${s(11.5)};
      font-weight: 600;
      color: ${g.textFaint};
      letter-spacing: 0.02em;
    }
    .dbm-row-btn {
      margin: 0 6px;
      min-height: 28px;
      padding: 0 8px;
      border-radius: 6px;
      display: flex;
      align-items: center;
      gap: 8px;
      font-size: ${s(13)};
      color: ${g.textMuted};
      cursor: pointer;
      border: none;
      background: transparent;
      text-align: left;
    }
    .dbm-row-btn:hover {
      background: ${g.hover};
      color: ${g.text};
    }
    .dbm-row-btn.selected {
      background: ${g.selected};
      color: ${g.text};
    }
    /* Área Central do Canvas */
    .dbm-canvas-stage {
      flex: 1;
      min-width: 0;
      height: 100%;
      position: relative;
      overflow: hidden;
      background: ${g.surface};
      outline: none;
    }
    .dbm-world {
      position: absolute;
      top: 0;
      left: 0;
      width: 0;
      height: 0;
      transform-origin: 0 0;
      will-change: transform;
    }
    .dbm-edges-svg {
      position: absolute;
      top: 0;
      left: 0;
      overflow: visible;
      pointer-events: none;
      z-index: 2;
    }
    /* Molduras de Grupo (TableGroup) */
    .dbm-group {
      position: absolute;
      border-radius: 14px;
      border-width: 1px;
      border-style: solid;
      z-index: 1;
      pointer-events: none;
    }
    .dbm-group-label {
      position: absolute;
      top: 8px;
      left: 10px;
      height: 22px;
      padding: 0 8px;
      border-radius: 6px;
      display: flex;
      align-items: center;
      gap: 6px;
      font-size: 12px;
      font-weight: 600;
      pointer-events: auto;
      cursor: move;
      white-space: nowrap;
    }
    /* Cartões de Tabela */
    .dbm-card {
      position: absolute;
      width: ${CARD_W}px;
      border-radius: ${RADIUS_CARD}px;
      border: 1px solid ${g.edge};
      background: ${cardBg};
      box-shadow: ${g.cardShadow};
      z-index: 3;
      transition: border-color 0.08s ease;
    }
    .dbm-card.selected {
      border-color: ${g.accent};
      box-shadow: ${g.cardShadowSelected};
    }
    .dbm-card-tint {
      position: absolute;
      top: 0;
      left: 0;
      width: 100%;
      height: 3px;
      border-top-left-radius: ${RADIUS_CARD}px;
      border-top-right-radius: ${RADIUS_CARD}px;
    }
    .dbm-card-hdr {
      height: ${HEADER_H}px;
      padding: 0 12px;
      display: flex;
      align-items: center;
      gap: 8px;
      border-top-left-radius: ${RADIUS_CARD}px;
      border-top-right-radius: ${RADIUS_CARD}px;
      border-bottom: 1px solid ${g.edgeSoft};
    }
    .dbm-card-icon {
      width: 22px;
      height: 22px;
      border-radius: 6px;
      display: flex;
      align-items: center;
      justify-content: center;
      flex: none;
    }
    .dbm-card-title {
      flex: 1;
      min-width: 0;
      overflow: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
      font-size: ${s(14)};
      font-weight: 600;
      color: ${g.text};
    }
    .dbm-card-count {
      padding: 1px 6px;
      border-radius: 5px;
      border: 1px solid ${g.edge};
      font-size: ${s(10.5)};
      color: ${g.textMuted};
    }
    .dbm-card-rows {
      padding-top: 4px;
    }
    .dbm-col-row {
      position: relative;
      height: ${ROW_H}px;
      padding: 0 12px;
      display: flex;
      align-items: center;
      gap: 8px;
      font-size: ${s(13)};
    }
    .dbm-col-row:hover {
      background: ${g.hover};
    }
    .dbm-col-port {
      position: absolute;
      right: -6px;
      top: ${ROW_H / 2 - 6}px;
      width: 12px;
      height: 12px;
      border-radius: 999px;
      border: 2px solid ${g.accent};
      background: ${g.card};
      cursor: crosshair;
      opacity: 0;
      transition: opacity 0.1s;
      z-index: 6;
    }
    .dbm-col-row:hover .dbm-col-port {
      opacity: 1;
    }
    .dbm-card-addcol {
      position: absolute;
      bottom: -11px;
      left: ${CARD_W / 2 - 11}px;
      width: 22px;
      height: 22px;
      border-radius: 999px;
      border: 1px solid ${g.edge};
      background: ${g.card};
      color: ${g.textMuted};
      display: flex;
      align-items: center;
      justify-content: center;
      cursor: pointer;
      opacity: 0;
      transition: opacity 0.1s, border-color 0.1s;
      z-index: 6;
      padding: 0;
    }
    .dbm-card:hover .dbm-card-addcol {
      opacity: 1;
    }
    .dbm-card-addcol:hover {
      border-color: ${g.accent};
      color: ${g.text};
    }
    /* Ilhas Flutuantes do Canvas */
    .dbm-island {
      display: flex;
      align-items: center;
      gap: 2px;
      padding: 4px;
      border-radius: ${RADIUS_ISLAND}px;
      border: 1px solid ${g.edge};
      background: ${g.island};
      backdrop-filter: blur(10px);
      box-shadow: ${g.islandShadow};
      z-index: 12;
    }
    .dbm-island-sep {
      width: 1px;
      height: 18px;
      margin: 0 4px;
      background: ${g.edge};
    }
    .dbm-tool-btn {
      width: 32px;
      height: 32px;
      border-radius: 8px;
      border: 1px solid transparent;
      background: transparent;
      color: ${g.textMuted};
      display: flex;
      align-items: center;
      justify-content: center;
      cursor: pointer;
      padding: 0;
    }
    .dbm-tool-btn:hover {
      background: ${g.hover};
      color: ${g.text};
    }
    .dbm-tool-btn.active {
      background: ${g.hover};
      border-color: ${g.islandSheen};
      color: ${g.text};
    }
    /* Status Bar */
    .dbm-statusbar {
      height: ${STATUSBAR_H}px;
      width: 100%;
      flex: none;
      display: flex;
      align-items: center;
      gap: 16px;
      padding: 0 12px;
      background: ${g.chrome};
      border-top: 1px solid ${g.edge};
      font-size: ${s(11.5)};
      color: ${g.textMuted};
      z-index: 25;
    }
    /* Inspector Controles */
    .dbm-insp-field {
      width: 100%;
      padding: 2px 4px;
      border-radius: 4px;
      cursor: text;
      overflow: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
    }
    .dbm-insp-field:hover {
      background: ${g.hover};
    }
    .dbm-insp-input {
      width: 100%;
      padding: 2px 6px;
      border-radius: 4px;
      border: 1px solid ${g.accent};
      background: ${g.surface};
      color: ${g.text};
      font-family: inherit;
      font-size: inherit;
      outline: none;
    }
    .dbm-chip {
      padding: 1px 4px;
      border-radius: 4px;
      border: 1px solid ${g.edgeSoft};
      font-size: 10px;
      color: ${g.textFaint};
      background: transparent;
      cursor: pointer;
    }
    .dbm-chip:hover {
      border-color: ${g.edge};
    }
    .dbm-chip.on {
      background: ${g.selected};
      border-color: ${g.accent};
      color: ${g.accent};
    }
    .dbm-insp-col-row {
      display: flex;
      align-items: center;
      gap: 6px;
      padding: 0 12px;
      height: 32px;
      border-bottom: 1px solid ${g.edgeSoft};
      font-size: ${s(13)};
    }
    .dbm-insp-col-del {
      opacity: 0;
      border: none;
      background: transparent;
      color: ${g.textMuted};
      cursor: pointer;
      padding: 2px;
      border-radius: 4px;
      display: flex;
      align-items: center;
    }
    .dbm-insp-col-row:hover .dbm-insp-col-del {
      opacity: 1;
    }
    .dbm-insp-col-del:hover {
      background: ${g.hover};
      color: ${g.danger};
    }
    .dbm-kbd {
      display: inline-flex;
      align-items: center;
      padding: 1px 5px;
      border-radius: 4px;
      border: 1px solid ${g.edge};
      background: ${g.surface};
      font-family: ui-monospace, monospace;
      font-size: 10.5px;
      color: ${g.textMuted};
    }
    /* Modais (Paleta Ctrl+K & Atalhos Ctrl+/) */
    .dbm-modal-backdrop {
      position: absolute;
      inset: 0;
      background: rgba(0, 0, 0, 0.45);
      display: flex;
      justify-content: center;
      align-items: flex-start;
      padding-top: 64px;
      z-index: 100;
    }
    .dbm-modal {
      width: 560px;
      max-width: 92vw;
      max-height: 75vh;
      border-radius: 10px;
      border: 1px solid ${g.edge};
      background: ${g.sidebar};
      box-shadow: ${g.islandShadow};
      display: flex;
      flex-direction: column;
      overflow: hidden;
    }
  `;
}
