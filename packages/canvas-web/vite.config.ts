import { defineConfig } from 'vite';
import { resolve } from 'path';

export default defineConfig(({ mode }) => {
  if (mode === 'lib') {
    return {
      build: {
        lib: {
          entry: resolve(__dirname, 'src/index.ts'),
          name: 'DbModelingCanvasLib',
          fileName: (format) => (format === 'iife' ? 'index.js' : `index.${format}.js`),
          formats: ['iife', 'es', 'cjs']
        },
        rollupOptions: {
          output: {
            exports: 'named'
          }
        }
      }
    };
  }

  return {
    root: '.',
    build: {
      outDir: 'dist-app',
      target: 'esnext'
    }
  };
});
