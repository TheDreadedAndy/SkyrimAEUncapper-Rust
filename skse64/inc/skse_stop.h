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

__declspec(noreturn) void StopPlugin();

#define STOP(s)\
do {\
    try {\
        HALT(s);\
        StopPlugin();\
    } catch(...) {\
        StopPlugin();\
    }\
} while (0)

#endif /* __SKYRIM_UNCAPPER_SKSE_STOP_H__ */
