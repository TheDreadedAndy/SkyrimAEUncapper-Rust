/**
 * @file stop_plugin.S
 * @author Andrew Spaulding (Kasplat)
 * @brief ASM instruction to stop a SKSE plugin.
 * @bug No known bugs.
 */

.global SKSE64_Errors__stop_plugin__
SKSE64_Errors__stop_plugin__:
    ud2 // Throw a hardware exception. SKSE64 will catch it.
