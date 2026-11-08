import { describe, it, expect } from 'vitest';
import { Viewport2D } from '../src/viewport';

describe('Viewport2D', () => {
  it('deve converter coordenadas tela <-> mundo com zoom 1.0x e pan (0,0)', () => {
    const vp = new Viewport2D(0, 0, 1.0);
    const screen = { x: 150, y: 250 };
    const world = vp.screenToWorld(screen.x, screen.y);

    expect(world.x).toBeCloseTo(150);
    expect(world.y).toBeCloseTo(250);

    const backToScreen = vp.worldToScreen(world.x, world.y);
    expect(backToScreen.x).toBeCloseTo(150);
    expect(backToScreen.y).toBeCloseTo(250);
  });

  it('deve converter coordenadas com zoom 2.0x e pan deslocado', () => {
    const vp = new Viewport2D(100, 50, 2.0);
    const screen = { x: 300, y: 250 };
    // worldX = (300 - 100) / 2 = 100
    // worldY = (250 - 50) / 2 = 100
    const world = vp.screenToWorld(screen.x, screen.y);

    expect(world.x).toBeCloseTo(100);
    expect(world.y).toBeCloseTo(100);

    const backToScreen = vp.worldToScreen(world.x, world.y);
    expect(backToScreen.x).toBeCloseTo(300);
    expect(backToScreen.y).toBeCloseTo(250);
  });

  it('deve aplicar panBy acumulando deslocamentos', () => {
    const vp = new Viewport2D(0, 0, 1.0);
    vp.panBy(45, -30);
    expect(vp.panX).toBe(45);
    expect(vp.panY).toBe(-30);

    vp.panBy(-15, 20);
    expect(vp.panX).toBe(30);
    expect(vp.panY).toBe(-10);
  });

  it('deve manter o ponto focal do cursor invariante durante zoomAt', () => {
    const vp = new Viewport2D(50, 50, 1.0);
    const cursorScreen = { x: 200, y: 200 };

    const worldBefore = vp.screenToWorld(cursorScreen.x, cursorScreen.y);
    vp.zoomAt(cursorScreen, 0.5); // Aumenta o zoom
    const worldAfter = vp.screenToWorld(cursorScreen.x, cursorScreen.y);

    // O ponto do mundo sob o cursor de tela deve permanecer exatamente o mesmo
    expect(worldAfter.x).toBeCloseTo(worldBefore.x, 3);
    expect(worldAfter.y).toBeCloseTo(worldBefore.y, 3);
  });
});
