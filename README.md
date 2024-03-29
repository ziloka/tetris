Something similar to [four-tris](https://github.com/fiorescarlatto/four-tris)

# Notes

The coordinate system is in the [center of the screen](https://bevy-cheatbook.github.io/features/coords.html?highlight=coordinate#2d-and-3d-scenes-and-cameras)

Everything is drawn every frame

Press ESC to open settings to change the controls
Press F3 if you want to see debugging information

Tetris Guideline spefications are followed as closely as possible
They are available via [here](https://tetris.fandom.com/wiki/Tetris_Guideline)

[Logging](https://www.reddit.com/r/learnrust/comments/16jde4t/how_do_i_add_a_debug_mode_without_tons_of_if/)

# Contributing

During development run 
```
cargo install cargo-watch
cargo watch -x run
```
to achieve hot code reloading

Use [Act](https://github.com/nektos/act)
To test github actions
```sh
docker build --no-cache --tag ziloka/tetris:act-20.04 .
# docker scout quickview
act --platform ubuntu=latest=ziloka/tetris:act-20.04 --pull=false --artifact-server-path tmp/artifacts
# act --platform ubuntu-latest=ghcr.io/catthehacker/ubuntu:rust-22.04 --artifact-server-path tmp/artifacts
```

# Inspiration taken from

https://github.com/vlang/v/tree/master/examples/tetris

https://github.com/corbamico/bevy-tetris

https://github.com/zigurous/unity-tetris-tutorial

https://github.com/JohnnyTurbo/LD43

# Wallkicks

https://www.youtube.com/watch?v=yIpk5TJ_uaI

After ~37 minutes 
On windows the memory goes from 50mb to 20~15mb


https://github.com/thesilican/tetris-ai