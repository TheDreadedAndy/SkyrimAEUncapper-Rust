/**
 * @file cargo-zip.c
 * @author Andrew Spaulding (Kasplat)
 * @brief Simple extension to cargo to generate zip files.
 * @bug No known bugs.
 */

#include <stdlib.h>

int main(void) {
    return system(
        "powershell.exe -NoProfile -Command \".\\cargo-zip.ps1\""
    );
}
