#pragma once

#include <godot_cpp/core/class_db.hpp>

void initialize_terrakit_godot(
    godot::ModuleInitializationLevel p_level
);

void uninitialize_terrakit_godot(
    godot::ModuleInitializationLevel p_level
);