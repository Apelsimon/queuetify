const path = require('path');

module.exports = {
  entry: {
    index: [path.resolve(__dirname, "src", "index.ts")],
    session: [path.resolve(__dirname, "src", "session.ts")],
    websocket: [path.resolve(__dirname, "src", "websocket.ts")]
  },
  mode: "development",
  devServer: {
      watchFiles: ["src/**/*"],
  },
  devtool: 'eval-source-map',
  module: {
    rules: [
      {
        test: /\.tsx?$/,
        use: 'ts-loader',
        exclude: /node_modules/,
      },
      {
          test: /\.css$/i,
          include: path.resolve(__dirname, "src"),
          use: ["style-loader", "css-loader", "postcss-loader"],
      }
    ],
  },
  resolve: {
    extensions: ['.tsx', '.ts', '.js'],
  },
  output: {
    filename: '[name].js',
    path: path.resolve(__dirname, 'public'),
    clean: true,
  },
};