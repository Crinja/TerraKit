using System;
using System.Linq;
using UnityEditor;
using UnityEngine;

namespace TerraKit.Editor
{
    internal static class TerraKitGraphEditorUtil
    {
        public static TerraKitNodeData CreateNodeData(ITerraKitNodeDefinition definition, Vector2 position)
        {
            var data = new TerraKitNodeData
            {
                id = Guid.NewGuid().ToString("N"),
                typeId = definition.TypeId,
                displayName = definition.DisplayName,
                position = position,
                size = new Vector2(260, 180)
            };

            EnsureParameterDefaults(data, definition);
            return data;
        }

        public static void EnsureParameterDefaults(TerraKitNodeData data, ITerraKitNodeDefinition definition)
        {
            foreach (var parameter in definition.Parameters)
            {
                var existing = data.parameters.FirstOrDefault(p => p.key == parameter.Key);
                if (existing == null)
                {
                    data.parameters.Add(new TerraKitParameterData
                    {
                        key = parameter.Key,
                        displayName = parameter.DisplayName,
                        type = parameter.Type,
                        value = parameter.DefaultValue
                    });
                }
                else
                {
                    existing.displayName = parameter.DisplayName;
                    existing.type = parameter.Type;
                    if (string.IsNullOrEmpty(existing.value))
                    {
                        existing.value = parameter.DefaultValue;
                    }
                }
            }
        }

        public static TerraKitEdgeData CreateEdgeData(TerraKitEditorPortBinding output, TerraKitEditorPortBinding input)
        {
            return new TerraKitEdgeData
            {
                id = Guid.NewGuid().ToString("N"),
                outputNodeId = output.NodeId,
                outputPortId = output.PortId,
                inputNodeId = input.NodeId,
                inputPortId = input.PortId
            };
        }

        public static void MarkDirty(UnityEngine.Object target)
        {
            if (target == null)
            {
                return;
            }

            EditorUtility.SetDirty(target);
            AssetDatabase.SaveAssets();
        }
    }
}
