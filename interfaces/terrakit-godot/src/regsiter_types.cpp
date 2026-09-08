#include "register_types.h"

#include "terrakit_bridge.h"

#include <gdextension_interface.h>

#include <godot_cpp/core/class_db.hpp>
#include <godot_cpp/core/defs.hpp>
#include <godot_cpp/godot.hpp>

using namespace godot;


void initialize_terrakit_godot(
    ModuleInitializationLevel p_level
) {
    if (
        p_level
        != MODULE_INITIALIZATION_LEVEL_SCENE
    ) {
        return;
    }

    GDREGISTER_CLASS(TerraKitBridge);
}


void uninitialize_terrakit_godot(
    ModuleInitializationLevel p_level
) {
    if (
        p_level
        != MODULE_INITIALIZATION_LEVEL_SCENE
    ) {
        return;
    }
}


extern "C" {

GDExtensionBool GDE_EXPORT
terrakit_godot_library_init(
    GDExtensionInterfaceGetProcAddress
        p_get_proc_address,
    GDExtensionClassLibraryPtr p_library,
    GDExtensionInitialization *r_initialization
) {
    GDExtensionBinding::InitObject init_obj(
        p_get_proc_address,
        p_library,
        r_initialization
    );

    init_obj.register_initializer(
        initialize_terrakit_godot
    );

    init_obj.register_terminator(
        uninitialize_terrakit_godot
    );

    init_obj.set_minimum_library_initialization_level(
        MODULE_INITIALIZATION_LEVEL_SCENE
    );

    return init_obj.init();
}

}