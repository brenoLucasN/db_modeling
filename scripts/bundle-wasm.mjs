/**
 * Script utilitário para gerar o bundle Wasm embutido em Base64.
 * Permite que a Webview do VS Code e o Web CLI carregem o núcleo Rowan/CST
 * de forma 100% síncrona e autônoma, sem requisições HTTP e imune a restrições de CSP.
 */

import * as fs from 'fs';
import * as path from 'path';

const WASM_FILE = path.resolve('packages/canvas-web/src/wasm/dbml_wasm_bg.wasm');
const JS_BINDINGS = path.resolve('packages/canvas-web/src/wasm/dbml_wasm.js');
const OUT_DIR = path.resolve('vscode-extension/dist');

if (!fs.existsSync(WASM_FILE) || !fs.existsSync(JS_BINDINGS)) {
  console.error('Arquivos wasm não encontrados. Execute wasm-bindgen antes.');
  process.exit(1);
}

if (!fs.existsSync(OUT_DIR)) {
  fs.mkdirSync(OUT_DIR, { recursive: true });
}

const wasmBuffer = fs.readFileSync(WASM_FILE);
const wasmBase64 = wasmBuffer.toString('base64');
let jsCode = fs.readFileSync(JS_BINDINGS, 'utf-8');

// Remove os imports/exports ES module para transformar em IIFE compatível com window
jsCode = jsCode.replace(/export\s+\{([^}]+)\};?/g, '');
jsCode = jsCode.replace(/export\s+function\s+/g, 'function ');
jsCode = jsCode.replace(/\/\* @ts-self-types[^*]*\*\//g, '');

const bundleContent = `(function() {
  ${jsCode}

  // Decodifica o binário WebAssembly embutido em base64
  const base64Data = "${wasmBase64}";
  const binaryString = atob(base64Data);
  const bytes = new Uint8Array(binaryString.length);
  for (let i = 0; i < binaryString.length; i++) {
    bytes[i] = binaryString.charCodeAt(i);
  }

  // Inicializa sincronamente o Wasm
  const wasmInstance = initSync({ module: bytes });

  window.DbModelingParser = {
    parse_and_analyze: parse_and_analyze,
    apply_command: apply_command,
    format_dbml: format_dbml,
    compute_layout: compute_layout,
    wasmInstance: wasmInstance
  };
})();
`;

fs.writeFileSync(path.join(OUT_DIR, 'dbml-wasm.bundle.js'), bundleContent);
console.log(`✓ Gerado bundle Wasm embutido: vscode-extension/dist/dbml-wasm.bundle.js (${(bundleContent.length / 1024).toFixed(1)} KB)`);
