rustup target add wasm32-unknown-unknown
rustup toolchain install nightly
rustup component add rust-src --toolchain nightly
cargo +nightly build -Z build-std=std,panic_abort --target wasm32-unknown-unknown --release

cd target/wasm32-unknown-unknown/release

echo "<html lang=\"en\">
<head>
    <meta charset=\"utf-8\">
    
    <!-- Force browser to redownload file https://github.com/orgs/community/discussions/19713#discussioncomment-3051290 -->
    <meta http-equiv='cache-control' content='no-cache'> 
    <meta http-equiv='expires' content='0'> 
    <meta http-equiv='pragma' content='no-cache'>

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
    <!-- Disable context menu https://stackoverflow.com/questions/737022/how-do-i-disable-right-click-on-my-web-page -->
    <canvas id=\"glcanvas\" tabindex='1' oncontextmenu=\"return false\"></canvas>
    <!-- Minified and statically hosted version of https://github.com/not-fl3/macroquad/blob/master/js/mq_js_bundle.js -->
    <script src=\"https://not-fl3.github.io/miniquad-samples/mq_js_bundle.js\"></script>
    <script>load(\"client.wasm\");</script> <!-- Your compiled wasm file -->
</body>
</html>" > index.html

echo "[advanced]

[[advanced.headers]]
source = \"**/*.{wasm,jpg,jpeg,png,ico,gif}\"
headers.Cache-Control = \"max-age=0;\"
headers.Strict-Transport-Security = \"max-age=0; includeSubDomains; preload\"" > config.toml

# Recommended way
if [ "$1" == "online" ]
then
    caddy file-server --listen :8080
    # caddy file-server --browse --listen :8080
fi