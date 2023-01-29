/**
 * @file wrapper.cpp
 * @author Andrew Spaulding (Kasplat)
 * @brief C-style wrappers around the C++ skse functions.
 * @bug No known bugs.
 *
 * Note that we must wrap each C++ call in a try-catch block, as exceptions
 * cannot safely unwind into rust code.
 */

#include <cstdint>
#include <cstdlib>

#include <common/IPrefix.h>
#include <Shlobj.h>
#include <skse_stop.h>
#include <skse_version.h>
#include <Utilities.h>
#include <SafeWrite.h>
#include <Relocation.h>
#include <BranchTrampoline.h>

/// @brief The type of trampoline each function should operate on.
enum class Trampoline { Global, Local };

/// @brief Gets a trampoline pointer from its enum.
static BranchTrampoline *
GetTrampoline(
    Trampoline t
) {
    switch (t) {
        case Trampoline::Global:
            return &g_branchTrampoline;
        case Trampoline::Local:
            return &g_localTrampoline;
        default:
            STOP("Invalid trampoline selection");
    }
}

extern "C" {
    void
    SKSE64_Errors__assert_failed__(
        const char *file,
        unsigned long line,
        const char *msg
    ) {
        try {
            _AssertionFailed(file, line, msg);
        } catch(...) {
            // What are we gonna do, panic harder?
            abort();
        }
    }

    void
    SKSE64_DebugLog__open__(
        const char *log
    ) {
        try {
            auto s = std::string("\\My Games\\" SAVE_FOLDER_NAME "\\SKSE\\");
            s += log;
            gLog.OpenRelative(CSIDL_MYDOCUMENTS, s.c_str());
        } catch(...) {
            STOP("Failed to open log file");
        }
    }

    void
    SKSE64_DebugLog__message__(
        const char *msg
    ) {
        try {
            _MESSAGE(msg);
        } catch(...) {
            STOP("Failed to write message to log.");
        }
    }

    void
    SKSE64_DebugLog__error__(
        const char *msg
    ) {
        try {
            _ERROR(msg);
        } catch(...) {
            STOP("Failed to write error to log.");
        }
    }

    const char *
    SKSE64_Utilities__get_runtime_dir__() {
        try {
            return GetRuntimeDirectory().c_str();
        } catch(...) {
            STOP("Failed to get runtime directory.");
        }
    }

    uintptr_t
    SKSE64_Reloc__base__() {
        return RelocationManager::s_baseAddr;
    }

    void
    SKSE64_BranchTrampoline__create__(
        Trampoline t,
        size_t len,
        void *module
    ) {
        try {
            ASSERT(GetTrampoline(t)->Create(len, module));
        } catch(...) {
            STOP("Unable to allocate trampoline buffer");
        }
    }

    void
    SKSE64_BranchTrampoline__destroy__(
        Trampoline t
    ) {
        try {
            GetTrampoline(t)->Destroy();
        } catch(...) {
            STOP("Failed to destroy trampoline");
        }
    }

    void
    SKSE64_BranchTrampoline__write_jump6__(
        Trampoline t,
        uintptr_t src,
        uintptr_t dst
    ) {
        try {
            ASSERT(GetTrampoline(t)->Write6Branch(src, dst));
        } catch(...) {
            STOP("Failed to write Jump-6 to trampoline");
        }
    }

    void
    SKSE64_BranchTrampoline__write_call6__(
        Trampoline t,
        uintptr_t src,
        uintptr_t dst
    ) {
        try {
            ASSERT(GetTrampoline(t)->Write6Call(src, dst));
        } catch(...) {
            STOP("Failed to write Call-6 to trampoline");
        }
    }

    void
    SKSE64_BranchTrampoline__write_jump5__(
        Trampoline t,
        uintptr_t src,
        uintptr_t dst
    ) {
        try {
            ASSERT(GetTrampoline(t)->Write5Branch(src, dst));
        } catch(...) {
            STOP("Failed to write Jump-5 to trampoline");
        }
    }

    void
    SKSE64_BranchTrampoline__write_call5__(
        Trampoline t,
        uintptr_t src,
        uintptr_t dst
    ) {
        try {
            ASSERT(GetTrampoline(t)->Write5Call(src, dst));
        } catch(...) {
            STOP("Failed to write Call-5 to trampoline");
        }
    }

    void
    SKSE64_SafeWrite__virtual_protect__(
        uintptr_t addr,
        size_t size,
        uint32_t new_prot,
        uint32_t *old_prot
    ) {
        try {
            ASSERT(VirtualProtect(
                reinterpret_cast<void*>(addr),
                size,
                new_prot,
                reinterpret_cast<PDWORD>(old_prot)
            ));
        } catch(...) {
            STOP("Failed to protect memory region");
        }
    }

    int
    SKSE64_SafeWrite__safe_write_jump__(
        uintptr_t src,
        uintptr_t dst
    ) {
        try {
            return SafeWriteJump(src, dst) ? 0 : -1;
        } catch(...) {
            STOP("Exception while writing direct jump.");
        }
    }

    int
    SKSE64_SafeWrite__safe_write_call__(
        uintptr_t src,
        uintptr_t dst
    ) {
        try {
            return SafeWriteCall(src, dst) ? 0 : -1;
        } catch(...) {
            STOP("Exception while writing direct call.");
        }
    }
}
