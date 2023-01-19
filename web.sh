rustup target add wasm32-unknown-unknown
cargo +nightly build -Z build-std=std,panic_abort --target wasm32-unknown-unknown --release

# Recommended way
cargo install --git https://github.com/static-web-server/static-web-server
cd target/wasm32-unknown-unknown/release

echo "<html lang=\"en\">
<head>
    <meta charset=\"utf-8\">
    <title>Tetris</title>
    <style>
        html,
        body,
        canvas {
            margin: 0px;
            padding: 0px;
            width: 100%;
            height: 100%;
            overflow: hidden;
            position: absolute;
            background: black;
            z-index: 0;
        }
    </style>
</head>
<body>
    <canvas id=\"glcanvas\" tabindex='1'></canvas>
    <!-- Minified and statically hosted version of https://github.com/not-fl3/macroquad/blob/master/js/mq_js_bundle.js -->
    <script src=\"https://not-fl3.github.io/miniquad-samples/mq_js_bundle.js\"></script>
    <script>load(\"tetris.wasm\");</script> <!-- Your compiled wasm file -->
</body>
</html>" > index.html
echo "Running application on http://localhost:8080"
static-web-server --port 8080 --root .