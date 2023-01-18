/**
 * @file wrapper.c
 * @author Andrew Spaulding (aspauldi)
 * @brief Wraps the VersionLibDB struct in a C interface.
 * @bug No known bugs.
 *
 * We wrap it in a C interface, since the versionlibdb.h makes bindgen shit
 * itself. In order to keep the interface as C-friendly as possible, we just
 * give abstract function definitions with no types.
 *
 * Additionally, we need to wrap our calls in a try-catch that does
 * "something reasonable" on failure.
 */

#include <common/IErrors.h>

#include <versionlibdb.h>

extern "C" {
    void *
    VersionLibDb__create__() {
        try {
            VersionDb *ret = new VersionDb();
            ret->Load();
            return ret;
        } catch(...) {
            return nullptr;
        }
    }

    void
    VersionLibDb__destroy__(
        void *database
    ) {
        VersionDb *db = static_cast<VersionDb*>(database);
        try {
            delete db;
        } catch(...) {
            HALT("Failed to destroy version database");
        }
    }

    int
    VersionLibDb__find_offset_by_id__(
        void *database,
        unsigned long long id,
        unsigned long long *result
    ) {
        VersionDb *db = static_cast<VersionDb*>(database);
        try {
            return db->FindOffsetById(id, *result) ? 0 : -1;
        } catch(...) {
            return -1;
        }
    }

    int
    VersionLibDb__find_id_by_offset__(
        void *database,
        unsigned long long offset,
        unsigned long long *result
    ) {
        VersionDb *db = static_cast<VersionDb*>(database);
        try {
            return db->FindOffsetById(offset, *result) ? 0 : -1;
        } catch(...) {
            return -1;
        }
    }
}
