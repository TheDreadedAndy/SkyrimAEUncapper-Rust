/**
 * @file cargo-zip.c
 * @author Andrew Spaulding (Kasplat)
 * @brief Simple extension to cargo to generate zip files.
 * @bug No known bugs.
 */

#include <stdlib.h>

const char *const commands =
    "powershell.exe -NoProfile -Command \""
    "cargo build;"
    "cargo build --release;"
    "mkdir -p data/SKSE/Plugins/;"
    "cp target/debug/SkyrimUncapper.dll data/SKSE/Plugins/;"
    "cp SkyrimUncapper/SkyrimUncapper.ini data/SKSE/Plugins/;"
    "7z a SkyrimUncapperAE-Debug.zip -tzip -r ./data;"
    "cp target/release/SkyrimUncapper.dll data/SKSE/Plugins/;"
    "7z a SkyrimUncapperAE.zip -tzip -r ./data;"
    "rm -r data\"";

int main(void) {
    return system(commands);
}
