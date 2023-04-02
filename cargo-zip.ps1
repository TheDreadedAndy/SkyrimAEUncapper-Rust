cargo build
cargo build --release
mkdir -p data/SKSE/Plugins/
cp target/debug/SkyrimUncapper.dll data/SKSE/Plugins/
cp SkyrimUncapper/SkyrimUncapper.ini data/SKSE/Plugins/
7z a SkyrimUncapperAE-Debug.zip -tzip -mx9 -r ./data
cp target/release/SkyrimUncapper.dll data/SKSE/Plugins/
7z a SkyrimUncapperAE.zip -tzip -mx9 -r ./data
rm -r data
