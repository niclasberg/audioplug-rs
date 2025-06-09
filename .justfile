set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

dylib_ext := if os_family() == "windows" { ".dll" } else { ".dylib" }
staticlib_ext := if os_family() == "windows" { ".lib" } else { ".a" }

build_all: (build "gain") (build "synth")

build target="gain" target_dir="target/debug": (_bundle_vst target target_dir) (_bundle_au target target_dir) 

_build target="gain":
    cargo build --package {{target}}

[windows]
_bundle_vst name target_dir: (_build name)
    md {{target_dir}}/{{name}}.vst3/Contents/Resources -Force
    md {{target_dir}}/{{name}}.vst3/Contents/x86_64-win -Force
    cp target/debug/{{name}}.dll {{target_dir}}/{{name}}.vst3/Contents/x86_64-win/{{name}}.vst3
    cp target/debug/{{name}}.pdb {{target_dir}}/{{name}}.vst3/Contents/x86_64-win/{{name}}.pdb

[macos]
_bundle_vst name target_dir: (_build name)
    mkdir -p {{target_dir}}/{{name}}.vst3/Contents/Resources
    mkdir -p {{target_dir}}/{{name}}.vst3/Contents/MacOS
    cp target/debug/lib{{name}}.dylib {{target_dir}}/{{name}}.vst3/Contents/MacOS/{{name}}
    @echo "<?xml version=\"1.0\" encoding=\"UTF-8\"?> \
    <!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\"> \
    <plist version=\"1.0\"> \
    <dict> \
        <key>CFBundleDevelopmentRegion</key> \
        <string>English</string> \
        <key>CFBundleExecutable</key> \
        <string>{{name}}</string> \
        <key>CFBundleGetInfoString</key> \
        <string>vst3</string> \
        <key>CFBundleIconFile</key> \
        <string></string> \
        <key>CFBundleIdentifier</key> \
        <string>com.github.niclasberg.{{name}}</string> \
        <key>CFBundleInfoDictionaryVersion</key> \
        <string>6.0</string> \
        <key>CFBundleName</key> \
        <string>{{name}}</string> \
        <key>CFBundlePackageType</key> \
        <string>BNDL</string> \
        <key>CFBundleVersion</key> \
        <string>1.0</string> \
        <key>CSResourcesFileMapped</key> \
        <string></string> \
    </dict> \
    </plist>" > {{target_dir}}/{{name}}.vst3/Contents/Info.plist
    codesign --force --sign - --timestamp=none {{target_dir}}/{{name}}.vst3

[macos]
_bundle_au name target_dir: (_build name)
    mkdir -p {{target_dir}}/tmp
    mkdir -p "./target/debug/rusttest.app/Contents/PlugIns/{{name}}.appex/Contents/MacOS"
    cp target/debug/lib{{name}}.a {{target_dir}}/tmp/libaudioplug.a
    clang++ -o "./target/debug/rusttest.app/Contents/PlugIns/{{name}}.appex/Contents/MacOS/{{name}}" -Wl,-no_adhoc_codesign -fobjc-arc -fobjc-link-runtime -fapplication-extension -e _NSExtensionMain -fmodules -framework Foundation -framework AudioToolbox -framework AppKit -framework CoreGraphics -framework CoreText -framework CoreAudioKit -L{{target_dir}}/tmp objc/view_controller.mm -laudioplug
    cp examples/{{name}}/AU/Info.plist {{target_dir}}/rusttest.app/Contents/PlugIns/{{name}}.appex/Contents/
    codesign --force --sign - -o runtime --entitlements ./examples/{{name}}/AU/entitlements.plist --timestamp=none "./target/debug/rusttest.app/Contents/PlugIns/{{name}}.appex"
    codesign --force --sign - --timestamp=none "./target/debug/rusttest.app"

[windows]
_bundle_au name target_dir:

[windows]
_build_shader shader entry:
    fxc.exe /T lib_5_0 src\platform\win\shaders/{{shader}}.hlsl /D D2D_FUNCTION /D D2D_ENTRY={{entry}} /I "C:\Program Files (x86)\Windows Kits\10\Include\10.0.22621.0\um" /Fl src\platform\win\shaders\{{shader}}.fxlib
    fxc.exe /T ps_5_0 src\platform\win\shaders/{{shader}}.hlsl /D D2D_FULL_SHADER /D D2D_ENTRY={{entry}} /E {{entry}} /setprivate src\platform\win\shaders\{{shader}}.fxlib /I "C:\Program Files (x86)\Windows Kits\10\Include\10.0.22621.0\um" /Fo src\platform\win\shaders/{{shader}}.cso

[windows]
build_shaders: \
    (_build_shader "rounded_rect_shadow" "RoundedShadowMain") \
    (_build_shader "rect_shadow" "RectShadowMain")