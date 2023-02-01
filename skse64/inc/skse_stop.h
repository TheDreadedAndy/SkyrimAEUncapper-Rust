/**
 * @file skse_stop.h
 * @author Andrew Spaulding (Kasplat)
 * @brief Wrapper around HALT() which ensures termination.
 * @bug No known bugs.
 */

#ifndef __SKYRIM_UNCAPPER_SKSE_STOP_H__
#define __SKYRIM_UNCAPPER_SKSE_STOP_H__

#include <cstdlib>
#include <common/IErrors.h>

extern "C" __declspec(noreturn) void SKSE64_Errors__stop_plugin__();

#define STOP(s)\
do {\
    try {\
        HALT(s);\
        SKSE64_Errors__stop_plugin__();\
    } catch(...) {\
        SKSE64_Errors__stop_plugin__();\
    }\
} while (0)

#endif /* __SKYRIM_UNCAPPER_SKSE_STOP_H__ */
