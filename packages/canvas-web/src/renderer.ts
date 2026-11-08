/**
 * Utilitários geométricos e de cores do Canvas do DB Modeling.
 * Mantém compatibilidade com suíte de testes unitários e constantes do Design System Glass.
 */

import { Schema, Table, Column } from './types';
import { Point } from './viewport';
import { CARD_W, HEADER_H, ROW_H, RADIUS_CARD, cardH } from './glass';

export const TABLE_WIDTH = CARD_W;
export const HEADER_HEIGHT = HEADER_H;
export const ROW_HEIGHT = ROW_H;
export const BORDER_RADIUS = RADIUS_CARD;

export interface ThemeColors {
  background: string;
  gridDot: string;
  tableBg: string;
  tableBorder: string;
  tableBorderSelected: string;
  headerDefaultBg: string;
  headerText: string;
  rowHoverBg: string;
  textPrimary: string;
  textSecondary: string;
  edgeColor: string;
  edgeColorActive: string;
  edgeGlow: string;
}

export const DARK_THEME: ThemeColors = {
  background: '#0a0a0b',
  gridDot: 'rgba(255, 255, 255, 0.07)',
  tableBg: '#151516',
  tableBorder: 'rgba(255, 255, 255, 0.10)',
  tableBorderSelected: '#c6f240',
  headerDefaultBg: 'rgba(198, 242, 64, 0.08)',
  headerText: '#ececed',
  rowHoverBg: 'rgba(255, 255, 255, 0.06)',
  textPrimary: '#ececed',
  textSecondary: '#a3a6ad',
  edgeColor: 'rgba(163, 166, 173, 0.55)',
  edgeColorActive: '#c6f240',
  edgeGlow: 'rgba(198, 242, 64, 0.25)',
};

export const LIGHT_THEME: ThemeColors = {
  background: '#f7f7f8',
  gridDot: 'rgba(31, 35, 40, 0.13)',
  tableBg: '#ffffff',
  tableBorder: 'rgba(31, 35, 40, 0.13)',
  tableBorderSelected: '#5f7d00',
  headerDefaultBg: 'rgba(95, 125, 0, 0.08)',
  headerText: '#1f2328',
  rowHoverBg: 'rgba(31, 35, 40, 0.05)',
  textPrimary: '#1f2328',
  textSecondary: '#57606a',
  edgeColor: 'rgba(87, 96, 106, 0.55)',
  edgeColorActive: '#5f7d00',
  edgeGlow: 'rgba(95, 125, 0, 0.25)',
};

export interface HitResult {
  table: Table;
  isHeader: boolean;
  columnIndex: number | null;
  column: Column | null;
}

export class DiagramRenderer {
  public theme: ThemeColors = DARK_THEME;

  public getTableHeight(table: Table): number {
    return cardH(table.columns.length);
  }

  /** Hit testing: identifica se um ponto de mundo está sobre uma tabela, cabeçalho ou coluna (`canvas.rs::column_at`) */
  public hitTest(
    worldPoint: Point,
    schema: Schema,
    positions: Record<string, Point>
  ): HitResult | null {
    for (let i = schema.tables.length - 1; i >= 0; i--) {
      const table = schema.tables[i];
      const pos = positions[table.name];
      if (!pos) continue;

      const height = this.getTableHeight(table);
      if (
        worldPoint.x >= pos.x &&
        worldPoint.x <= pos.x + CARD_W &&
        worldPoint.y >= pos.y &&
        worldPoint.y <= pos.y + height
      ) {
        const isHeader = worldPoint.y <= pos.y + HEADER_H;
        let colIndex: number | null = null;
        let col: Column | null = null;

        if (!isHeader) {
          const relativeY = worldPoint.y - (pos.y + HEADER_H + 4);
          colIndex = Math.floor(Math.max(0, relativeY) / ROW_H);
          if (colIndex >= 0 && colIndex < table.columns.length) {
            col = table.columns[colIndex];
          } else {
            colIndex = null;
          }
        }

        return {
          table,
          isHeader,
          columnIndex: colIndex,
          column: col,
        };
      }
    }

    return null;
  }
}
