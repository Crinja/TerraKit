#include "terrakit_bridge.h"

#include <godot_cpp/core/class_db.hpp>

#include <terrakit.h>

namespace godot {

void TerraKitBridge::_bind_methods() {
    ClassDB::bind_method(
        D_METHOD("get_abi_version"),
        &TerraKitBridge::get_abi_version
    );
}

String TerraKitBridge::get_abi_version() const {
    tk_version_t version{};

    const tk_status_t status =
        tk_get_abi_version(&version);

    if (status != TK_STATUS_OK) {
        return String("error:")
            + String::num_int64(status);
    }

    return String::num_int64(version.major)
        + "."
        + String::num_int64(version.minor)
        + "."
        + String::num_int64(version.patch);
}

}