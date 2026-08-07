using System;
using System.Collections.Generic;
using System.Linq;
using UnityEditor;
using UnityEditor.Experimental.GraphView;
using UnityEngine;

namespace TerraKit.Editor
{
    internal sealed class TerraKitNodeSearchProvider : ScriptableObject, ISearchWindowProvider
    {
        private const string RecentNodesKey = "TerraKit.NodeSearch.Recent";
        private const int RecentNodeLimit = 6;

        private static readonly string[] CategoryOrder =
        {
            "Input",
            "Terrain",
            "Mask",
            "Biome",
            "Placement",
            "Output"
        };

        private TerraKitGraphView _graphView;
        private Vector2 _graphPosition;

        public void Initialize(TerraKitGraphView graphView, Vector2 graphPosition)
        {
            _graphView = graphView;
            _graphPosition = graphPosition;
        }

        public List<SearchTreeEntry> CreateSearchTree(SearchWindowContext context)
        {
            var backendDefinitions = TerraKitNodeRegistry.All
                .Where(definition => TerraKitNodeRegistry.IsBackendExecutable(definition.TypeId))
                .ToList();
            var tree = new List<SearchTreeEntry>
            {
                new SearchTreeGroupEntry(new GUIContent("Create TerraKit Node"), 0)
            };

            var recentDefinitions = LoadRecentDefinitions();
            if (recentDefinitions.Count > 0)
            {
                tree.Add(new SearchTreeGroupEntry(new GUIContent("Recently Used"), 1));
                AddDefinitionEntries(tree, recentDefinitions, 2, false);
            }

            foreach (string category in GetOrderedCategories(backendDefinitions))
            {
                var categoryDefinitions = backendDefinitions
                    .Where(definition => definition.Category == category)
                    .OrderBy(definition => definition.DisplayName)
                    .ToList();

                if (categoryDefinitions.Count == 0)
                {
                    continue;
                }

                tree.Add(new SearchTreeGroupEntry(new GUIContent(category), 1));
                AddDefinitionEntries(tree, categoryDefinitions, 2, true);
            }

            return tree;
        }

        public bool OnSelectEntry(SearchTreeEntry entry, SearchWindowContext context)
        {
            var definition = entry.userData as ITerraKitNodeDefinition;
            if (_graphView == null || definition == null)
            {
                return false;
            }

            _graphView.CreateNode(definition, _graphPosition);
            RecordRecentNode(definition.TypeId);
            return true;
        }

        private static void AddDefinitionEntries(
            ICollection<SearchTreeEntry> tree,
            IEnumerable<ITerraKitNodeDefinition> definitions,
            int level,
            bool includeInSearch)
        {
            foreach (var definition in definitions)
            {
                string summary = GetMenuSummary(definition.TypeId);
                string label = string.IsNullOrWhiteSpace(summary)
                    ? definition.DisplayName
                    : definition.DisplayName + " — " + summary;

                // SearchTreeEntry.name is read-only and always comes from the visible
                // GUIContent text. Invisible separators preserve the recent menu's
                // appearance while preventing normal multi-letter searches from also
                // matching the recent alias of the same node.
                string entryLabel = includeInSearch
                    ? label
                    : string.Join("\u2063", label.ToCharArray());

                tree.Add(new SearchTreeEntry(new GUIContent(
                    entryLabel,
                    definition.Description))
                {
                    level = level,
                    userData = definition
                });
            }
        }

        private static IEnumerable<string> GetOrderedCategories(
            IEnumerable<ITerraKitNodeDefinition> definitions)
        {
            foreach (string category in CategoryOrder)
            {
                yield return category;
            }

            foreach (string category in definitions
                .Select(definition => definition.Category)
                .Distinct()
                .Where(category => !CategoryOrder.Contains(category))
                .OrderBy(category => category))
            {
                yield return category;
            }
        }

        private static List<ITerraKitNodeDefinition> LoadRecentDefinitions()
        {
            return LoadRecentTypeIds()
                .Select(typeId =>
                {
                    ITerraKitNodeDefinition definition;
                    return TerraKitNodeRegistry.TryGet(typeId, out definition)
                        ? definition
                        : null;
                })
                .Where(definition =>
                    definition != null &&
                    TerraKitNodeRegistry.IsBackendExecutable(definition.TypeId))
                .ToList();
        }

        private static string GetMenuSummary(string typeId)
        {
            switch (typeId)
            {
                case "terrakit.height.flat":
                    return "Creates a constant height field.";
                case "terrakit.height.noise":
                    return "Applies deterministic height noise.";
                case "terrakit.mesh.height_field":
                    return "Converts a height field into a mesh.";
                default:
                    return string.Empty;
            }
        }

        private static List<string> LoadRecentTypeIds()
        {
            string savedValue = EditorPrefs.GetString(RecentNodesKey, string.Empty);
            return savedValue
                .Split(new[] { '|' }, StringSplitOptions.RemoveEmptyEntries)
                .Distinct()
                .Take(RecentNodeLimit)
                .ToList();
        }

        private static void RecordRecentNode(string typeId)
        {
            var recentTypeIds = LoadRecentTypeIds();
            recentTypeIds.Remove(typeId);
            recentTypeIds.Insert(0, typeId);

            EditorPrefs.SetString(
                RecentNodesKey,
                string.Join("|", recentTypeIds.Take(RecentNodeLimit)));
        }
    }
}
