# crates-tui

A TUI viewer for crates.io

https://github.com/ratatui-org/crates-tui/assets/1813121/c84eaad7-4688-4ebb-91c0-683cc9a0abfe

## Logging

https://github.com/ratatui-org/crates-tui/assets/1813121/9609a0f1-4da7-426d-8ce8-2c5a77c54754

## Print Default Configuration

```plain
$ crates-tui --print-default-config

Config {
    data_home: "",
    config_home: "",
    log_level: Some(
        LevelFilter::DEBUG,
    ),
    tick_rate: 1.0,
    frame_rate: 15.0,
    background_color: Reset,
    search_query_outline_color: Reset,
    filter_query_outline_color: Reset,
    row_background_color_1: Reset,
    row_background_color_2: Reset,
}
```
