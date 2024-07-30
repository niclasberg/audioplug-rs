#!/bin/bash -e 
# Make sure we have the arguments we need
if [[ -z $1 || -z $2 ]]; then
    echo "Generates a macOS au bundle from a compiled dylib file"
    echo "Example:"
    echo -e "\t$0 Plugin target/release/plugin.dylib"
    echo -e "\tCreates a Plugin.appex bundle"
else
    # Make the bundle folder
    mkdir -p "target/debug/$1.appex/Contents/MacOS"
	mkdir -p "target/debug/$1.appex/Contents/Resources"

	# Build 
	mkdir -p "target/debug/tmp"
	cp $2 "target/debug/tmp/libaudioplug.a"
	clang++ -o "./target/debug/rusttest.app/Contents/PlugIns/$1.appex/Contents/MacOS/$1" -Wl,-no_adhoc_codesign -fobjc-arc -fobjc-link-runtime -fapplication-extension -e _NSExtensionMain -fmodules -framework Foundation -framework AudioToolbox -framework AppKit -framework CoreGraphics -framework CoreText -framework CoreAudioKit -L "./target/debug/tmp" objc/view_controller.mm -laudioplug

    # Create the PkgInfo
    # echo "BNDL????" > "target/debug/$1.appex/Contents/PkgInfo"

    #build the Info.Plist
    echo "<?xml version=\"1.0\" encoding=\"UTF-8\"?>
<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">
<plist version=\"1.0\">
<dict>
	<key>CFBundleDevelopmentRegion</key>
	<string>en</string>
	<key>CFBundleDisplayName</key>
	<string>InstrumentDemoAppExtensionOSX</string>
	<key>CFBundleExecutable</key>
	<string>$1</string>
	<key>CFBundleIconFile</key>
	<string>icon</string>
	<key>CFBundleIdentifier</key>
	<string>com.niclas.$1</string>
	<key>CFBundleInfoDictionaryVersion</key>
	<string>6.0</string>
	<key>CFBundleName</key>
	<string>$1</string>
	<key>CFBundlePackageType</key>
	<string>XPC!</string>
	<key>CFBundleShortVersionString</key>
	<string>1.6</string>
	<key>CFBundleVersion</key>
	<string>1</string>
	<key>NSExtension</key>
	<dict>
		<key>NSExtensionAttributes</key>
		<dict>
			<key>AudioComponents</key>
			<array>
				<dict>
					<key>description</key>
					<string>AUV3InstrumentDemo</string>
					<key>manufacturer</key>
					<string>Nibe</string>
					<key>name</key>
					<string>RUST TEST</string>
					<key>factory</key>
					<string>MyViewController</string>
					<key>sandboxSafe</key>
					<true/>
					<key>subtype</key>
					<string>demo</string>
					<key>tags</key>
					<array>
						<string>Effect</string>
					</array>
					<key>type</key>
					<string>aufx</string>
					<key>version</key>
					<integer>67072</integer>
				</dict>
			</array>
		</dict>
		<key>NSExtensionPointIdentifier</key>
		<string>com.apple.AudioUnit</string>
		<key>NSExtensionPrincipalClass</key>
		<string>MyViewController</string>
	</dict>
</dict>
</plist>" > "./target/debug/rusttest.app/Contents/PlugIns/$1.appex/Contents/Info.plist"
	codesign --force --sign - -o runtime --entitlements ./examples/gain/AU/entitlements.plist --timestamp=none "./target/debug/rusttest.app/Contents/PlugIns/$1.appex"

	codesign --force --sign - --timestamp=none "./target/debug/rusttest.app"
    echo "Created bundle target/debug/$1.appex"
fi