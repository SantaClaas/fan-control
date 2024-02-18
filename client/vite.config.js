import { defineConfig } from "vite";
import solidPlugin from "vite-plugin-solid";
// import devtools from 'solid-devtools/vite';

export default defineConfig({
  plugins: [
    /*
    Uncomment the following line to enable solid-devtools.
    For more info see https://github.com/thetarnav/solid-devtools/tree/main/packages/extension#readme
    */
    // devtools(),
    solidPlugin(),
  ],
  server: {
    port: 3000,
    proxy: {
      // During development vite hosts the SPA and redirects API requests to the server
      // During production the SPA is hosted by the server
      "/api": {
        target: "http://localhost:4000",
      },
    },
  },
  build: {
    target: "esnext",
  },
});
