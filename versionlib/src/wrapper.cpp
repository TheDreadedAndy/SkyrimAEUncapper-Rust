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
    VersionLibDb__init__() {
        try {
            return new VersionDb();
        } catch(...) {
            HALT("Failed to construct version database");
        }
    }

    void
    VersionLibDb__destroy__(
        VersionDb *db
    ) {
        try {
            ASSERT(db);
            delete db;
        } catch(...) {
            HALT("Failed to destroy version database");
        }
    }

    void
    VersionLibDb__load_current__(
        VersionDb *db
    ) {
        try {
            ASSERT(db);
            ASSERT(db->Load());
        } catch(...) {
            HALT("Failed to load database into version db");
        }
    }

    void
    VersionLibDb__load_release__(
        VersionDb *db,
        int major,
        int minor,
        int build,
        int sub
    ) {
        try {
            ASSERT(db);
            ASSERT(db->Load(major, minor, build, sub));
        } catch(...) {
            HALT("Failed to load specific release into db");
        }
    }

    int
    VersionLibDb__find_offset_by_id__(
        VersionDb *db,
        unsigned long long id,
        unsigned long long *result
    ) {
        try {
            ASSERT(db);
            return db->FindOffsetById(id, *result) ? 0 : -1;
        } catch(...) {
            HALT("Failed to find offset by id in version db");
        }
    }

    int
    VersionLibDb__find_id_by_offset__(
        VersionDb *db,
        unsigned long long offset,
        unsigned long long *result
    ) {
        try {
            ASSERT(db);
            return db->FindIdByOffset(offset, *result) ? 0 : -1;
        } catch(...) {
            HALT("Failed to find id by offset in version db");
        }
    }
}
