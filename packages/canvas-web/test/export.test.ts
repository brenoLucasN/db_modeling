import { describe, it, expect } from 'vitest';
import { exportToMermaid, exportToSvg } from '../src/export';
import { Schema } from '../src/types';

describe('Diagram Export', () => {
  const schema: Schema = {
    tables: [
      {
        name: 'users',
        columns: [
          { name: 'id', ty: 'int', pk: true, not_null: true, unique: false },
          { name: 'email', ty: 'varchar', pk: false, not_null: true, unique: true },
        ],
      },
      {
        name: 'posts',
        columns: [
          { name: 'id', ty: 'int', pk: true, not_null: true, unique: false },
          { name: 'user_id', ty: 'int', pk: false, not_null: true, unique: false },
        ],
      },
    ],
    refs: [
      {
        from: { table: 'posts', column: 'user_id' },
        to: { table: 'users', column: 'id' },
        kind: 'ManyToOne',
      },
    ],
    groups: [],
  };

  const positions = {
    users: { x: 50, y: 50 },
    posts: { x: 350, y: 50 },
  };

  it('should export to Mermaid erDiagram with valid syntax and cardinality', () => {
    const mermaid = exportToMermaid(schema);

    expect(mermaid).toContain('erDiagram');
    expect(mermaid).toContain('users {');
    expect(mermaid).toContain('int id PK');
    expect(mermaid).toContain('varchar email UK');
    expect(mermaid).toContain('posts {');
    expect(mermaid).toContain('posts }o--|| users : "user_id to id"');
  });

  it('should export to SVG containing vector nodes and edges', () => {
    const svg = exportToSvg(schema, positions);

    expect(svg).toContain('<svg');
    expect(svg).toContain('</svg>');
    expect(svg).toContain('users');
    expect(svg).toContain('posts');
    expect(svg).toContain('path d="M');
    expect(svg).toContain('class="table-box"');
  });
});
