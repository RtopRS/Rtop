<!--suppress HtmlDeprecatedAttribute -->
<h1 align="center">
  Rtop
</h1>
<p align="center">
    <a href="https://www.rust-lang.org/">
        <img src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white" alt="Made with Rust">
    </a>
    <a href="https://github.com/RTopRS/Rtop">
        <img src="https://img.shields.io/badge/Git-F05032?style=for-the-badge&logo=git&logoColor=white" alt="Use git">
    </a>
    <br>
    <a href="https://github.com/RTopRS/Rtop/blob/main/LICENSE">
        <img src="https://img.shields.io/github/license/SquitchYT/RTop?style=for-the-badge" alt="License">
    </a>
    <a href="https://github.com/RTopRS/Rtop/stargazers">
        <img src="https://img.shields.io/github/stars/SquitchYT/RTop?style=for-the-badge" alt="Stars">
    </a>
</p>
<h3 align="center">
    <strong>Faster and better alternative to Vtop written in Rust.</strong>
</h3>

## Features
* Lightweight < 1MB
* Responsive UI
* Work on Linux and MacOS
* Easy to Use
* Designed for You
* Extensible with an [API](https://github.com/RTopRS/RtopDev)

## Downloads
### Crates.io
Rtop is available on [crates.io](https://crates.io/crates/rtop-rs/) You can download it with
```bash
cargo install rtop-rs
```

## Build manually
Start by cloning the repo:
```bash
git clone https://github.com/RTopRS/Rtop.git
```
**For the next step you need to have Rust and cargo installed on your PC, for that follow the [official documentation](https://www.rust-lang.org/tools/install).**

Now switch to project folder and compile a release:
```bash
cd RTop && cargo build --release
```

Your executable will be in the `target/release/` folder, it is named `rtop-rs`.

## Option file
You can customize Rtop as like you want!
First, create this file `~/.config/rtop/config`<br>
Then, paste it this config template:
```json
{
    "pages": [
        [
            "cpu_chart",
            "memory_chart",
            "process_list"
        ]
    ],
    "plugins": [
    ]
}
```

If you want to add a plugin, simply add this entry in the `plugins` key
```json
{
    "path": "/path/to/the/lib.so",
    "provided_widgets": [
        "foo",
        "bar"
    ]
}
```
Then, simply add some plugin's widget to one page like this
```json
[
    "foo",
    "bar"
]
```

The final result should look like this
```json
{
    "pages": [
        [
            "cpu_chart",
            "memory_chart",
            "process_list"
        ],
        [
            "foo",
            "bar"
        ]
    ],
    "plugins": [
        {
            "path": "/path/to/the/lib.so",
            "provided_widgets": [
                "foo",
                "bar"
            ]
        }
    ]
}
```
**Just remember, you can only put 4 widgets per page**

## Contributors
[<img width="45" src="https://avatars.githubusercontent.com/u/63391793?v=4" alt="SquitchYT">](https://github.com/SquitchYT)

## License
**[RTop](https://github.com/RTopRS/Rtop) | [Mozilla Public License 2.0](https://github.com/RTopRS/Rtop/blob/main/LICENSE)**