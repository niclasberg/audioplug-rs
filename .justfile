set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

dylib_ext := if os_family() == "windows" { ".dll" } else { ".dylib" }
staticlib_ext := if os_family() == "windows" { ".lib" } else { ".a" }

build target="gain" target_dir="target/debug": (_bundle_au target target_dir) (_bundle_vst target target_dir)

_build target="gain":
    cargo build --package {{target}}

[windows]
_bundle_vst name target_dir: (_build name)
    md {{target_dir}}/{{name}}.vst3/Contents/Resources -Force
    md {{target_dir}}/{{name}}.vst3/Contents/x86_64-win -Force
    cp target/debug/{{name}}.dll {{target_dir}}/{{name}}.vst3/Contents/x86_64-win/{{name}}.vst3

[macos]
_bundle_vst name target_dir: (_build name)
    mkdir -p {{target_dir}}/{{name}}.vst3/Contents/Resources
    mkdir -p {{target_dir}}/{{name}}.vst3/Contents/MacOS
    cp target/debug/lib{{name}}.dylib {{target_dir}}/{{name}}.vst3/Contents/MacOS/{{name}}
    cp examples/{{name}}/vst3/Info.plist {{target_dir}}/{{name}}.vst3/Contents/Info.plist
    codesign --force --sign - --timestamp=none {{target_dir}}/{{name}}.vst3

[macos]
_bundle_au name target_dir: (_build name)
    mkdir -p {{target_dir}}/tmp
    cp target/debug/lib{{name}}.a {{target_dir}}/tmp/libaudioplug.a
    clang++ -o "./target/debug/rusttest.app/Contents/PlugIns/rusttest.appex/Contents/MacOS/rusttest" -Wl,-no_adhoc_codesign -fobjc-arc -fobjc-link-runtime -fapplication-extension -e _NSExtensionMain -fmodules -framework Foundation -framework AudioToolbox -framework AppKit -framework CoreGraphics -framework CoreText -framework CoreAudioKit -L{{target_dir}}/tmp objc/view_controller.mm -laudioplug
    cp examples/{{name}}/AU/Info.plist {{target_dir}}
    codesign --force --sign - -o runtime --entitlements ./examples/{{name}}/AU/entitlements.plist --timestamp=none "./target/debug/rusttest.app/Contents/PlugIns/rusttest.appex"
    codesign --force --sign - --timestamp=none "./target/debug/rusttest.app"

[windows]
_bundle_au name target_dir: