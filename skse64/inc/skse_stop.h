/**
 * @file skse_stop.h
 * @author Andrew Spaulding (Kasplat)
 * @brief Wrapper around HALT() which ensures termination.
 * @bug No known bugs.
 */

#ifndef STOP

#include <cstdlib>
#include <common/IErrors.h>

#define STOP(s)\
do {\
    try {\
        HALT(s);\
        abort();\
    } catch(...) {\
        abort();\
    }\
} while (0)
#endif
