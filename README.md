# Wayland Webkit Widget

a widget system allows you using html, javascript and css to create something on your screen :3.

a youtube demonstration video in below:

[![Thumbnail](http://i3.ytimg.com/vi/TyJTUVT9ZWs/maxresdefault.jpg)](https://www.youtube.com/watch?v=TyJTUVT9ZWs)

# Initiatives

1. I want creating widget in HTML, CSS and JavaScript although it might be bloat. :3
2. I want a larger eco-system I guess? ([three.js](https://threejs.org/), [gsap](https://gsap.com/), [lottie](https://airbnb.design/lottie/)... etc)

# Goals

- [x] able to show sth on screen :3
- [x] put widget in background or always on top :3
- [x] enable click through & exclusive zone
- [ ] allow webview calls system command
- [ ] allow dispatching sth to specific webview via command line
- [ ] publish

# Usage

## Start daemon

```sh
cargo run init
```

## Load config

```sh
cargo run create
   --directory <a path relative to ~/.config/www/> # lets say you have a index.html located at ~/.config/www/my-app/index.html, the path would be my-app
   --monitor <the index of your monitor> # use cargo run version to obtain the information of the indices
   --layer <overlay, top, bottom, background> # overlay will visble even when fullscreen
   --anchors <top, right, bottom, left> # you need to specify anchors to show the application
   --default-width <number>
   --default-height <number>
```

## Show widget

```sh
cargo run show
# --- your could show by id
cargo run show --id ffffff
# --- also by tags
cargo run show --tags wallpaper
# --- or by directory
cargo run show --directory my-app
```

## Hide widget

```sh
cargo run hide
   # --id <id>
   # --directory <path>
   # --tags <tag>
```

## Unload widget

```sh
cargo run delete
   # --id <id>
   # --directory <path>
   # --tags <tag>
```

## Reload widget

```sh
cargo run reload
   # --id <id>
   # --directory <path>
   # --tags <tag>
```

## Kill daemon

```sh
cargo run kill
```
