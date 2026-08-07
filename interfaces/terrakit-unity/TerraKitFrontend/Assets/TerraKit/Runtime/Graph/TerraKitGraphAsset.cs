using System.Collections.Generic;
using UnityEngine;

namespace TerraKit
{
    /// <summary>
    /// Serializable node graph asset. This is intentionally independent from Unity's GraphView UI.
    /// </summary>
    [CreateAssetMenu(menuName = "TerraKit/Generation Graph", fileName = "TerraKitGenerationGraph")]
    public sealed class TerraKitGraphAsset : ScriptableObject
    {
        public List<TerraKitNodeData> nodes = new List<TerraKitNodeData>();
        public List<TerraKitEdgeData> edges = new List<TerraKitEdgeData>();
    }
}
