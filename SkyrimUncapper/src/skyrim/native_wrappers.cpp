/**
 * @file native_wrappers.cpp
 * @author Andrew Spaulding (Kasplat)
 * @brief Thin wrappers around native game functions to catch any exceptions they throw.
 * @bug No known bugs.
 *
 * This file is necessary to prevent U.B. when crossing the FFI bound back into rust/asm code.
 */

#include <cstdint>

typedef uint64_t u64;
typedef uint32_t u32;
typedef uint16_t u16;
typedef uint8_t u8;
typedef float f32;

#define CATCH_UNWIND(body)\
do {\
    try {\
        body\
    } catch(...) {\
        handle_ffi_exception(__func__, sizeof(__func__) - 1);\
    }\
} while(0)\

extern "C" {
    /* Native entry points */
    extern u16 (*get_level_entry)(void*);
    extern f32 (*player_avo_get_base_entry)(void*, int);
    extern f32 (*player_avo_get_current_entry)(void*, int);
    extern void (*player_avo_mod_base_entry)(void*, int, float);
    extern void (*player_avo_mod_current_entry)(void*, u32, int, f32);

    /* ASM wrappers */
    extern f32 player_avo_get_current_original_wrapper_se(void*, int);
    extern f32 player_avo_get_current_original_wrapper_ae(void*, int);
    extern void improve_player_skill_points_original(
        void *, int, f32, u64, u32, u8, bool
    );

    /* Panic function */
    __declspec(noreturn) extern void handle_ffi_exception(const char *, size_t);

    /* Wrappers */

    u16
    get_level_net(
        void *player
    ) {
        CATCH_UNWIND(return get_level_entry(player););
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
        bool is_se,
        bool patch_en
    ) {
        CATCH_UNWIND(
            if (!patch_en) {
                // No patch, so we can just call the og function
                // (and must, since we don't have a trampoline).
                return player_avo_get_current_entry(av, attr);
            } else if (is_se) {
                // SE patch installed, so we need to use the wrapper.
                return player_avo_get_current_original_wrapper_se(av, attr);
            } else {
                // AE patch installed, so we need to use the wrapper.
                return player_avo_get_current_original_wrapper_ae(av, attr);
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

    void
    improve_player_skill_points_net(
        void *data,
        int attr,
        f32 exp,
        u64 unk1,
        u32 unk2,
        u8 unk3,
        bool unk4
    ) {
        CATCH_UNWIND(
            improve_player_skill_points_original(data, attr, exp, unk1, unk2, unk3, unk4);
        );
    }
}
