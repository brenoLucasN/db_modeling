import { describe, it, expect } from 'vitest';
import { DiagramRenderer, TABLE_WIDTH, HEADER_HEIGHT, ROW_HEIGHT } from '../src/renderer';
import { Schema } from '../src/types';

describe('DiagramRenderer - Hit Testing', () => {
  const schema: Schema = {
    tables: [
      {
        name: 'users',
        columns: [
          { name: 'id', ty: 'int', pk: true, not_null: true, unique: false },
          { name: 'email', ty: 'varchar', pk: false, not_null: true, unique: true },
          { name: 'created_at', ty: 'timestamp', pk: false, not_null: false, unique: false },
        ],
      },
    ],
    refs: [],
    groups: [],
  };

  const positions = {
    users: { x: 100, y: 100 },
  };

  const renderer = new DiagramRenderer();

  it('deve detectar clique no cabeçalho da tabela', () => {
    // Cabeçalho vai de y=100 até y=100 + HEADER_HEIGHT (38)
    const hit = renderer.hitTest({ x: 150, y: 120 }, schema, positions);

    expect(hit).not.toBeNull();
    expect(hit?.table.name).toBe('users');
    expect(hit?.isHeader).toBe(true);
    expect(hit?.column).toBeNull();
  });

  it('deve detectar clique na primeira coluna (id)', () => {
    // Linha 0: y de 138 até 166
    const hit = renderer.hitTest({ x: 150, y: 150 }, schema, positions);

    expect(hit).not.toBeNull();
    expect(hit?.table.name).toBe('users');
    expect(hit?.isHeader).toBe(false);
    expect(hit?.columnIndex).toBe(0);
    expect(hit?.column?.name).toBe('id');
    expect(hit?.column?.pk).toBe(true);
  });

  it('deve detectar clique na segunda coluna (email)', () => {
    // Linha 1: y de 166 até 194
    const hit = renderer.hitTest({ x: 150, y: 180 }, schema, positions);

    expect(hit).not.toBeNull();
    expect(hit?.table.name).toBe('users');
    expect(hit?.isHeader).toBe(false);
    expect(hit?.columnIndex).toBe(1);
    expect(hit?.column?.name).toBe('email');
    expect(hit?.column?.unique).toBe(true);
  });

  it('deve retornar null se clicar fora dos limites da tabela', () => {
    const hitOutsideLeft = renderer.hitTest({ x: 50, y: 120 }, schema, positions);
    expect(hitOutsideLeft).toBeNull();

    const hitOutsideRight = renderer.hitTest({ x: 100 + TABLE_WIDTH + 10, y: 120 }, schema, positions);
    expect(hitOutsideRight).toBeNull();

    const hitOutsideBottom = renderer.hitTest({ x: 150, y: 300 }, schema, positions);
    expect(hitOutsideBottom).toBeNull();
  });
});
