const path = require('path');
const { CleanWebpackPlugin } = require('clean-webpack-plugin');

module.exports = (env, argv) => {
  const isProduction = argv.mode === 'production';
  
  return {
    entry: {
      main: './frontend/src/main.js'
    },
    
    output: {
      path: path.resolve(__dirname, 'static/dist'),
      filename: '[name].bundle.js',
      chunkFilename: '[name].chunk.js',
      clean: true,
      publicPath: '/static/dist/'
    },
    
    optimization: {
      splitChunks: {
        chunks: 'all',
        cacheGroups: {
          starknetWallet: {
            test: /[\\/]node_modules[\\/]@starknet-io[\\/]/,
            name: 'starknet-wallet',
            chunks: 'async',
            priority: 30
          },
          starknetCore: {
            test: /[\\/]node_modules[\\/]starknet[\\/]/,
            name: 'starknet-core',
            chunks: 'async',
            priority: 25
          },
          vendor: {
            test: /[\\/]node_modules[\\/](?!(@starknet-io|starknet)[\\/])/,
            name: 'vendor',
            chunks: 'initial',
            priority: 10
          }
        }
      }
    },
    
    module: {
      rules: [
        {
          test: /\.js$/,
          exclude: /node_modules/,
          use: {
            loader: 'babel-loader',
            options: {
              presets: [
                ['@babel/preset-env', {
                  targets: {
                    browsers: ['> 1%', 'last 2 versions']
                  }
                }]
              ]
            }
          }
        }
      ]
    },
    
    plugins: [
      new CleanWebpackPlugin()
    ],
    
    devtool: isProduction ? 'source-map' : 'eval-source-map',
    
    resolve: {
      extensions: ['.js', '.json']
    },
    
    stats: {
      chunks: true,
      chunkModules: false,
      colors: true,
      modules: false
    }
  };
};