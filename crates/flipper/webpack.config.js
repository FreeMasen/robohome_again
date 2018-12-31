const path = require('path');
module.exports = function(env) {
    let ret = {
        entry: {
            app: path.join(__dirname, 'ts', 'app.tsx'),
            auth: path.join(__dirname, 'ts', 'auth.js'),
        },
        output: {
            filename: '[name].js',
            path: path.join(__dirname, 'public', 'js'),
            chunkFilename: "[id].chunk.js",
            publicPath: '/js/'
        },
        resolve: {
            extensions: ['.ts', '.tsx', '.js', '.jsx', '.wasm']
        },
        module: {
            rules: [{
                test: /\.tsx?$/,
                use: 'awesome-typescript-loader'
            }]
        },
        devServer: {
            historyApiFallback: true,
            publicPath: '/js/',
            contentBase: path.join(__dirname, 'public'),
        },
    };

    if (env && typeof env === 'string' && env.startsWith('prod')) {
        ret.mode = 'production';
    } else {
        ret.mode = 'development';
        ret.devtool = 'source-map';
    }
    return ret;
}