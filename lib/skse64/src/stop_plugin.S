/**
 * @file stop_plugin.S
 * @author Andrew Spaulding (Kasplat)
 * @brief ASM instruction to stop a SKSE plugin.
 * @bug No known bugs.
 */

// We include this file; no need to make this symbol global.
stop_plugin:
    ud2 // Throw a hardware exception. SKSE64 will catch it.
