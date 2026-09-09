namespace TerraKit
{
    /// <summary>
    /// Data categories that can flow through TerraKit node ports.
    /// The editor uses this for connection compatibility; the compiler uses it for validation.
    /// </summary>
    public enum TerraKitPortType
    {
        Any,
        Seed,
        ChunkCoordinate,
        HeightMap,
        BiomeMap,
        Mask,
        ObjectSet,
        Mesh,
        ChunkData
    }
}
