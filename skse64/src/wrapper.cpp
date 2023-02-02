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
#include <skse_version.h>
#include <Utilities.h>
#include <SafeWrite.h>
#include <Relocation.h>
#include <BranchTrampoline.h>

// Stops the plugin.
#define STOP(s) SKSE64_Errors__rust_panic__(\
    reinterpret_cast<const uint8_t *>(__FILE__),\
    sizeof(__FILE__),\
    __LINE__,\
    reinterpret_cast<const uint8_t *>(s),\
    sizeof(s)\
)

/// @brief The type of trampoline each function should operate on.
enum class Trampoline { Global, Local };

// Defined in assembly.
extern "C" __declspec(noreturn) void SKSE64_Errors__stop_plugin__();

/*
 * These functions were originally defined in
 * skse64_src/common/common/IErrors.cpp, however the implementation
 * contained U.B. that clang refuses to compile as the authors intended.
 *
 * As such, they are reimplemented here to instead stop in a well defined way.
 */
__declspec(noreturn) void
_AssertionFailed(
    const char *file,
    unsigned long line,
    const char *desc
) {
    _FATALERROR("%s:%d: `%s'", file, line, desc);
    SKSE64_Errors__stop_plugin__();
}

__declspec(noreturn) void
_AssertionFailed_ErrCode(
    const char *file,
    unsigned long line,
    const char *desc,
    unsigned long long code
) {
    _FATALERROR("%s:%d: `%s' (code = %zx)", file, line, desc, code);
    SKSE64_Errors__stop_plugin__();
}

__declspec(noreturn) void
_AssertionFailed_ErrCode(
    const char *file,
    unsigned long line,
    const char *desc,
    const char *code
) {
    _FATALERROR("%s:%d: `%s' (code = %s)", file, line, desc, code);
    SKSE64_Errors__stop_plugin__();
}

/*
 * Bindings for rust code to call.
 */

extern "C" {
    __declspec(noreturn) void
    SKSE64_Errors__rust_panic__(
        const uint8_t *file,
        size_t file_len,
        size_t line,
        const uint8_t *msg,
        size_t msg_len
    ) {
        try {
            _FATALERROR("%.*s:%zu: `%.*s'", file_len, file, line, msg_len, msg);
            SKSE64_Errors__stop_plugin__();
        } catch(...) {
            SKSE64_Errors__stop_plugin__();
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
        const uint8_t *msg,
        size_t len
    ) {
        try {
            _MESSAGE("%.*s", len, msg);
        } catch(...) {
            STOP("Failed to write message to log.");
        }
    }

    void
    SKSE64_DebugLog__error__(
        const uint8_t *msg,
        size_t len
    ) {
        try {
            _ERROR("%.*s", len, msg);
        } catch(...) {
            STOP("Failed to write error to log.");
        }
    }
}

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
