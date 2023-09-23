/**
 * @file HookWrappers.S
 * @author Andrew Spaulding (Kasplat)
 * @author Vadfromnu
 * @author Kassent
 * @brief Assembly hook entry points for each hook.
 * @bug No known bugs.
 *
 * ASSEMBLY IS ASSEMBLY.
 * ASSEMBLY BELONGS IN ASSEMBLY FILES.
 *
 * Note:
 * Within this file, some symbols will be surrended by curly braces. These
 * instances are Rust inline assembly syntax, and they are redirected to a
 * mangled symbol with the same (or a similar) name by the macro that includes
 * this asm file. We access these symbols like that to avoid publically
 * exporting them from the DLL file unnecessarily.
 *
 * Note:
 * x64 calling convention is weird, and it is very important that
 * it be obeyed (or else unwinding will cause UB). Notably, the stack
 * pointer must stay 16 byte alligned, which means any reg saving needs
 * to allocate stack space as 16 * n + 8. There must also be 32 bytes of
 * stack space for any called functions to use above the return address.
 * Another important point is that once an epilogue starts nothing else can
 * happen except for that epilogue (for unwinding).
 *
 * Note also that the compiler can sometimes optimize out the saving of
 * caller saved registers if it knows the function its calling wont trash
 * them. This means we really can't make any assumptions about reg saving,
 * unfortunately.
 */

// These are exposed to the native_wrappers C file, so we must mark them as
// globals. The other wrappers don't need to be global, since this file is
// directly included in source file that references them.
.global player_avo_get_current_original_wrapper_ae
.global player_avo_get_current_original_wrapper_se
.global update_skill_list_original_wrapper

// Saves all caller saved registers.
.macro SAVEALL is_jump
    .ifnb \is_jump
    sub $0x08, %rsp
    .endif
    push %rax
    push %rcx
    push %rdx
    push %r8
    push %r9
    push %r10
    push %r11
    sub $0x80, %rsp
    movdqu %xmm0, 0x20(%rsp)
    movdqu %xmm1, 0x30(%rsp)
    movdqu %xmm2, 0x40(%rsp)
    movdqu %xmm3, 0x50(%rsp)
    movdqu %xmm4, 0x60(%rsp)
    movdqu %xmm5, 0x70(%rsp)
.endm

// Restores all caller saved registers.
.macro RESTOREALL retgpr, retfpr, is_jump
    .ifb \retfpr
    movdqu 0x20(%rsp), %xmm0
    .endif
    movdqu 0x30(%rsp), %xmm1
    movdqu 0x40(%rsp), %xmm2
    movdqu 0x50(%rsp), %xmm3
    movdqu 0x60(%rsp), %xmm4
    movdqu 0x70(%rsp), %xmm5
    add $0x80, %rsp
    pop %r11
    pop %r10
    pop %r9
    pop %r8
    pop %rdx
    pop %rcx
    .ifb \retgpr
    pop %rax
    .else
    add $0x08, %rsp
    .endif
    .ifnb \is_jump
    add $0x08, %rsp
    .endif
.endm

/*
 * This function gets injected in the middle of another, so we must use the call
 * injection macros to protect the register state.
 */
skill_cap_patch_wrapper_ae:
    SAVEALL

    mov %edx, %ecx // We don't have an AVO arg.
    call {player_avo_get_base_unchecked} // Reimplement replaced call.
    movaps %xmm0, %xmm8 // Reimplement original move.
    mov %esi, %ecx // Move skill ID back into first arg.
    call {get_skill_cap_hook}
    movss %xmm0, %xmm10 // Replace maximum with fn result.
    movss %xmm8, %xmm0 // Restore skill level for comparison.

    RESTOREALL retfpr=1
    ret

skill_cap_patch_wrapper_se:
    SAVEALL

    mov %edx, %ecx // We don't have an AVO arg.
    call {player_avo_get_base_unchecked} // Reimplement replaced call.
    movaps %xmm0, %xmm6 // Reimplement og move.
    mov %esi, %ecx // Move skill ID back into first arg.
    call {get_skill_cap_hook}
    movss %xmm0, %xmm8  // Replace max with fn result.
    movss %xmm6, %xmm0  // Fix xmm0 out of paranoia.

    RESTOREALL retfpr=1
    ret

/*
 * Begins a max_charge calculation by changing the enchanting formula cap
 * to use the weapon cap if the enchanted item is offensive.
 */
max_charge_begin_wrapper_ae:
    SAVEALL

    // Reimplement replaced instruction.
    movss %xmm0, 0x160(%rsp)

    mov %r14d, %ecx // Enchantment type is in r14d.
    jmp _max_charge_begin_wrapper

max_charge_begin_wrapper_se:
    SAVEALL

    movss %xmm0, 0x180(%rsp) // Max charge is now at 0xc0(%rsp) in caller.

    mov %r15d, %ecx // Enchantment type is in r15d.

_max_charge_begin_wrapper:
    call {max_charge_begin_hook}

    RESTOREALL

    // I think this technically violates unwinding calling convention, but we
    // block unwinds below us anyway.
    test %rcx, %rcx // Reimplement replaced test.
    ret

/*
 * Ends a max_charge calculation by returning the skill formula equation
 * to using the magnitude formula cap, if necessary.
 */
max_charge_end_wrapper_ae:
    // Reimplement overwritten instructions.
    movss 0xa8(%rsp), %xmm0 // Get max_charge
    maxss %xmm9, %xmm0 // MAX(max_charge, 0)
    jmp _max_charge_end_wrapper

max_charge_end_wrapper_se:
    movss 0xc8(%rsp), %xmm0 // Max charge is now at 0xc0(%rsp) in caller.
    maxss %xmm8, %xmm0 // MAX(max_charge, 0)

_max_charge_end_wrapper:
    SAVEALL
    call {max_charge_end_hook}
    RESTOREALL
    ret

/*
 * Wraps our calculate_charge_points_per_use() function. We use this wrapper
 * to get access to our callers max_charge value.
 *
 * Note that rax is smashed in the code we overwrite, so we're safe to trash
 * it here.
 */
calculate_charge_points_per_use_wrapper_ae:
    lea 0xa8(%rsp), %rax
    jmp _calculate_charge_points_per_use_wrapper

calculate_charge_points_per_use_wrapper_se:
    lea 0xc8(%rsp), %rax

_calculate_charge_points_per_use_wrapper:
    SAVEALL

    movss (%rax), %xmm1 // Get max_charge
    xorps %xmm0, %xmm0 // Reimplement max from original code.
    maxss %xmm0, %xmm1
    movaps %xmm7, %xmm0 // Get base_points
    call {calculate_charge_points_per_use_hook}

    RESTOREALL retfpr=1
    ret

/*
 * This is mostly here out of paranoia. Since the compiler could technically
 * optimize out register saving here, we save everything except our return
 * value.
 */
player_avo_get_current_wrapper:
    SAVEALL
    call {player_avo_get_current_hook}
    RESTOREALL retfpr=1
    ret

/*
 * This function allows us to call the OG player_avo_get_current() function by
 * running the overwritten instructions and then jumping to the address after
 * our hook.
 */
player_avo_get_current_original_wrapper_ae:
    mov %rsp, %r11
    push %rbp
    push %rsi
    push %rdi
    push %r14
    push %r15
    sub $0x50, %rsp
    jmp *{player_avo_get_current_return_trampoline}(%rip)

player_avo_get_current_original_wrapper_se:
    push %rbp
    push %rsi
    push %rdi
    push %r14
    push %r15
    sub $0x40, %rsp
    jmp *{player_avo_get_current_return_trampoline}(%rip)

/*
 * Absolute paranoia, as above.
 */
update_skill_list_wrapper:
    SAVEALL
    call {update_skill_list_hook}
    RESTOREALL
    ret

/*
 * Reimplements the prologue of the function which displays skill levels in the
 * UI, which allows us to call the original function despite redirecting at the
 * prologue.
 */
update_skill_list_original_wrapper:
    mov %rsp, %rax
    push %rbp
    push %rbx
    push %rsi
    push %rdi
    push %r12
    push %r13
    push %r14
    jmp *{update_skill_list_return_trampoline}(%rip)

/*
 * Applies additional multipliers to the skill experience gained in the location
 * where the game applies the skill experience base multiplier and offset.
 *
 * Note well that this is an example of a location where the compiler expects
 * every caller saved gpr except eax to not be smashed (in SE, at least).
 */
improve_player_skill_points_wrapper_ae:
    movaps %xmm6, %xmm0 // base exp in AE.
    jmp _improve_player_skill_points_wrapper

improve_player_skill_points_wrapper_se:
    movaps %xmm7, %xmm0 // base exp in SE.

_improve_player_skill_points_wrapper:
    SAVEALL

    mov %esi, %ecx          // Get actor attribute.
    movss 0xf0(%rsp), %xmm1 // Get skill use mult.
    mulss %xmm0, %xmm1      // exp_base = base * mult
    movss 0xf4(%rsp), %xmm2 // Get skill offset
    call {improve_player_skill_points_hook}

    RESTOREALL retfpr=1
    ret

/*
 * Reads in the number of perk points to add to the players pool and then updates
 * the pool based on the sign of that number and our config settings.
 */
modify_perk_pool_wrapper_ae:
    SAVEALL
    mov %dil, %cl // Get modification count.
    call {modify_perk_pool_hook}
    RESTOREALL
    ret

modify_perk_pool_wrapper_se:
    SAVEALL
    mov %bl, %cl // Get modification count (%ebx in SE).
    call {modify_perk_pool_hook}
    RESTOREALL
    ret

/*
 * Passes the EXP gain to our function for further modification.
 * Note that the code here is rather different than the OG implementation
 * because the original improve_level_exp_by_skill_level() function got inlined.
 *
 * Note also that SE expects exp to be in xmm0 while ae expects it in xmm1.
 */
improve_level_exp_by_skill_level_wrapper_ae:
    SAVEALL

    subss %xmm2, %xmm1 // Reimplement replaced code.
    movss %xmm1, %xmm0 // xmm1 contains level exp (without mult)
    mov %esi, %edx     // esi contains skill_id.
    call {improve_level_exp_by_skill_level_hook}
    movdqu %xmm0, 0x30(%rsp) // Replace xmm1 with our result.

    RESTOREALL
    ret

improve_level_exp_by_skill_level_wrapper_se:
    SAVEALL

    subss %xmm1, %xmm0 // Reimplement replaced code.
    mov %esi, %edx     // esi contains skill_id.
    call {improve_level_exp_by_skill_level_hook}

    RESTOREALL retfpr=1
    ret

/*
 * Fixes the arguments before jumping to the hook.
 */
improve_attribute_when_level_up_wrapper:
    SAVEALL
    mov %edx, %ecx
    call {improve_attribute_when_level_up_hook}
    RESTOREALL
    ret

/*
 * Modifies the rest level of legendarying a skill depending on the user
 * settings and the base level in xmm0.
 */
legendary_reset_skill_level_wrapper:
    SAVEALL

    // Get the level we should set the legendary skill to.
    call {legendary_reset_skill_level_hook}

    // Save the new value in what will be arg 3 (xmm2) of the call we return to.
    movdqu %xmm0, 0x40(%rsp)

    // Get the player AVO, we will return back to a call which applies
    // the new skill level value.
    call {get_player_avo}

    mov %rax, 0xa8(%rsp) // Move player avo into arg 1 (saved rcx).
    mov (%rax), %rax     // Load v-table for the call we return to.

    RESTOREALL retgpr=1
    ret

/*
 * Overwrites the call to the original legendary skill condition check
 * with a call to our own condition check.
 */
check_condition_for_legendary_skill_wrapper:
    SAVEALL
    mov %eax, %ecx
    call {check_condition_for_legendary_skill_hook}
    RESTOREALL retfpr=1
    ret

/*
 * Allows the legendary button to be hidden based on an INI setting.
 */
hide_legendary_button_wrapper_ae:
    SAVEALL
    mov %r15d, %ecx
    call {hide_legendary_button_hook}
    RESTOREALL retfpr=1
    ret

hide_legendary_button_wrapper_se:
    SAVEALL
    mov %esi, %ecx
    call {hide_legendary_button_hook}
    RESTOREALL retfpr=1
    ret

/*
 * Forces the game to clear the legendary button in accordance with
 * what our mod expects when scrolling in the skill menu.
 */
clear_legendary_button_wrapper_ae:
    SAVEALL
    mov %r15d, %ecx
    call {clear_legendary_button_hook}
    RESTOREALL retfpr=1
    ret

clear_legendary_button_wrapper_se:
    SAVEALL
    mov %esi, %ecx
    call {clear_legendary_button_hook}
    RESTOREALL retfpr=1
    ret

////////////////////////////////////////////////////////////////////////////////
// Legendary skill patches
////////////////////////////////////////////////////////////////////////////////
// The patches in this section are controlled by the bUseLegendarySettings
// field in the INI file. They allow the player to reconfigure the level
// requirements for legendarying skills.

// Overwrites the game code which checks if a skill is at level 100 by changing
// the value checked to be the one which the user configured in the INI file.
//
// Note that this patch must be generated for multiple different locations,
// since the source code its changing was inlined multiple times. As such, it is
// declared as a macro with the relevant trampolines as its parameters.
.macro LEGENDARY_CHECK_PATCH return_trampoline, parent_return
    // NOW TRUE:
    //   - %xmm0 contains the level of the relevant skill.
    //   - %rax matches the current value of %rip from our jump to this patch.
    //   - All other registers are values we must not alter.
    //   - We have no return address, since we jumped to this location.
    //   - The shadow space is set up for calling a function.
    //   - Our stack is aligned to 16 bytes.
    // MUST BE TRUE:
    //   - The symbol {legendary_skill_level_en_value} points to the f32 value
    //     of the condition check setting.
    comiss {legendary_skill_level_en_value}(%rip), %xmm0

    // NOW TRUE:
    //   - The %eflags register contains the result of our comparison.
    //   - All other values remain the same.
    jb _check_condition_return_jump

    // NOW TRUE:
    //   - The level value contained in %xmm0 is greater than or equal to the 
    //     configured legendary skill enable setting.
    //   - All other values remain the same.
    // MUST BE TRUE:
    //   - The trampoline for this patch returns to the location after the 'jb'
    //     instruction in the games source code.
    jmp *return_trampoline(%rip)

_check_condition_return_jump?:
    // NOW TRUE:
    //   - The level value contained in %xmm0 is less than the configured
    //     legendary skill enable setting.
    //   - All other values remain the same.
    // MUST BE TRUE:
    //   - The symbol below contains the address which starts the return code
    //     for the legendary condition check function.
    jmp *parent_return(%rip)
.endm

check_condition_for_legendary_skill_wrapper:
    LEGENDARY_CHECK_PATCH return_trampoline={check_condition_for_legendary_skill_return_trampoline}, parent_return={check_condition_for_legendary_skill_parent_return}

check_condition_for_legendary_skill_alt_wrapper:
    LEGENDARY_CHECK_PATCH return_trampoline={check_condition_for_legendary_skill_alt_return_trampoline}, parent_return={check_condition_for_legendary_skill_alt_parent_return}

////////////////////////////////////////////////////////////////////////////////
