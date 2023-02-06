/**
 * @file native_wrappers.cpp
 * @author Andrew Spaulding (Kasplat)
 * @brief Thin wrappers around native game functions to catch any exceptions they throw.
 * @bug No known bugs.
 *
 * This file is necessary to prevent U.B. when crossing the FFI bound back into rust/asm code.
 */

// We're on x86-64, so whatever.
typedef unsigned int u32;
typedef unsigned short u16;
typedef float f32;

#define CATCH_UNWIND(body)\
do {\
    try {\
        body\
    } catch(...) {\
        handle_ffi_exception();\
    }\
} while(0)\

extern "C" {
    /* Native entry points */
    u16 (*get_level_entry)(void*);
    void *(*get_game_setting_entry)(void*, const char*);
    f32 (*player_avo_get_base_entry)(void*, int);
    f32 (*player_avo_get_current_entry)(void*, int);
    void (*player_avo_mod_base_entry)(void*, int, float);
    void (*player_avo_mod_current_entry)(void*, u32, int, f32);

    /* ASM wrappers */
    f32 player_avo_get_current_original_wrapper(void*, int);

    /* Panic function */
    __declspec(noreturn) void handle_ffi_exception(void);

    /* Wrappers */

    u16
    get_level_net(
        void *player
    ) {
        CATCH_UNWIND(return get_level_entry(player););
    }

    void *
    get_game_setting_net(
        void *player,
        const char *setting
    ) {
        CATCH_UNWIND(return get_game_setting_entry(player, setting););
    }

    f32
    player_avo_get_base_net(
        void *av,
        int attr
    ) {
        CATCH_UNWIND(return player_avo_get_base_entry(av, attr););
    }

    f32
    player_avo_get_current_net(
        void *av,
        int attr,
        bool patch_en
    ) {
        CATCH_UNWIND(
            if (patch_en) {
                // Patch installed, so we need to use the wrapper.
                return player_avo_get_current_original_wrapper(av, attr);
            } else {
                // No patch, so we can just call the og function
                // (and must, since we don't have a trampoline).
                return player_avo_get_current_entry(av, attr);
            }
        );
    }

    void
    player_avo_mod_base_net(
        void *av,
        int attr,
        f32 delta
    ) {
        CATCH_UNWIND(player_avo_mod_base_entry(av, attr, delta););
    }

    void
    player_avo_mod_current_net(
        void *av,
        u32 unk1,
        int attr,
        f32 delta
    ) {
        CATCH_UNWIND(player_avo_mod_current_entry(av, unk1, attr, delta););
    }
}
