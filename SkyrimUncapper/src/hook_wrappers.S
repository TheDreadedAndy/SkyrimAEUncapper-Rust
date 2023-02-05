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
 * x64 calling convention is weird, and it is very important that
 * it be obeyed (or else unwinding will cause UB). Notably, the stack
 * pointer must stay 16 byte alligned, which means any reg saving needs
 * to allocate stack space as 16 * n + 8. There must also be 32 bytes of
 * stack space for any called functions to use above the return address.
 * Another important point is that once an epilogue starts nothing else can
 * happen except for that epilogue (for unwinding).
 */

.global skill_cap_patch_wrapper
.global max_charge_begin_wrapper
.global max_charge_end_wrapper
.global calculate_charge_points_per_use_wrapper
.global player_avo_get_current_original_wrapper
.global display_true_skill_level_hook
.global display_true_skill_color_hook
.global improve_skill_by_training_hook
.global improve_player_skill_points_original
.global modify_perk_pool_wrapper
.global improve_level_exp_by_skill_level_wrapper
.global improve_attribute_when_level_up_wrapper
.global legendary_reset_skill_level_wrapper
.global check_condition_for_legendary_skill_wrapper
.global check_condition_for_legendary_skill_alt_wrapper
.global hide_legendary_button_wrapper
.global clear_legendary_button_wrapper

// Saves all caller saved registers.
.macro SAVEALL
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
.macro RESTOREALL
    movdqu 0x20(%rsp), %xmm0
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
    pop %rax
.endm

// Saves all caller saved registers, except rax.
.macro SAVEALL_RETGPR
    push %rcx
    push %rdx
    push %r8
    push %r9
    push %r10
    push %r11
    sub $0x88, %rsp // Allocate extra 32 + 8 bytes for calling convention.
    movdqu %xmm0, 0x20(%rsp)
    movdqu %xmm1, 0x30(%rsp)
    movdqu %xmm2, 0x40(%rsp)
    movdqu %xmm3, 0x50(%rsp)
    movdqu %xmm4, 0x60(%rsp)
    movdqu %xmm5, 0x70(%rsp)
.endm

// Restores the caller saved registers, except rax.
.macro RESTOREALL_RETGPR
    movdqu 0x20(%rsp), %xmm0
    movdqu 0x30(%rsp), %xmm1
    movdqu 0x40(%rsp), %xmm2
    movdqu 0x50(%rsp), %xmm3
    movdqu 0x60(%rsp), %xmm4
    movdqu 0x70(%rsp), %xmm5
    add $0x88, %rsp
    pop %r11
    pop %r10
    pop %r9
    pop %r8
    pop %rdx
    pop %rcx
.endm

// Saves all caller saved registers except xmm0.
.macro SAVEALL_RETFPR
    push %rax
    push %rcx
    push %rdx
    push %r8
    push %r9
    push %r10
    push %r11
    sub $0x70, %rsp
    movdqu %xmm1, 0x20(%rsp)
    movdqu %xmm2, 0x30(%rsp)
    movdqu %xmm3, 0x40(%rsp)
    movdqu %xmm4, 0x50(%rsp)
    movdqu %xmm5, 0x60(%rsp)
.endm

// Restores all caller saved registers except xmm0.
.macro RESTOREALL_RETFPR
    movdqu 0x20(%rsp), %xmm1
    movdqu 0x30(%rsp), %xmm2
    movdqu 0x40(%rsp), %xmm3
    movdqu 0x50(%rsp), %xmm4
    movdqu 0x60(%rsp), %xmm5
    add $0x70, %rsp
    pop %r11
    pop %r10
    pop %r9
    pop %r8
    pop %rdx
    pop %rcx
    pop %rax
.endm

/*
 * This function gets injected in the middle of another, so we must use the call
 * injection macros to protect the register state.
 */
skill_cap_patch_wrapper:
    sub $0x28, %rsp

    mov %esi, %ecx // pass SkillID in ecx to hook.
    call player_avo_get_base
    movaps %xmm0, %xmm8 // Reimplement original move.

    mov %esi, %ecx // Move skill ID back into first arg.
    call get_skill_cap_hook
    movaps %xmm8, %xmm0 // Restore xmm0, just in case (this call wasn't here before).
    movss %xmm0, %xmm10 // Replace maximum with fn result.

    add $0x28, %rsp
    ret

/*
 * Begins a max_charge calculation by changing the enchanting formula cap
 * to use the weapon cap if the enchanted item is offensive.
 */
max_charge_begin_wrapper:
    SAVEALL

    // Reimplement replaced instruction.
    movss %xmm0, 0x160(%rsp)

    mov %r14d, %ecx // Enchantment type is in r14d.
    call max_charge_begin_hook

    mov 0xa8(%rsp), %rcx // Get rcx value
    test %rcx, %rcx // Reimplement replaced test + je.
    jne _finish_max_charge_begin_wrapper
    addq $0x12f, 0xb8(%rsp) // Offset return address (this is a crime).

_finish_max_charge_begin_wrapper:
    RESTOREALL
    ret

/*
 * Ends a max_charge calculation by returning the skill formula equation
 * to using the magnitude formula cap, if necessary.
 */
max_charge_end_wrapper:
    // We jump here, but we also need to save register state. We can't do that without
    // violating calling convention, though, so we call another function which saves the
    // state and then we use our trampoline.
    call max_charge_end_invoker
    jmp *max_charge_end_return_trampoline(%rip)

max_charge_end_invoker:
    SAVEALL_RETFPR

    call max_charge_end_hook

    // Reimplement overwritten instructions.
    movss 0x150(%rsp), %xmm0 // Get max_charge
    maxss %xmm9, %xmm0 // MAX(max_charge, 0)

    RESTOREALL_RETFPR
    ret

/*
 * Wraps our calculate_charge_points_per_use() function. We use this wrapper to
 * get access to the maxCharge argument that our caller gets, which is its 5th
 * argument (rsp + a0). We replace the call to the original function, so no
 * reg save. We're just adding an arg.
 */
calculate_charge_points_per_use_wrapper:
    movss 0xa8(%rsp), %xmm1 // Get max_charge.
    movaps %xmm7, %xmm0 // Get base_points
    xorps %xmm2, %xmm2 // Reimplement max from original code.
    maxss %xmm2, %xmm1
    jmp calculate_charge_points_per_use_hook

/*
 * This function allows us to call the OG player_avo_get_current() function by
 * running the overwritten instructions and then jumping to the address after
 * our hook.
 */
player_avo_get_current_original_wrapper:
    mov %rsp, %r11
    push %rbp
    push %rsi
    push %rdi
    push %r14
    push %r15
    sub $0x50, %rsp
    jmp *player_avo_get_current_return_trampoline(%rip)

/*
 * Forces the code which displays skill values in the skills menu to show the
 * true skill level/color instead of the damaged value by calling the OG function.
 *
 * It just so happens that things get thrown in the same regs, so the patch is
 * the same for both.
 */
display_true_skill_level_hook:
display_true_skill_color_hook:
    sub $0x28, %rsp

    // Reimplement replaced setup.
    call get_player_avo
    mov %rax, %rcx
    mov %edi, %edx

    // Call the unmodified function.
    call player_avo_get_current_original_wrapper

    add $0x28, %rsp
    ret

/*
 * Forces the training fn to call the original improve_player_skill_points()
 * function.
 */
improve_skill_by_training_hook:
    // Reimplement the arg setup we overwrote.
    mov %r13b, 0x28(%rsp)
    mov %r15, %rcx
    mov %r13d, 0x20(%rsp)
    call improve_player_skill_points_original
    jmp *improve_skill_by_training_return_trampoline(%rip)

/*
 * This function allows us to call the games original
 * improve_player_skill_points() function by reimplementing the code our hook
 * replaces and then jumping to the original game code that follows our hook.
 */
improve_player_skill_points_original:
    mov %rsp, %rax
    push %rdi
    push %r12
    push %r13
    push %r14
    push %r15
    sub $0x180, %rsp
    jmp *improve_player_skill_points_return_trampoline(%rip)

/*
 * Reads in the number of perk points to add to the players pool and then updates
 * the pool based on the sign of that number and our config settings.
 *
 * We save all registers, since this is an injected call.
 */
modify_perk_pool_wrapper:
    SAVEALL
    mov %dil, %cl // Get modification count.
    call modify_perk_pool_hook
    RESTOREALL
    ret

/*
 * Passes the EXP gain to our function for further modification.
 * Note that the code here is rather different than the OG implementation
 * because the original improve_level_exp_by_skill_level() function got inlined.
 */
improve_level_exp_by_skill_level_wrapper:
    SAVEALL

    movss %xmm1, %xmm0 // xmm1 contains level exp
    mov %rsi, %rdx // rsi contains skill_id.
    call improve_level_exp_by_skill_level_hook

    mov 0xb0(%rsp), %rax // Get the saved value for rax.
    addss (%rax), %xmm0 // This is the code we overwrote, except we
    movss %xmm0, (%rax) // now use xmm0 instead of xmm1.

    // Set up args for the call we return to.
    call get_player
    mov %rax, 0xa8(%rsp) // Move into saved rcx.
    mov %esi, 0xa0(%rsp) // Move into saved rdx.

    RESTOREALL

    ret

/*
 * Fixes the return address before jumping to the hook.
 */
improve_attribute_when_level_up_wrapper:
    mov %edx, %ecx
    jmp improve_attribute_when_level_up_hook

/*
 * Modifies the rest level of legendarying a skill depending on the user
 * settings and the base level in xmm0.
 */
legendary_reset_skill_level_wrapper:
    SAVEALL
    call legendary_reset_skill_level_hook
    RESTOREALL
    ret

/*
 * Overwrites the call to the original legendary skill condition check
 * with a call to our own condition check.
 * Since we are overwriting another fn call, we dont need to reg save.
 *
 * Note that we have to use jumps and not call/ret here, as putting the
 * comparison after the rsp add violates the epilogue calling convention.
 * We can't use leaq to stop the flags from being wiped, either, as that
 * would also violate calling convention.
 */
check_condition_for_legendary_skill_wrapper:
    mov %edx, %ecx
    call check_condition_for_legendary_skill_hook
    cmp $1, %al
    jmp *check_condition_for_legendary_skill_return_trampoline(%rip)

check_condition_for_legendary_skill_alt_wrapper:
    mov %edx, %ecx
    call check_condition_for_legendary_skill_hook
    cmp $1, %al
    jmp *check_condition_for_legendary_skill_alt_return_trampoline(%rip)

/*
 * Allows the legendary button to be hidden based on an INI setting.
 * We are overwriting another function call here, so we dont need to save
 * the register state.
 */
hide_legendary_button_wrapper:
    mov %r15d, %ecx
    call hide_legendary_button_hook
    cmp $1, %al
    jmp *hide_legendary_button_return_trampoline(%rip)

/*
 * Forces the game to clear the legendary button in accordance with
 * what our mod expects when scrolling in the skill menu.
 */
clear_legendary_button_wrapper:
    mov %r15d, %ecx
    jmp clear_legendary_button_hook
