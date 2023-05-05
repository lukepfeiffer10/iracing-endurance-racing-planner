const path = require("path");
const CopyPlugin = require("copy-webpack-plugin");
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");
const autoprefixer = require("autoprefixer");
const dist = path.resolve(__dirname, "dist");

module.exports = {
    mode: "development",
    entry: {
        index: ["./js/index.js", "./index.scss"]
    },
    output: {
        path: dist,
        filename: "[name].js",
        publicPath: "/"
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
        })
    ],
    module: {
        rules: [
            {
                test: /\.s[ac]ss$/i,
                use: [
                    {
                        loader: 'file-loader',
                        options: {
                            name: 'bundle.css',
                        },
                    },
                    { loader: 'extract-loader' },
                    {
                        loader: 'css-loader',
                        options: {
                            esModule: false
                        }
                    },
                    {
                        loader: 'postcss-loader',
                        options: {
                            postcssOptions: {
                                plugins: [
                                    autoprefixer()
                                ]
                            }
                        }
                    },
                    {
                        loader: 'sass-loader',
                        options: {
                            // Prefer Dart Sass
                            implementation: require('sass'),

                            webpackImporter: true,
                            sassOptions: {
                                includePaths: ['./node_modules']
                            }
                        },
                    }
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
