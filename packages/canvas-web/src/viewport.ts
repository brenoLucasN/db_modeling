/**
 * Viewport 2D com transformações de matriz afim para câmera, pan e zoom contínuo.
 * Suporta Retina / HiDPI via Device Pixel Ratio (DPR).
 */

export interface Point {
  x: number;
  y: number;
}

export class Viewport2D {
  public panX: number = 0;
  public panY: number = 0;
  public zoom: number = 1.0;

  public minZoom: number = 0.15;
  public maxZoom: number = 3.0;

  constructor(panX = 100, panY = 100, zoom = 1.0) {
    this.panX = panX;
    this.panY = panY;
    this.zoom = zoom;
  }

  /** Converte coordenada do mundo do diagrama para pixel na tela */
  public worldToScreen(worldX: number, worldY: number): Point {
    return {
      x: worldX * this.zoom + this.panX,
      y: worldY * this.zoom + this.panY,
    };
  }

  /** Converte pixel da tela para coordenada no mundo do diagrama */
  public screenToWorld(screenX: number, screenY: number): Point {
    return {
      x: (screenX - this.panX) / this.zoom,
      y: (screenY - this.panY) / this.zoom,
    };
  }

  /** Aplica zoom mantendo o ponto do cursor fixo na mesma coordenada de mundo */
  public zoomAt(screenPoint: Point, zoomDelta: number): void {
    const oldZoom = this.zoom;
    const factor = Math.exp(zoomDelta);
    const newZoom = Math.max(this.minZoom, Math.min(this.maxZoom, oldZoom * factor));

    if (newZoom === oldZoom) return;

    // Ajusta o pan para que o ponto sob o mouse permaneça imóvel no mundo
    this.panX = screenPoint.x - (screenPoint.x - this.panX) * (newZoom / oldZoom);
    this.panY = screenPoint.y - (screenPoint.y - this.panY) * (newZoom / oldZoom);
    this.zoom = newZoom;
  }

  /** Desloca a câmera pelo delta especificado em pixels de tela */
  public panBy(deltaX: number, deltaY: number): void {
    this.panX += deltaX;
    this.panY += deltaY;
  }

  /** Aplica a transformação de câmera atual ao contexto 2D do Canvas */
  public applyToContext(ctx: CanvasRenderingContext2D, dpr: number): void {
    ctx.setTransform(1, 0, 0, 1, 0, 0); // Reseta matriz
    ctx.scale(dpr, dpr);
    ctx.translate(this.panX, this.panY);
    ctx.scale(this.zoom, this.zoom);
  }
}
