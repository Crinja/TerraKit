using System.Collections.Generic;
using System.Linq;

namespace TerraKit
{
    /// <summary>
    /// Central registry for built-in TerraKit node definitions.
    /// Add custom nodes by extending this registry or replacing it with a reflection/package registry later.
    /// </summary>
    public static class TerraKitNodeRegistry
    {
        private static readonly HashSet<string> BackendExecutableTypeIds = new HashSet<string>
        {
            "terrakit.height.flat",
            "terrakit.height.noise",
            "terrakit.mesh.height_field"
        };

        private static readonly List<ITerraKitNodeDefinition> Definitions = new List<ITerraKitNodeDefinition>
        {
            new TerraKitNodeDefinition(
                "terra.input.seed",
                "World Seed",
                "Input",
                new TerraKitPortDefinition[0],
                new []
                {
                    Out("seed", "Seed", TerraKitPortType.Seed)
                },
                new []
                {
                    Param(
                        "seed", "Seed", TerraKitParameterType.Integer, "12345",
                        "Controls repeatable random generation. The same seed produces the same result.")
                },
                "Repeatable random seed."),

            new TerraKitNodeDefinition(
                "terra.input.chunk_coordinate",
                "Chunk Coordinate",
                "Input",
                new TerraKitPortDefinition[0],
                new []
                {
                    Out("chunk", "Chunk", TerraKitPortType.ChunkCoordinate)
                },
                new []
                {
                    Param(
                        "x", "X", TerraKitParameterType.Integer, "0",
                        "Selects the chunk coordinate on the horizontal X axis."),
                    Param(
                        "z", "Z", TerraKitParameterType.Integer, "0",
                        "Selects the chunk coordinate on the horizontal Z axis.")
                },
                "Terrain chunk coordinates."),

            new TerraKitNodeDefinition(
                "terra.terrain.noise",
                "Noise Terrain",
                "Terrain",
                new []
                {
                    In("seed", "Seed", TerraKitPortType.Seed, true),
                    In("chunk", "Chunk", TerraKitPortType.ChunkCoordinate, false)
                },
                new []
                {
                    Out("heightmap", "Height Map", TerraKitPortType.HeightMap)
                },
                new []
                {
                    Param(
                        "scale", "Scale", TerraKitParameterType.Float, "100",
                        "Controls the horizontal size of terrain features. Larger values create broader shapes.",
                        0, false),
                    Param(
                        "amplitude", "Amplitude", TerraKitParameterType.Float, "50",
                        "Controls the maximum vertical height contributed by the noise.",
                        0, true),
                    Param(
                        "octaves", "Octaves", TerraKitParameterType.Integer, "4",
                        "Controls the number of noise-detail layers. More octaves add finer detail.",
                        1, true)
                },
                "Procedural terrain height map."),

            new TerraKitNodeDefinition(
                "terrakit.height.flat",
                "Flat Height (Backend)",
                "Terrain",
                new TerraKitPortDefinition[0],
                new []
                {
                    Out("height", "Height", TerraKitPortType.HeightMap)
                },
                new []
                {
                    Param(
                        "elevation", "Elevation", TerraKitParameterType.Float, "0",
                        "Constant elevation assigned to every generated height sample."),
                    Param(
                        "height_axis", "Height Axis", TerraKitParameterType.String, "0,1,0",
                        "World-space height axis as comma-separated X,Y,Z values.")
                },
                "Backend built-in stage that creates a constant height field."),

            new TerraKitNodeDefinition(
                "terrakit.height.noise",
                "Noise Height (Backend)",
                "Terrain",
                new []
                {
                    In("source", "Source", TerraKitPortType.HeightMap, true)
                },
                new []
                {
                    Out("height", "Height", TerraKitPortType.HeightMap)
                },
                new []
                {
                    Param(
                        "algorithm", "Algorithm", TerraKitParameterType.String, "value",
                        "Backend noise algorithm identifier. Version 0.0.1 supports value."),
                    Param(
                        "octaves", "Octaves", TerraKitParameterType.Integer, "4",
                        "Number of fractal noise octaves. The backend accepts 1 through 32.",
                        1, true, 32, true),
                    Param(
                        "frequency", "Frequency", TerraKitParameterType.Float, "0.01",
                        "Initial coordinate multiplier used by the backend noise generator."),
                    Param(
                        "lacunarity", "Lacunarity", TerraKitParameterType.Float, "2",
                        "Per-octave frequency multiplier."),
                    Param(
                        "persistence", "Persistence", TerraKitParameterType.Float, "0.5",
                        "Per-octave amplitude multiplier.",
                        0, true),
                    Param(
                        "amplitude", "Amplitude", TerraKitParameterType.Float, "1",
                        "Initial noise amplitude. Backend 0.0.1 also accepts negative values."),
                    Param(
                        "normalize", "Normalize", TerraKitParameterType.Boolean, "true",
                        "Normalize by accumulated absolute octave weight."),
                    Param(
                        "seed_domain", "Seed Domain", TerraKitParameterType.Integer, "0",
                        "Unsigned domain value used to derive this stage's deterministic seed.",
                        0, true),
                    Param(
                        "mode", "Mode", TerraKitParameterType.String, "add",
                        "Combination mode: add, replace, or multiply.")
                },
                "Backend built-in stage that applies deterministic noise to a source height field."),

            new TerraKitNodeDefinition(
                "terrakit.mesh.height_field",
                "Heightfield Mesh (Backend)",
                "Output",
                new []
                {
                    In("height", "Height", TerraKitPortType.HeightMap, true)
                },
                new []
                {
                    Out("mesh", "Mesh", TerraKitPortType.Mesh)
                },
                new []
                {
                    Param(
                        "generate_normals", "Generate Normals", TerraKitParameterType.Boolean, "true",
                        "Generate smooth vertex normals in the backend mesh."),
                    Param(
                        "generate_texcoords", "Generate Texcoords", TerraKitParameterType.Boolean, "true",
                        "Generate regular-grid texture coordinates in the backend mesh.")
                },
                "Backend built-in stage that converts a height field into an indexed mesh."),

            new TerraKitNodeDefinition(
                "terra.terrain.erosion",
                "Erosion Pass",
                "Terrain",
                new []
                {
                    In("heightmap", "Height Map", TerraKitPortType.HeightMap, true)
                },
                new []
                {
                    Out("heightmap", "Height Map", TerraKitPortType.HeightMap)
                },
                new []
                {
                    Param(
                        "iterations", "Iterations", TerraKitParameterType.Integer, "32",
                        "Controls how many erosion simulation steps are applied."),
                    Param(
                        "strength", "Strength", TerraKitParameterType.Float, "0.35",
                        "Controls how strongly the erosion pass changes the terrain.")
                },
                "Terrain erosion simulation."),

            new TerraKitNodeDefinition(
                "terra.terrain.height_blend",
                "Height Blend",
                "Terrain",
                new []
                {
                    In("a", "Height A", TerraKitPortType.HeightMap, true),
                    In("b", "Height B", TerraKitPortType.HeightMap, true),
                    In("mask", "Mask", TerraKitPortType.Mask, false)
                },
                new []
                {
                    Out("heightmap", "Height Map", TerraKitPortType.HeightMap)
                },
                new []
                {
                    Param(
                        "blend", "Blend", TerraKitParameterType.Float, "0.5",
                        "Controls the relative mix between Height A and Height B.")
                },
                "Blend two terrain height maps."),

            new TerraKitNodeDefinition(
                "terra.mask.height",
                "Mask From Height",
                "Mask",
                new []
                {
                    In("heightmap", "Height Map", TerraKitPortType.HeightMap, true)
                },
                new []
                {
                    Out("mask", "Mask", TerraKitPortType.Mask)
                },
                new []
                {
                    Param(
                        "minHeight", "Min Height", TerraKitParameterType.Float, "10",
                        "Sets the lower height used to build the mask."),
                    Param(
                        "maxHeight", "Max Height", TerraKitParameterType.Float, "80",
                        "Sets the upper height used to build the mask.")
                },
                "Mask from a height range."),

            new TerraKitNodeDefinition(
                "terra.biome.classifier",
                "Biome Classifier",
                "Biome",
                new []
                {
                    In("heightmap", "Height Map", TerraKitPortType.HeightMap, true),
                    In("mask", "Mask", TerraKitPortType.Mask, false)
                },
                new []
                {
                    Out("biomes", "Biome Map", TerraKitPortType.BiomeMap)
                },
                new []
                {
                    Param(
                        "temperatureBias", "Temperature Bias", TerraKitParameterType.Float, "0",
                        "Shifts biome classification toward warmer or colder conditions."),
                    Param(
                        "moistureBias", "Moisture Bias", TerraKitParameterType.Float, "0",
                        "Shifts biome classification toward wetter or drier conditions.")
                },
                "Terrain biome classification."),

            new TerraKitNodeDefinition(
                "terra.placement.scatter",
                "Object Scatter",
                "Placement",
                new []
                {
                    In("heightmap", "Height Map", TerraKitPortType.HeightMap, true),
                    In("biomes", "Biome Map", TerraKitPortType.BiomeMap, false),
                    In("mask", "Mask", TerraKitPortType.Mask, false)
                },
                new []
                {
                    Out("objects", "Objects", TerraKitPortType.ObjectSet)
                },
                new []
                {
                    Param(
                        "density", "Density", TerraKitParameterType.Float, "0.15",
                        "Controls the target amount of objects placed across the terrain."),
                    Param(
                        "prefabTag", "Prefab Tag", TerraKitParameterType.String, "Tree",
                        "Selects which tagged prefab group should be scattered.")
                },
                "Place tagged terrain objects."),

            new TerraKitNodeDefinition(
                "terra.output.preview_mesh",
                "Preview Mesh",
                "Output",
                new []
                {
                    In("heightmap", "Height Map", TerraKitPortType.HeightMap, true),
                    In("objects", "Objects", TerraKitPortType.ObjectSet, false)
                },
                new []
                {
                    Out("mesh", "Mesh", TerraKitPortType.Mesh)
                },
                new []
                {
                    Param(
                        "resolution", "Resolution", TerraKitParameterType.Integer, "128",
                        "Controls the number of samples used by the preview mesh. Higher values add detail.")
                },
                "Previewable terrain mesh."),

            new TerraKitNodeDefinition(
                "terra.output.export_chunk",
                "Export Chunk",
                "Output",
                new []
                {
                    In("heightmap", "Height Map", TerraKitPortType.HeightMap, true),
                    In("biomes", "Biome Map", TerraKitPortType.BiomeMap, false),
                    In("objects", "Objects", TerraKitPortType.ObjectSet, false)
                },
                new TerraKitPortDefinition[0],
                new []
                {
                    Param(
                        "path", "Output Path", TerraKitParameterType.String, "Assets/TerraKitOutput",
                        "Sets the Unity project folder used for exported chunk assets.")
                },
                "Export generated chunk assets.")
        };

        public static IReadOnlyList<ITerraKitNodeDefinition> All
        {
            get { return Definitions; }
        }

        public static bool TryGet(string typeId, out ITerraKitNodeDefinition definition)
        {
            definition = Definitions.FirstOrDefault(item => item.TypeId == typeId);
            return definition != null;
        }

        /// <summary>
        /// Returns whether the installed MVP backend can execute this node as a native stage.
        /// Frontend prototype definitions remain registered so existing graph assets can still load.
        /// </summary>
        public static bool IsBackendExecutable(string typeId)
        {
            return !string.IsNullOrEmpty(typeId) && BackendExecutableTypeIds.Contains(typeId);
        }

        public static bool ArePortTypesCompatible(TerraKitPortType outputType, TerraKitPortType inputType)
        {
            return outputType == TerraKitPortType.Any ||
                   inputType == TerraKitPortType.Any ||
                   outputType == inputType;
        }

        private static TerraKitPortDefinition In(string id, string name, TerraKitPortType type, bool required)
        {
            return new TerraKitPortDefinition(id, name, type, required, false);
        }

        private static TerraKitPortDefinition Out(string id, string name, TerraKitPortType type)
        {
            return new TerraKitPortDefinition(id, name, type, false, true);
        }

        private static TerraKitParameterDefinition Param(
            string key,
            string name,
            TerraKitParameterType type,
            string defaultValue,
            string description,
            double? minimumValue = null,
            bool minimumInclusive = true,
            double? maximumValue = null,
            bool maximumInclusive = true)
        {
            return new TerraKitParameterDefinition(
                key,
                name,
                type,
                defaultValue,
                minimumValue,
                minimumInclusive,
                description,
                maximumValue,
                maximumInclusive);
        }
    }
}
