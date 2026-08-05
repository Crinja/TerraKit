#include <inttypes.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "terrakit.h"

#define RESOURCE_BASE_HEIGHT UINT64_C(100)
#define RESOURCE_NOISY_HEIGHT UINT64_C(101)
#define RESOURCE_MESH UINT64_C(102)

#define TK_SV_LITERAL(value) \
    (tk_string_view_t) { (value), sizeof(value) - 1 }

static void fail(tk_status_t status, const char *call) {
    size_t required = 0;
    char *message = NULL;

    (void)tk_last_error_message_copy(NULL, 0, &required);
    message = (char *)malloc(required == 0 ? 1 : required);
    if (message != NULL) {
        (void)tk_last_error_message_copy(message, required, &required);
    }

    fprintf(stderr, "%s failed with status %d: %s\n", call, status, message != NULL ? message : "");
    free(message);
    exit(1);
}

#define TK_CALL(call)                 \
    do {                              \
        tk_status_t status = (call);  \
        if (status != TK_STATUS_OK) { \
            fail(status, #call);      \
        }                             \
    } while (0)

static uint32_t schema_version(tk_stage_registry_t *registry, const char *stage_type) {
    size_t index = 0;
    tk_stage_schema_info_t schema;
    tk_string_view_t id = { stage_type, strlen(stage_type) };

    TK_CALL(tk_stage_registry_find_schema(registry, id, &index));
    memset(&schema, 0, sizeof(schema));
    TK_CALL(tk_stage_registry_get_schema(registry, index, &schema));
    return schema.schema_version;
}

static void set_f32(tk_stage_construction_t *stage, const char *id, float value) {
    tk_parameter_value_t parameter;
    memset(&parameter, 0, sizeof(parameter));
    parameter.kind = TK_PARAMETER_KIND_F32;
    parameter.f32_value = value;
    TK_CALL(tk_stage_construction_set_parameter(stage, (tk_string_view_t){ id, strlen(id) }, &parameter));
}

static void set_f64(tk_stage_construction_t *stage, const char *id, double value) {
    tk_parameter_value_t parameter;
    memset(&parameter, 0, sizeof(parameter));
    parameter.kind = TK_PARAMETER_KIND_F64;
    parameter.f64_value = value;
    TK_CALL(tk_stage_construction_set_parameter(stage, (tk_string_view_t){ id, strlen(id) }, &parameter));
}

static void set_u64(tk_stage_construction_t *stage, const char *id, uint64_t value) {
    tk_parameter_value_t parameter;
    memset(&parameter, 0, sizeof(parameter));
    parameter.kind = TK_PARAMETER_KIND_U64;
    parameter.u64_value = value;
    TK_CALL(tk_stage_construction_set_parameter(stage, (tk_string_view_t){ id, strlen(id) }, &parameter));
}

int main(void) {
    tk_version_t version;
    tk_stage_registry_t *registry = NULL;
    tk_stage_construction_t *flat = NULL;
    tk_stage_construction_t *noise = NULL;
    tk_stage_construction_t *mesh_stage = NULL;
    tk_pipeline_assembler_t *assembler = NULL;
    tk_pipeline_t *pipeline = NULL;
    tk_runtime_t *runtime = NULL;
    tk_generation_result_t *result = NULL;
    tk_height_field_view_t height;
    tk_terrain_mesh_view_t mesh;
    tk_region_layout_2d_t layout;
    tk_generation_request_2d_t request;
    uint32_t flat_version;
    uint32_t noise_version;
    uint32_t mesh_version;

    memset(&version, 0, sizeof(version));
    TK_CALL(tk_get_abi_version(&version));
    printf("TerraKit ABI %" PRIu32 ".%" PRIu32 ".%" PRIu32 "\n", version.major, version.minor, version.patch);

    TK_CALL(tk_stage_registry_create_builtin(&registry));
    flat_version = schema_version(registry, "terrakit.height.flat");
    noise_version = schema_version(registry, "terrakit.height.noise");
    mesh_version = schema_version(registry, "terrakit.mesh.height_field");

    TK_CALL(tk_stage_construction_create(1, TK_SV_LITERAL("terrakit.height.flat"), flat_version, &flat));
    TK_CALL(tk_stage_construction_bind_output(flat, TK_SV_LITERAL("height"), RESOURCE_BASE_HEIGHT));

    TK_CALL(tk_stage_construction_create(2, TK_SV_LITERAL("terrakit.height.noise"), noise_version, &noise));
    TK_CALL(tk_stage_construction_bind_input(noise, TK_SV_LITERAL("source"), RESOURCE_BASE_HEIGHT));
    TK_CALL(tk_stage_construction_bind_output(noise, TK_SV_LITERAL("height"), RESOURCE_NOISY_HEIGHT));
    set_u64(noise, "octaves", 3);
    set_f64(noise, "frequency", 0.09);
    set_f32(noise, "amplitude", 0.25f);

    TK_CALL(tk_stage_construction_create(3, TK_SV_LITERAL("terrakit.mesh.height_field"), mesh_version, &mesh_stage));
    TK_CALL(tk_stage_construction_bind_input(mesh_stage, TK_SV_LITERAL("height"), RESOURCE_NOISY_HEIGHT));
    TK_CALL(tk_stage_construction_bind_output(mesh_stage, TK_SV_LITERAL("mesh"), RESOURCE_MESH));

    TK_CALL(tk_pipeline_assembler_create(registry, &assembler));
    TK_CALL(tk_pipeline_assembler_add_stage(assembler, &flat));
    TK_CALL(tk_pipeline_assembler_add_stage(assembler, &noise));
    TK_CALL(tk_pipeline_assembler_add_stage(assembler, &mesh_stage));
    TK_CALL(tk_pipeline_assembler_finish(&assembler, &pipeline));

    memset(&layout, 0, sizeof(layout));
    layout.cell_width = 16;
    layout.cell_height = 16;
    layout.base_spacing.x = 1.0;
    layout.base_spacing.y = 1.0;
    TK_CALL(tk_runtime_create_2d(&pipeline, &layout, &runtime));

    memset(&request, 0, sizeof(request));
    request.seed = 1234;
    request.region_x = 0;
    request.region_y = 0;
    request.lod_level = 0;
    TK_CALL(tk_runtime_generate_2d(runtime, &request, &result));

    memset(&height, 0, sizeof(height));
    memset(&mesh, 0, sizeof(mesh));
    TK_CALL(tk_generation_result_get_height_field(result, RESOURCE_NOISY_HEIGHT, &height));
    TK_CALL(tk_generation_result_get_mesh(result, RESOURCE_MESH, &mesh));

    printf("Height samples: %" PRIu32 " x %" PRIu32 "\n", height.width, height.height);
    printf("Mesh vertices: %zu\n", mesh.position_count);
    printf("Mesh triangles: %zu\n", mesh.index_count / 3);
    printf("Mesh origin: %.3f %.3f %.3f\n", mesh.origin.x, mesh.origin.y, mesh.origin.z);

    TK_CALL(tk_runtime_destroy(&runtime));

    memset(&mesh, 0, sizeof(mesh));
    TK_CALL(tk_generation_result_get_mesh(result, RESOURCE_MESH, &mesh));
    printf("Mesh still readable after runtime destroy: %zu vertices\n", mesh.position_count);

    TK_CALL(tk_generation_result_destroy(&result));
    TK_CALL(tk_stage_registry_destroy(&registry));
    return 0;
}
