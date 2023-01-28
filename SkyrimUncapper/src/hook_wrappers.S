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
 */

#define PUBLIC_FN(ident) .global ident; ident:

// Saves all caller saved registers, except rax.
.macro BEGIN_INJECTED_CALL
    push %rcx
    push %rdx
    push %r8
    push %r9
    push %r10
    push %r11
    sub $0x60, %rsp
    movdqu %xmm0, (%rsp)
    movdqu %xmm1, 0x10(%rsp)
    movdqu %xmm2, 0x20(%rsp)
    movdqu %xmm3, 0x30(%rsp)
    movdqu %xmm4, 0x40(%rsp)
    movdqu %xmm5, 0x50(%rsp)
.endm

// Restores the caller saved registers, except rax.
.macro END_INJECTED_CALL
    movdqu (%rsp), %xmm0
    movdqu 0x10(%rsp), %xmm1
    movdqu 0x20(%rsp), %xmm2
    movdqu 0x30(%rsp), %xmm3
    movdqu 0x40(%rsp), %xmm4
    movdqu 0x50(%rsp), %xmm5
    add $0x60, %rsp
    pop %r11
    pop %r10
    pop %r9
    pop %r8
    pop %rdx
    pop %rcx
.endm

// Saves all caller saved registers.
.macro SAVEALL
    push %rax
    BEGIN_INJECTED_CALL
.endm

// Restores all caller saved registers.
.macro RESTOREALL
    END_INJECTED_CALL
    pop %rax
.endm

/*
 * This function gets injected in the middle of another, so we must use the call
 * injection macros to protect the register state.
 */
PUBLIC_FN(skill_cap_patch_wrapper)
    BEGIN_INJECTED_CALL
    mov %esi, %ecx // pass SkillID in ecx to hook.
    sub $0x20, %rsp // Required by calling convention to alloc this.
    call get_skill_cap_hook
    add $0x20, %rsp
    movss %xmm0, %xmm10 // Replace maximum with fn result.
    END_INJECTED_CALL
    ret

/*
 * Wraps our calculate_charge_points_per_use() function. We use this wrapper to
 * get access to the maxCharge argument that our caller gets, which is its 5th
 * argument (rsp + a0). We replace the call to the original function, so no
 * reg save. We're just adding an arg.
 */
PUBLIC_FN(calculate_charge_points_per_use_wrapper)
    movss 0xa8(%rsp), %xmm2 // Get max_charge.
    movaps %xmm7, %xmm1 // Get base_points
    xorps %xmm3, %xmm3 // Reimplement max from original code.
    maxss %xmm3, %xmm2
    jmp calculate_charge_points_per_use_hook

/*
 * This function allows us to call the OG player_avo_get_current() function by
 * running the overwritten instructions and then jumping to the address after
 * our hook.
 */
PUBLIC_FN(player_avo_get_current_original_wrapper)
    mov %rsp, %r11
    push %rbp
    push %rsi
    push %rdi
    jmp player_avo_get_current_return_trampoline

/*
 * Forces the code which displays skill values in the skills menu to show the
 * true skill level instead of the damaged value by calling the OG function.
 */
PUBLIC_FN(display_true_skill_level_hook)
    sub $0x20, %rsp
    call player_avo_get_current_original_wrapper // Replacing a call, no need to save.
    add $0x20, %rsp
    cvttss2si %xmm0, %ecx
    ret

/*
 * Forces the code which displays skill color in the skills menu to show the
 * true skill color instead of the damaged color by calling the OG
 * player_avo_get_current() function.
 */
PUBLIC_FN(display_true_skill_color_hook)
    /*
     * We need this later, as we overwrote an instruction which would
     * move the player AVO vtable into rax, and that offset is version
     * dependent.
     */
    push %rax
    sub $0x20, %rsp
    call player_avo_get_current_original_wrapper
    add $0x20, %rsp
    pop %rax
    ret

/*
 * This function allows us to call the games original
 * improve_player_skill_points() function by reimplementing the code our hook
 * replaces and then jumping to the original game code that follows our hook.
 */
PUBLIC_FN(improve_player_skill_points_original)
    mov %rsp, %rax
    push %rdi
    push %r12
    jmp improve_player_skill_points_return_trampoline

/*
 * Injected after the current number of perk points is read. Returns to the
 * instruction where the new perk count is written back. We do this to avoid
 * non-portable (across skyrim versions) accesses to the player class.
 */
PUBLIC_FN(modify_perk_pool_wrapper)
    BEGIN_INJECTED_CALL
    mov %rdi, %rdx // Get modification count.
    sub $0x20, %rsp
    call modify_perk_pool_hook
    add $0x20, %rsp
    END_INJECTED_CALL
    mov %al, %cl // We'll return to an instruction that'll store this in the player
    jmp modify_perk_pool_return_trampoline

/*
 * Passes the EXP gain to our function for further modification.
 * Note that the code here is rather different than the OG implementation
 * because the original improve_level_exp_by_skill_level() function got inlined.
 */
PUBLIC_FN(improve_level_exp_by_skill_level_wrapper)
    sub $0x10, %rsp
    movdqu %xmm6, (%rsp) // Save xmm6, we will wipe it later.
    SAVEALL

    movss %xmm1, %xmm0 // xmm1 contains level exp
    mov %rsi, %rdx // rsi contains skill_id.
    sub $0x20, %rsp
    call improve_level_exp_by_skill_level_hook
    add $0x20, %rsp
    movss %xmm0, %xmm6 // Save result in xmm6.

    RESTOREALL

    addss (%rax), %xmm6 // This is the code we overwrote, except we
    movss %xmm6, (%rax) // now use xmm6 instead of xmm1.

    movdqu (%rsp), %xmm6 // Restore xmm6.
    add $0x10, %rsp
    ret

/*
 * Modifies the rest level of legendarying a skill depending on the user
 * settings and the base level in xmm0.
 */
PUBLIC_FN(legendary_reset_skill_level_wrapper)
    SAVEALL
    sub $0x20, %rsp
    call legendary_reset_skill_level_hook
    add $0x20, %rsp
    RESTOREALL
    ret

/*
 * Overwrites the call to the original legendary skill condition check
 * with a call to our own condition check.
 * Since we are overwriting another fn call, we dont need to reg save.
 */
PUBLIC_FN(check_condition_for_legendary_skill_wrapper)
    sub $0x20, %rsp
    call check_condition_for_legendary_skill_hook
    add $0x20, %rsp
    cmp $1, %al
    ret

/*
 * Allows the legendary button to be hidden based on an INI setting.
 * We are overwriting another function call here, so we dont need to save
 * the register state.
 */
PUBLIC_FN(hide_legendary_button_wrapper)
    sub $0x20, %rsp
    call hide_legendary_button_hook
    add $0x20, %rsp
    cmp $1, %al
    ret
