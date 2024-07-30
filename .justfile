set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

dylib_ext := if os_family() == "windows" { ".dll" } else { ".dylib" }
staticlib_ext := if os_family() == "windows" { ".lib" } else { ".a" }

build target="gain":
    cargo build --package {{target}}

[windows]
_bundle-vst name target_dir: (build name)
    md {{target_dir}}/{{name}}/Contents/Resources -Force
    md {{target_dir}}/{{name}}/Contents/x86_64-win -Force
    cp target/debug/{{name}}.dll {{target_dir}}/{{name}}/Contents/x86_64-win/{{name}}.vst3

[macos]
_bundle-vst name target_dir: (build name)
    mkdir -p {{target_dir}}/{{name}}.vst3/Contents/Resources
    mkdir -p {{target_dir}}/{{name}}.vst3/Contents/MacOS
    cp target/debug/{{name}}.dylib {{target_dir}}/{{name}}.vst3/Contents/MacOS/{{name}}.vst3
	cp examples/{{name}}/VST3/Info.plist {{target_dir}}/{{name}}/Contents/Info.plist

[macos]
_bundle_au name target_dir: (build name)
	mkdir -p {{target_dir}}/tmp
	cp target/debug/{{name}}.a {{target_dir}}/tmp/libaudioplug.a
	clang++ -o "./target/debug/rusttest.app/Contents/PlugIns/$1.appex/Contents/MacOS/$1" -Wl,-no_adhoc_codesign -fobjc-arc -fobjc-link-runtime -fapplication-extension -e _NSExtensionMain -fmodules -framework Foundation -framework AudioToolbox -framework AppKit -framework CoreGraphics -framework CoreText -framework CoreAudioKit -L{{target_dir}}/tmp objc/view_controller.mm -laudioplug
	cp examples/{{name}}/AU/Info.plist {{target_dir}}

build-all target="gain" target_dir="target/debug": 