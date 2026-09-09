using System;
using System.Globalization;
using UnityEditor;
using UnityEngine;

namespace TerraKit.Editor
{
    internal sealed class TerraKitBackendGeneratorWindow : EditorWindow
    {
        [SerializeField] private TerraKitGraphAsset _graph;
        [SerializeField] private string _seed = "12345";
        [SerializeField] private long _regionX;
        [SerializeField] private long _regionY;
        [SerializeField] private int _lodLevel;
        [SerializeField] private int _cellWidth = 16;
        [SerializeField] private int _cellHeight = 16;
        [SerializeField] private Vector2 _spacing = Vector2.one;

        private Vector2 _scrollPosition;

        [MenuItem("Tools/TerraKit/Backend Generator", priority = 220)]
        private static void Open()
        {
            TerraKitBackendGeneratorWindow window =
                GetWindow<TerraKitBackendGeneratorWindow>("Backend Generator");
            window.minSize = new Vector2(430, 520);
            window.TryUseSelectedGraph();
            window.Show();
        }

        private void OnEnable()
        {
            TryUseSelectedGraph();
        }

        private void OnGUI()
        {
            _scrollPosition = EditorGUILayout.BeginScrollView(_scrollPosition);

            EditorGUILayout.Space(8);
            using (new EditorGUILayout.HorizontalScope())
            {
                GUILayout.Space(12);
                using (new EditorGUILayout.VerticalScope())
                {
                    DrawHeader();
                    EditorGUILayout.Space(8);

                    DrawGraphSection();
                    EditorGUILayout.Space(6);

                    DrawRequestSection();

                    string validationMessage;
                    ulong parsedSeed;
                    bool isValid = TryValidate(out parsedSeed, out validationMessage);

                    EditorGUILayout.Space(6);
                    DrawOutputEstimate(isValid, validationMessage);

                    EditorGUILayout.Space(6);
                    DrawGenerateSection(isValid, parsedSeed);

                    EditorGUILayout.Space(6);
                    DrawSaveSection();

                    EditorGUILayout.Space(8);
                    DrawFooter();
                    EditorGUILayout.Space(12);
                }
                GUILayout.Space(12);
            }

            EditorGUILayout.EndScrollView();
        }

        private static void DrawHeader()
        {
            EditorGUILayout.LabelField("Backend Generator", EditorStyles.largeLabel);
            EditorGUILayout.LabelField(
                "Generate Unity terrain meshes from a backend-compatible TerraKit graph.",
                EditorStyles.wordWrappedMiniLabel);
        }

        private void DrawGraphSection()
        {
            using (new EditorGUILayout.VerticalScope(EditorStyles.helpBox))
            {
                EditorGUILayout.LabelField("1  Graph", EditorStyles.boldLabel);
                EditorGUILayout.LabelField(
                    "Choose the graph that the backend runtime will execute.",
                    EditorStyles.wordWrappedMiniLabel);
                EditorGUILayout.Space(5);

                DrawGraphAssetField();

                TerraKitGraphAsset projectSelection =
                    Selection.activeObject as TerraKitGraphAsset;
                using (new EditorGUI.DisabledScope(projectSelection == null))
                {
                    if (GUILayout.Button(
                            new GUIContent(
                                "Use Project Selection",
                                "Copy the TerraKit Graph Asset currently selected " +
                                "in the Project window into the Graph field."),
                            GUILayout.Height(26)))
                    {
                        _graph = projectSelection;
                    }
                }

                EditorGUILayout.LabelField(
                    projectSelection == null
                        ? "Project selection: no TerraKit Graph Asset selected."
                        : "Project selection: " + projectSelection.name,
                    EditorStyles.wordWrappedMiniLabel);

                EditorGUILayout.Space(3);
                EditorGUILayout.LabelField(
                    "Runtime request settings are external to graph stages. " +
                    "This integration wrapper is not a Plugin SDK.",
                    EditorStyles.wordWrappedMiniLabel);
            }
        }

        private void DrawGraphAssetField()
        {
            using (new EditorGUILayout.HorizontalScope())
            {
                EditorGUILayout.PrefixLabel(
                    new GUIContent(
                        "Graph",
                        "Backend-compatible TerraKit graph to execute."));

                if (EditorGUILayout.DropdownButton(
                        new GUIContent(
                            GetGraphAssetDisplayName(_graph),
                            "Choose a TerraKit Graph Asset."),
                        FocusType.Keyboard))
                {
                    ShowGraphAssetMenu();
                }
            }
        }

        private static string GetGraphAssetDisplayName(
            TerraKitGraphAsset graph)
        {
            return graph == null
                ? "None (TerraKit Graph Asset)"
                : graph.name + " (TerraKit Graph Asset)";
        }

        private void ShowGraphAssetMenu()
        {
            var menu = new GenericMenu();
            menu.AddItem(
                new GUIContent("None"),
                _graph == null,
                () =>
                {
                    _graph = null;
                    Repaint();
                });
            menu.AddSeparator(string.Empty);

            string[] graphGuids =
                AssetDatabase.FindAssets("t:TerraKitGraphAsset");
            Array.Sort(graphGuids, StringComparer.Ordinal);

            if (graphGuids.Length == 0)
            {
                menu.AddDisabledItem(
                    new GUIContent("No TerraKit Graph Assets found"));
            }

            foreach (string guid in graphGuids)
            {
                string assetPath = AssetDatabase.GUIDToAssetPath(guid);
                TerraKitGraphAsset graph =
                    AssetDatabase.LoadAssetAtPath<TerraKitGraphAsset>(
                        assetPath);
                if (graph == null)
                {
                    continue;
                }

                string menuLabel = graph.name.Replace("/", "⁄");
                menu.AddItem(
                    new GUIContent(menuLabel),
                    _graph == graph,
                    () =>
                    {
                        _graph = graph;
                        GUI.FocusControl(null);
                        Repaint();
                    });
            }

            menu.ShowAsContext();
        }

        private void DrawRequestSection()
        {
            using (new EditorGUILayout.VerticalScope(EditorStyles.helpBox))
            {
                EditorGUILayout.LabelField("2  Request Settings", EditorStyles.boldLabel);
                EditorGUILayout.LabelField(
                    "Set the deterministic seed, target region and mesh resolution.",
                    EditorStyles.wordWrappedMiniLabel);
                EditorGUILayout.Space(5);

                float previousLabelWidth = EditorGUIUtility.labelWidth;
                EditorGUIUtility.labelWidth = 108f;
                try
                {
                    EditorGUILayout.LabelField("Coordinates", EditorStyles.miniBoldLabel);
                    _seed = EditorGUILayout.TextField(
                        new GUIContent(
                            "Seed",
                            "Unsigned 64-bit deterministic generation seed."),
                        _seed);
                    _regionX = EditorGUILayout.LongField(
                        new GUIContent("Region X", "Horizontal region coordinate."),
                        _regionX);
                    _regionY = EditorGUILayout.LongField(
                        new GUIContent(
                            "Region Y",
                            "Second horizontal region coordinate, mapped to world Z."),
                        _regionY);
                    _lodLevel = EditorGUILayout.IntField(
                        new GUIContent(
                            "LOD",
                            "Each level halves the cell resolution while preserving world size."),
                        _lodLevel);

                    EditorGUILayout.Space(6);
                    EditorGUILayout.LabelField("Region Layout", EditorStyles.miniBoldLabel);
                    _cellWidth = EditorGUILayout.IntField(
                        new GUIContent("Cell Width", "Highest-detail cell count along X."),
                        _cellWidth);
                    _cellHeight = EditorGUILayout.IntField(
                        new GUIContent("Cell Height", "Highest-detail cell count along Z."),
                        _cellHeight);
                    _spacing = EditorGUILayout.Vector2Field(
                        new GUIContent(
                            "Base Spacing",
                            "World-space distance between highest-detail samples."),
                        _spacing);
                }
                finally
                {
                    EditorGUIUtility.labelWidth = previousLabelWidth;
                }
            }
        }

        private void DrawOutputEstimate(bool isValid, string validationMessage)
        {
            if (!isValid)
            {
                EditorGUILayout.HelpBox(validationMessage, MessageType.Warning);
                return;
            }

            long scale = 1L << _lodLevel;
            long pointWidth = _cellWidth / scale + 1L;
            long pointHeight = _cellHeight / scale + 1L;
            long vertexCount = pointWidth * pointHeight;
            long triangleCount =
                (_cellWidth / scale) * (long)(_cellHeight / scale) * 2L;

            using (new EditorGUILayout.VerticalScope(EditorStyles.helpBox))
            {
                EditorGUILayout.LabelField("Output Estimate", EditorStyles.boldLabel);

                float previousLabelWidth = EditorGUIUtility.labelWidth;
                EditorGUIUtility.labelWidth = 140f;
                try
                {
                    EditorGUILayout.LabelField(
                        "Resolved Samples",
                        pointWidth + " x " + pointHeight);
                    EditorGUILayout.LabelField(
                        "Expected Vertices",
                        vertexCount.ToString(CultureInfo.InvariantCulture));
                    EditorGUILayout.LabelField(
                        "Expected Triangles",
                        triangleCount.ToString(CultureInfo.InvariantCulture));
                }
                finally
                {
                    EditorGUIUtility.labelWidth = previousLabelWidth;
                }
            }
        }

        private void DrawGenerateSection(bool isValid, ulong parsedSeed)
        {
            using (new EditorGUILayout.VerticalScope(EditorStyles.helpBox))
            {
                EditorGUILayout.LabelField("3  Generate Preview", EditorStyles.boldLabel);
                EditorGUILayout.LabelField(
                    "Generate one region for a quick check, or a 3 x 3 grid to inspect seams.",
                    EditorStyles.wordWrappedMiniLabel);
                EditorGUILayout.Space(5);

                using (new EditorGUI.DisabledScope(!isValid))
                {
                    if (GUILayout.Button(
                            "Generate Single Region Preview",
                            GUILayout.Height(36)))
                    {
                        TerraKitBackendGenerationRequest generationRequest =
                            CreateGenerationRequest(parsedSeed);
                        TerraKitBackendGenerateMenu.GenerateGraph(
                            _graph,
                            generationRequest);
                    }

                    if (GUILayout.Button(
                            "Generate 3 x 3 Region Preview",
                            GUILayout.Height(36)))
                    {
                        TerraKitBackendGenerationRequest generationRequest =
                            CreateGenerationRequest(parsedSeed);
                        TerraKitBackendGenerateMenu.GenerateRegionGrid(
                            _graph,
                            generationRequest);
                    }
                }
            }
        }

        private void DrawSaveSection()
        {
            bool hasSinglePreview =
                TerraKitBackendGenerateMenu.HasPreviewMesh();
            bool hasGridPreview =
                TerraKitBackendGenerateMenu.HasGridPreview();

            using (new EditorGUILayout.VerticalScope(EditorStyles.helpBox))
            {
                EditorGUILayout.LabelField("4  Save Output", EditorStyles.boldLabel);
                EditorGUILayout.LabelField(
                    "Save generated meshes as reusable Unity project assets.",
                    EditorStyles.wordWrappedMiniLabel);
                EditorGUILayout.Space(5);

                using (new EditorGUI.DisabledScope(!hasSinglePreview))
                {
                    if (GUILayout.Button(
                            "Save Single Mesh Asset",
                            GUILayout.Height(30)))
                    {
                        TerraKitBackendGenerateMenu.SaveCurrentPreviewMeshAsset(
                            _graph == null ? string.Empty : _graph.name);
                    }
                }

                using (new EditorGUI.DisabledScope(!hasGridPreview))
                {
                    if (GUILayout.Button(
                            "Save 3 x 3 Meshes + Prefab",
                            GUILayout.Height(30)))
                    {
                        TerraKitBackendGenerateMenu.SaveCurrentGridAssetsAndPrefab(
                            _graph == null ? string.Empty : _graph.name);
                    }
                }

                EditorGUILayout.Space(3);
                EditorGUILayout.LabelField(
                    hasSinglePreview || hasGridPreview
                        ? "Available outputs are enabled above."
                        : "Generate a preview to enable its save action.",
                    EditorStyles.wordWrappedMiniLabel);
            }
        }

        private void DrawFooter()
        {
            using (new EditorGUILayout.HorizontalScope())
            {
                GUILayout.FlexibleSpace();
                if (GUILayout.Button(
                        "Reset Settings",
                        GUILayout.Width(140),
                        GUILayout.Height(25)))
                {
                    ResetGenerationSettings();
                }
            }
        }

        private bool TryValidate(out ulong parsedSeed, out string message)
        {
            parsedSeed = 0;
            if (_graph == null)
            {
                message = "Choose a TerraKit Graph Asset.";
                return false;
            }
            if (!ulong.TryParse(
                    (_seed ?? string.Empty).Trim(),
                    NumberStyles.Integer,
                    CultureInfo.InvariantCulture,
                    out parsedSeed))
            {
                message = "Seed must be a whole number from 0 through 18446744073709551615.";
                return false;
            }
            if (_lodLevel < 0 || _lodLevel > 30)
            {
                message = "LOD must be between 0 and 30.";
                return false;
            }
            if (_cellWidth < 1 || _cellHeight < 1 ||
                _cellWidth > 4096 || _cellHeight > 4096)
            {
                message = "Cell Width and Cell Height must be between 1 and 4096.";
                return false;
            }
            if (float.IsNaN(_spacing.x) || float.IsInfinity(_spacing.x) || _spacing.x <= 0 ||
                float.IsNaN(_spacing.y) || float.IsInfinity(_spacing.y) || _spacing.y <= 0)
            {
                message = "Both Base Spacing values must be finite and greater than zero.";
                return false;
            }

            long lodScale = 1L << _lodLevel;
            if (_cellWidth % lodScale != 0 || _cellHeight % lodScale != 0)
            {
                message =
                    "At LOD " + _lodLevel + ", both cell dimensions must be divisible by " +
                    lodScale + ".";
                return false;
            }

            message = string.Empty;
            return true;
        }

        private void TryUseSelectedGraph()
        {
            if (_graph == null && Selection.activeObject is TerraKitGraphAsset)
            {
                _graph = Selection.activeObject as TerraKitGraphAsset;
            }
        }

        private void ResetGenerationSettings()
        {
            _seed = "12345";
            _regionX = 0;
            _regionY = 0;
            _lodLevel = 0;
            _cellWidth = 16;
            _cellHeight = 16;
            _spacing = Vector2.one;
            GUI.FocusControl(null);
        }

        private TerraKitBackendGenerationRequest CreateGenerationRequest(ulong parsedSeed)
        {
            return new TerraKitBackendGenerationRequest(
                parsedSeed,
                _regionX,
                _regionY,
                checked((ushort)_lodLevel),
                checked((uint)_cellWidth),
                checked((uint)_cellHeight),
                _spacing.x,
                _spacing.y);
        }
    }
}
