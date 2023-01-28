# 1 "src/hook_wrappers.S"
# 1 "<built-in>" 1
# 1 "src/hook_wrappers.S" 2
# 16 "src/hook_wrappers.S"
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


.macro SAVEALL
    push %rax
    BEGIN_INJECTED_CALL
.endm


.macro RESTOREALL
    END_INJECTED_CALL
    pop %rax
.endm





.global skill_cap_patch_wrapper; skill_cap_patch_wrapper:
    BEGIN_INJECTED_CALL
    mov %esi, %ecx
    sub $0x20, %rsp
    call get_skill_cap_hook
    add $0x20, %rsp
    movss %xmm0, %xmm10
    END_INJECTED_CALL
    ret







.global calculate_charge_points_per_use_wrapper; calculate_charge_points_per_use_wrapper:
    movss 0xa8(%rsp), %xmm2
    movaps %xmm7, %xmm1
    xorps %xmm3, %xmm3
    maxss %xmm3, %xmm2
    jmp calculate_charge_points_per_use_hook






.global player_avo_get_current_original_wrapper; player_avo_get_current_original_wrapper:
    mov %rsp, %r11
    push %rbp
    push %rsi
    push %rdi
    jmp player_avo_get_current_return_trampoline





.global display_true_skill_level_hook; display_true_skill_level_hook:
    call player_avo_get_current_original_wrapper
    cvttss2si %xmm0, %ecx
    jmp display_true_skill_level_return_trampoline






.global display_true_skill_color_hook; display_true_skill_color_hook:





    push %rax
    sub $0x20, %rsp
    call player_avo_get_current_original_wrapper
    add $0x20, %rsp
    pop %rax
    ret






.global improve_player_skill_points_original; improve_player_skill_points_original:
    mov %rsp, %rax
    push %rdi
    push %r12
    jmp improve_player_skill_points_return_trampoline






.global modify_perk_pool_wrapper; modify_perk_pool_wrapper:
    BEGIN_INJECTED_CALL
    mov %rdi, %rdx
    sub $0x20, %rsp
    call modify_perk_pool_hook
    add $0x20, %rsp
    END_INJECTED_CALL
    mov %al, %cl
    jmp modify_perk_pool_return_trampoline






.global improve_level_exp_by_skill_level_wrapper; improve_level_exp_by_skill_level_wrapper:
    sub $0x10, %rsp
    movdqu %xmm6, (%rsp)
    SAVEALL

    movss %xmm1, %xmm0
    mov %rsi, %rdx
    sub $0x20, %rsp
    call improve_level_exp_by_skill_level_hook
    add $0x20, %rsp
    movss %xmm0, %xmm6

    RESTOREALL

    addss (%rax), %xmm6
    movss %xmm6, (%rax)

    movdqu (%rsp), %xmm6
    add $0x10, %rsp
    ret





.global legendary_reset_skill_level_wrapper; legendary_reset_skill_level_wrapper:
    SAVEALL
    sub $0x20, %rsp
    call legendary_reset_skill_level_hook
    add $0x20, %rsp
    RESTOREALL
    ret






.global check_condition_for_legendary_skill_wrapper; check_condition_for_legendary_skill_wrapper:
    call check_condition_for_legendary_skill_hook
    cmp $1, %al
    jmp check_condition_for_legendary_skill_return_trampoline






.global hide_legendary_button_wrapper; hide_legendary_button_wrapper:
    call hide_legendary_button_hook
    cmp $1, %al
    jmp hide_legendary_button_return_trampoline
