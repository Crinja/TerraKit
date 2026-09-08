#pragma once

#include <godot_cpp/classes/ref_counted.hpp>
#include <godot_cpp/variant/string.hpp>

namespace godot {

class TerraKitBridge : public RefCounted {
    GDCLASS(TerraKitBridge, RefCounted);

protected:
    static void _bind_methods();

public:
    String get_abi_version() const;
};

}