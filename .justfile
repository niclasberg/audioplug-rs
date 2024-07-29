set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

dylib_ext := if os_family() == "windows" { ".dll" } else { ".dylib" }
staticlib_ext := if os_family() == "windows" { ".lib" } else { ".a" }

build target="gain": && (_bundle-vst target "target/debug")
    cargo build --package {{target}}

[windows]
_bundle-vst name target_dir:
    md {{target_dir}}/{{name}}/Contents/Resources -Force
    md {{target_dir}}/{{name}}/Contents/x86_64-win -Force
    cp target/debug/{{name}}.dll {{target_dir}}/{{name}}/Contents/x86_64-win/{{name}}.vst3

[macos]
_bundle-vst name target_dir:
    mkdir -p {{target_dir}}/{{name}}/Contents/Resources
    mkdir -p {{target_dir}}/{{name}}/Contents/MacOS
    cp target/debug/{{name}}.dylib {{target_dir}}/{{name}}/Contents/MacOS/gain.vst3
