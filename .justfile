set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

dylib_ext := if os_family() == "windows" { ".dll" } else { ".dylib" }
staticlib_ext := if os_family() == "windows" { ".lib" } else { ".a" }

build_all: (build "gain") (build "synth")

build target="gain" target_dir="target/debug": (_bundle_vst target target_dir) (_bundle_au target target_dir) 

[macos]
_build target="gain":
    AUDIOPLUG_OBJC_NAMESPACE="AudioPlug_{{target}}" cargo build --package {{target}}

[windows, linux]
_build target="gain":
    cargo build --package {{target}}

[windows]
_bundle_vst name target_dir: (_build name)
    md {{target_dir}}/{{name}}.vst3/Contents/Resources -Force
    md {{target_dir}}/{{name}}.vst3/Contents/x86_64-win -Force
    cp target/debug/{{name}}.dll {{target_dir}}/{{name}}.vst3/Contents/x86_64-win/{{name}}.vst3
    cp target/debug/{{name}}.pdb {{target_dir}}/{{name}}.vst3/Contents/x86_64-win/{{name}}.pdb

[linux]
_bundle_vst name target_dir: (_build name)
    mkdir -p {{target_dir}}/{{name}}.vst3/Contents/Resources
    mkdir -p {{target_dir}}/{{name}}.vst3/Contents/x86_64-linux
    cp target/debug/lib{{name}}.so {{target_dir}}/{{name}}.vst3/Contents/x86_64-linux/{{name}}.so
    #cp target/debug/{{name}}.pdb {{target_dir}}/{{name}}.vst3/Contents/x86_64-win/{{name}}.pdb

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
    mkdir -p "./target/debug/{{name}}.app/Contents/PlugIns/{{name}}.appex/Contents/MacOS"
    mkdir -p "./target/debug/{{name}}.app/Contents/MacOS"
    clang -framework Cocoa -o ./target/debug/{{name}}.app/Contents/MacOS/{{name}} ./objc/dummy_app.mm
    @echo "<?xml version=\"1.0\" encoding=\"UTF-8\"?> \
        <!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\"> \
        <plist version="1.0"> \
        <dict> \
            <key>CFBundleExecutable</key> \
            <string>{{name}}</string> \
            <key>CFBundleIdentifier</key> \
            <string>com.github.niclasberg.{{name}}.hostapp</string> \
            <key>CFBundleName</key> \
            <string>{{name}}</string> \
            <key>CFBundlePackageType</key> \
            <string>APPL</string> \
            <key>CFBundleVersion</key> \
            <string>1</string> \
            <key>CFBundleShortVersionString</key> \
            <string>1.0</string> \
        </dict> \
        </plist>" > "./target/debug/{{name}}.app/Contents/Info.plist"
    cp target/debug/lib{{name}}.a {{target_dir}}/tmp/libaudioplug.a
    clang++ -o "./target/debug/{{name}}.app/Contents/PlugIns/{{name}}.appex/Contents/MacOS/{{name}}" \
        -Wl,-no_adhoc_codesign -fobjc-arc -fobjc-link-runtime -fapplication-extension -e _NSExtensionMain -fmodules \
        -framework Foundation -framework AudioToolbox -framework AppKit -framework CoreGraphics -framework Metal -framework CoreText -framework CoreAudioKit \
        -L{{target_dir}}/tmp objc/view_controller.mm -laudioplug -DAUDIOPLUG_VIEW_CONTROLLER_NAME="AudioPlug_{{name}}_ViewController"
    cp examples/{{name}}/AU/Info.plist {{target_dir}}/{{name}}.app/Contents/PlugIns/{{name}}.appex/Contents/

    codesign --force --sign - -o runtime --entitlements ./examples/{{name}}/AU/entitlements.plist --timestamp=none "./target/debug/{{name}}.app/Contents/PlugIns/{{name}}.appex"
    codesign --force --sign - --timestamp=none "./target/debug/{{name}}.app"

[windows, linux]
_bundle_au name target_dir:
