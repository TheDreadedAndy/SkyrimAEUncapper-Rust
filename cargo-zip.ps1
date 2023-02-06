mkdir -p data/SKSE/Plugins/
cp target/debug/SkyrimUncapper.dll data/SKSE/Plugins/
cp SkyrimUncapper.ini data/SKSE/Plugins/
7z a SkyrimUncapperAE.zip -tzip -r ./data
rm -r data
