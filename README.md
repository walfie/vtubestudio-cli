# vtubestudio-cli (`vts`)

CLI program for sending one-off requests to the [VTube Studio API]. It connects
to the websocket, authenticates, performs one or two other requests, and then
disconnects and exits.

The primary use case is to do infrequent actions such as triggering hotkeys or
registering custom parameters, without needing to establish a long-running
websocket connection.

For more complex tasks, you might want to look at client libraries like
[`vtubestudio-rs`] (the Rust library that this program uses under the hood).

[VTube Studio API]: https://github.com/DenchiSoft/VTubeStudio
[`vtubestudio-rs`]: https://github.com/walfie/vtubestudio-rs

## Download

Check the [releases page](https://github.com/walfie/vtubestudio-cli/releases)
to download prebuilt binaries for your platform.

Otherwise you can use `cargo install vtubestudio-cli` or build it manually from
this repo using `cargo build --release`.

## Initialization

`vts` reads auth token info from a JSON config file whose default location depends on platform.

To generate the config in the default location, you can run:

```sh
vts config init
```

This will register the plugin with the VTube Studio API (the user will get a
pop-up in the app asking for confirmation) and save the token for use in future
calls. The plugin name and developer name can be customized with
`--plugin-name` and `--developer-name`, respectively.

### Config file location

By default, the config file can be found at:

* Windows: `%USERPROFILE%\AppData\Roaming\walfie\vtubestudio-cli\config.json`
* macOS: `$HOME/Library/Application Support/com.github.walfie.vtubestudio-cli/config.json`
* Linux: `$XDG_CONFIG_DIR/vtubestudio-cli/config.json` or `$HOME/.config/vtubestudio-cli/config.json`

This path can be overridden by setting the `VTS_CONFIG` environment variable or
passing the `--config-file` flag.

You can also run `vts config path` to show the path to the config, or `vts
config show` to show the contents of the config file.

## Usage

### Hotkeys

* List hotkeys

    ```sh
    vts hotkeys list
    ```

* Trigger hotkey by id

    ```sh
    vts hotkeys trigger e50ef5139b114d63af342eb65072a5e3
    ```

* Trigger hotkey by name

    ```sh
    vts hotkeys trigger --name MyHotkeyName
    ```

### Artmeshes

* List artmeshes

    ```sh
    vts artmeshes list
    ```

* Tint artmesh (rainbow)

    ```sh
    vts artmeshes tint --rainbow --duration 5s --tag-contains shirt sleeves
    ```

    VTube Studio resets artmesh tints when the plugin disconnects. Since this
    CLI program normally disconnects immediately after executing commands
    (which would otherwise reset the tint), the `--duration` flag adds a delay
    afterwards, to keep the tint active.

* Tint artmesh (hex color)

    ```sh
    vts artmeshes tint --color ff0000 --duration 5s --tag-contains eye
    ```

    The hex color also supports alpha, so values like `ff0000aa` are also valid.

### Params

* Create parameter

    ```sh
    vts params create MyParameterName --default 0 --min 0 --max 50
    ```

* Inject parameter value

    ```sh
    vts params inject MyParameterName 5
    ```

* Get value of parameter

    ```sh
    vts params get MyParameterName
    ```

### Models

* List models

    ```sh
    vts models list
    ```

* Load model by ID

    ```sh
    vts models load 8caf15fa0c664f489873386e43835a7f
    ```

* Load model by name

    ```sh
    vts models load --name Akari
    ```

* Move model

    ```sh
    vts models move --relative --duration 0.5s --rotation 180
    ```

### Others

```sh
vts --help
```

