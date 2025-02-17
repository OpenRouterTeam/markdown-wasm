import type webpack from "webpack";
import path from "path";
import WasmPackPlugin from "@wasm-tool/wasm-pack-plugin";
import BundleDeclarationsWebpackPlugin from "bundle-declarations-webpack-plugin";

// Needed to use build-std on stable rust
process.env.RUSTC_BOOTSTRAP = "1";

export default {
  mode: "production",

  entry: "./src/index.ts",
  output: {
    path: path.resolve(__dirname, "dist"),
    filename: "index.mjs",
    clean: true,

    library: {
      type: "module",
    },
    wasmLoading: "fetch",
  },

  module: {
    rules: [
      {
        test: /\.tsx?$/,
        use: "ts-loader",
        exclude: /node_modules/,
      },
    ],
  },

  plugins: [
    new WasmPackPlugin({
      crateDirectory: path.resolve(__dirname),
      outDir: "build",
      extraArgs:
        "--no-pack --target bundler . -Z build-std=panic_abort,std -Z build-std-features=panic_immediate_abort",
    }),
    new BundleDeclarationsWebpackPlugin(),
  ],

  experiments: {
    outputModule: true,
    asyncWebAssembly: true,
  },
} satisfies webpack.Configuration;
