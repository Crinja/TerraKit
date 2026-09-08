@tool
extends EditorPlugin


func _enter_tree() -> void:
    var terrakit := TerraKitBridge.new()

    print(
        "TerraKit ABI: ",
        terrakit.get_abi_version()
    )


func _exit_tree() -> void:
    pass