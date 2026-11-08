/**
 * Implementações da camada de transporte para comunicação desacoplada do Canvas
 * com a Webview do VS Code e com o servidor Axum CLI via WebSocket.
 */

import {
  HostTransport,
  TextPatch,
  SidecarLayout,
  Diagnostic,
  WorkspaceFileEntry,
} from './types';

declare function acquireVsCodeApi(): {
  postMessage(message: unknown): void;
  getState(): unknown;
  setState(state: unknown): void;
};

let cachedVsCodeApi: ReturnType<typeof acquireVsCodeApi> | null = null;
function getVsCodeApi(): ReturnType<typeof acquireVsCodeApi> {
  if (cachedVsCodeApi) return cachedVsCodeApi;
  cachedVsCodeApi =
    typeof acquireVsCodeApi === 'function'
      ? acquireVsCodeApi()
      : {
          postMessage: (msg: unknown) => console.log('[Mock VS Code postMessage]', msg),
          getState: () => null,
          setState: () => {},
        };
  return cachedVsCodeApi;
}

/** Transporte para execução dentro da Webview do VS Code */
export class VsCodeTransport implements HostTransport {
  private vscode: ReturnType<typeof acquireVsCodeApi>;
  private updateCallback?: (
    source: string,
    layout?: SidecarLayout,
    meta?: { title?: string; files?: WorkspaceFileEntry[] }
  ) => void;
  private workspaceFilesCallback?: (
    files: WorkspaceFileEntry[],
    activeTitle?: string
  ) => void;
  private externalChangeCallback?: (newSource: string) => void;
  private errorCallback?: (diagnostics: Diagnostic[]) => void;

  constructor() {
    this.vscode = getVsCodeApi();

    window.addEventListener('message', (event) => {
      const message = event.data;
      if (!message) return;
      switch (message.type) {
        case 'update':
          if (this.updateCallback) {
            this.updateCallback(message.source, message.layout, {
              title: message.title,
              files: message.files,
            });
          }
          if (message.files && this.workspaceFilesCallback) {
            this.workspaceFilesCallback(message.files, message.title);
          }
          break;
        case 'workspace_files':
          if (this.workspaceFilesCallback) {
            this.workspaceFilesCallback(message.files || [], message.title);
          }
          break;
        case 'externalChange':
          if (this.externalChangeCallback) {
            this.externalChangeCallback(message.source);
          }
          break;
        case 'diagnostics':
          if (this.errorCallback) {
            this.errorCallback(message.diagnostics);
          }
          break;
      }
    });
  }

  public async sendPatch(patch: TextPatch): Promise<void> {
    this.vscode.postMessage({
      command: 'apply_patch',
      patch,
    });
  }

  public async saveLayout(layout: SidecarLayout): Promise<void> {
    this.vscode.postMessage({
      command: 'save_layout',
      layout,
    });
  }

  public async exportFile(filename: string, content: string, isBinary: boolean): Promise<void> {
    this.vscode.postMessage({
      command: 'export_file',
      filename,
      content,
      isBinary,
    });
  }

  public async openFile(path: string): Promise<void> {
    this.vscode.postMessage({
      command: 'open_file',
      path,
    });
  }

  /** Avisa o host que a Webview está montada e pronta para receber o documento. */
  public ready(): void {
    this.vscode.postMessage({ command: 'ready' });
  }

  /** Publica os diagnósticos do Wasm na aba nativa Problems do VS Code. */
  public sendDiagnostics(diagnostics: Diagnostic[]): void {
    this.vscode.postMessage({ command: 'diagnostics', diagnostics });
  }

  public async createNewFile(): Promise<void> {
    this.vscode.postMessage({
      command: 'new_file',
    });
  }

  public onDocumentUpdated(
    callback: (
      source: string,
      layout?: SidecarLayout,
      meta?: { title?: string; files?: WorkspaceFileEntry[] }
    ) => void
  ): void {
    this.updateCallback = callback;
  }

  public onWorkspaceFilesUpdated(
    callback: (files: WorkspaceFileEntry[], activeTitle?: string) => void
  ): void {
    this.workspaceFilesCallback = callback;
  }

  public onFileChangedExternally(callback: (newSource: string) => void): void {
    this.externalChangeCallback = callback;
  }

  public onError(callback: (diagnostics: Diagnostic[]) => void): void {
    this.errorCallback = callback;
  }
}

/** Transporte para execução dentro do navegador conectado ao servidor Axum CLI */
export class WebSocketTransport implements HostTransport {
  private ws: WebSocket | null = null;
  private url: string;
  private updateCallback?: (
    source: string,
    layout?: SidecarLayout,
    meta?: { title?: string; files?: WorkspaceFileEntry[] }
  ) => void;
  private externalChangeCallback?: (newSource: string) => void;
  private reconnectTimer: number | null = null;

  constructor(url = `ws://${window.location.host}/ws`) {
    this.url = url;
    this.connect();
  }

  private connect(): void {
    try {
      this.ws = new WebSocket(this.url);

      this.ws.onmessage = (event) => {
        try {
          const msg = JSON.parse(event.data);
          if (msg.type === 'init' || msg.type === 'update') {
            if (this.updateCallback) {
              this.updateCallback(msg.source, msg.layout, {
                title: msg.title,
                files: msg.files,
              });
            }
          } else if (msg.type === 'external_change') {
            if (this.externalChangeCallback) {
              this.externalChangeCallback(msg.source);
            }
          }
        } catch (e) {
          console.error('Erro ao processar mensagem WebSocket:', e);
        }
      };

      this.ws.onclose = () => {
        if (!this.reconnectTimer) {
          this.reconnectTimer = window.setTimeout(() => {
            this.reconnectTimer = null;
            this.connect();
          }, 2000);
        }
      };
    } catch (e) {
      console.warn('Falha ao conectar WebSocket:', e);
    }
  }

  public async sendPatch(patch: TextPatch): Promise<void> {
    if (this.ws && this.ws.readyState === WebSocket.OPEN) {
      this.ws.send(
        JSON.stringify({
          type: 'apply_patch',
          payload: {
            byte_start: patch.byte_start,
            byte_end: patch.byte_end,
            new_text: patch.new_text,
          },
        })
      );
    }
  }

  public async saveLayout(layout: SidecarLayout): Promise<void> {
    if (this.ws && this.ws.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify({ type: 'save_layout', payload: layout }));
    }
  }

  public async exportFile(filename: string, content: string, isBinary: boolean): Promise<void> {
    let blob: Blob;
    if (isBinary) {
      const binaryString = atob(content);
      const bytes = new Uint8Array(binaryString.length);
      for (let i = 0; i < binaryString.length; i++) {
        bytes[i] = binaryString.charCodeAt(i);
      }
      const mime =
        filename.endsWith('.jpg') || filename.endsWith('.jpeg') ? 'image/jpeg' : 'image/png';
      blob = new Blob([bytes], { type: mime });
    } else {
      const mime = filename.endsWith('.svg')
        ? 'image/svg+xml;charset=utf-8'
        : 'text/plain;charset=utf-8';
      blob = new Blob([content], { type: mime });
    }
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = filename;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  }

  public onDocumentUpdated(
    callback: (
      source: string,
      layout?: SidecarLayout,
      meta?: { title?: string; files?: WorkspaceFileEntry[] }
    ) => void
  ): void {
    this.updateCallback = callback;
  }

  public onFileChangedExternally(callback: (newSource: string) => void): void {
    this.externalChangeCallback = callback;
  }
}
