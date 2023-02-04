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
