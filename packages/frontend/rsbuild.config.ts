import { paraglideRspackPlugin } from '@inlang/paraglide-js';
import { defineConfig } from '@rsbuild/core';
import { pluginReact } from '@rsbuild/plugin-react';
import { tanstackRouter } from '@tanstack/router-plugin/rspack';

export default defineConfig({
  plugins: [pluginReact()],
  tools: {
    rspack: {
      plugins: [
        tanstackRouter({
          target: 'react',
          autoCodeSplitting: true,
        }),
        paraglideRspackPlugin({
          project: './project.inlang',
          outdir: './src/paraglide',
        }),
      ],
    },
  },
});
