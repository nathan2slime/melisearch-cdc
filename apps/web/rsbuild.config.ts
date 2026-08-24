import { defineConfig, loadEnv } from "@rsbuild/core";
import { pluginReact } from "@rsbuild/plugin-react";
import { pluginTailwindcss } from "@rsbuild/plugin-tailwindcss";
import { tanstackRouter } from "@tanstack/router-plugin/rspack";
import path from "node:path";

const { parsed, publicVars } = loadEnv({
  cwd: path.join(process.cwd(), "..", ".."),
  prefixes: ["REACT_APP_PUBLIC_"],
});

const apiProxyTarget = `http://localhost:${parsed.PORT ?? process.env.PORT ?? "5400"}`;

export default defineConfig({
  html: {
    title: "Melisearch",
  },
  plugins: [pluginReact(), pluginTailwindcss()],
  source: {
    define: publicVars,
  },
  server: {
    proxy: {
      "/api": apiProxyTarget,
    },
  },
  tools: {
    rspack: {
      devtool: "source-map",
      plugins: [
        tanstackRouter({
          target: "react",
          autoCodeSplitting: true,
        }),
      ],
    },
  },
});
