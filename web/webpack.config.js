const path = require("path");
const CopyPlugin = require("copy-webpack-plugin");
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");

const dist = path.resolve(__dirname, "dist");

module.exports = {
    mode: "development",
    entry: {
        index: "./js/index.js"
    },
    output: {
        path: dist,
        filename: "[name].js"
    },
    devServer: {
        static: {
            directory: dist,
        },
        compress: true,
        port: 9000,
        historyApiFallback: true
    },
    plugins: [
        new CopyPlugin({
            patterns: [
                path.resolve(__dirname, "static")
            ]
        }),

        new WasmPackPlugin({
            crateDirectory: __dirname,
        }),
    ],
    module: {
        rules: [
            {
                test: /\.s[ac]ss$/i,
                use: [
                    // Creates `style` nodes from JS strings
                    "style-loader",
                    // Translates CSS into CommonJS
                    "css-loader",
                    // Compiles Sass to CSS
                    "sass-loader",
                ],
            },
            {
                test: /\.wasm$/,
                type: 'webassembly/sync',
            }
        ],
    },
    experiments: {
        syncWebAssembly: true
    }
};
