using System;
using System.Collections.Generic;
using System.IO;
using UnityEditor;
using UnityEngine;
using UnityEngine.Rendering;

namespace TerraKit.Editor
{
    /// <summary>
    /// Handles backend terrain preview generation and asset export in the Unity Editor.
    /// </summary>
    internal static class TerraKitBackendGenerateMenu
    {
        private const string PreviewObjectName = "TerraKit Backend Preview";
        private const string GridPreviewObjectName = "TerraKit Backend Preview Grid";

        [MenuItem("Tools/TerraKit/Generate Selected Backend Graph", priority = 221)]
        private static void GenerateSelectedGraph()
        {
            var graph = Selection.activeObject as TerraKitGraphAsset;
            if (graph == null)
            {
                EditorUtility.DisplayDialog(
                    "TerraKit Backend",
                    "Select a TerraKit Graph Asset in the Project window, then run this command again.",
                    "OK");
                return;
            }

            GenerateGraph(graph, TerraKitBackendGenerationRequest.Default);
        }

        // Generates a single backend region and displays it as a Unity mesh.
        internal static void GenerateGraph(
            TerraKitGraphAsset graph,
            TerraKitBackendGenerationRequest generationRequest)
        {
            try
            {
                TerraKitGeneratedMeshData generated =
                    new TerraKitBackendGraphRunner().Generate(graph, generationRequest);
                SetSceneObjectActive(GridPreviewObjectName, false);
                GameObject preview = GetOrCreatePreviewObject();
                ApplyMesh(preview, generated);
                Selection.activeGameObject = preview;
                SceneView.lastActiveSceneView?.FrameSelected();

                string message =
                    "TerraKit backend generated a Unity mesh.\n\n" +
                    "Graph: " + graph.name + "\n" +
                    "Vertices: " + generated.Positions.Length + "\n" +
                    "Triangles: " + (generated.Indices.Length / 3) + "\n" +
                    "Seed: " + generationRequest.Seed + "\n" +
                    "Region: (" + generationRequest.RegionX + ", " +
                    generationRequest.RegionY + ")\n" +
                    "LOD: " + generationRequest.LodLevel + "\n" +
                    "Cells: " + generationRequest.CellWidth + " x " +
                    generationRequest.CellHeight + "\n" +
                    "Spacing: " + generationRequest.SpacingX + " x " +
                    generationRequest.SpacingY;
                Debug.Log("[TerraKit] " + message.Replace("\n", " "), preview);
                EditorUtility.DisplayDialog("TerraKit Backend", message, "OK");
            }
            catch (Exception exception)
            {
                Debug.LogException(exception);
                EditorUtility.DisplayDialog(
                    "TerraKit Backend Generation Failed",
                    exception.Message,
                    "OK");
            }
        }

        // Generates a 3 x 3 grid centred on the selected region.
        internal static void GenerateRegionGrid(
            TerraKitGraphAsset graph,
            TerraKitBackendGenerationRequest centerRequest)
        {
            try
            {
                var requests = new List<TerraKitBackendGenerationRequest>(9);
                for (int yOffset = -1; yOffset <= 1; yOffset++)
                {
                    for (int xOffset = -1; xOffset <= 1; xOffset++)
                    {
                        requests.Add(new TerraKitBackendGenerationRequest(
                            centerRequest.Seed,
                            checked(centerRequest.RegionX + xOffset),
                            checked(centerRequest.RegionY + yOffset),
                            centerRequest.LodLevel,
                            centerRequest.CellWidth,
                            centerRequest.CellHeight,
                            centerRequest.SpacingX,
                            centerRequest.SpacingY));
                    }
                }

                IReadOnlyList<TerraKitGeneratedMeshData> generatedMeshes =
                    new TerraKitBackendGraphRunner().Generate(graph, requests);

                SetSceneObjectActive(PreviewObjectName, false);
                GameObject gridRoot = GetOrCreateGridPreviewObject();
                ClearGridChildren(gridRoot);

                int meshIndex = 0;
                long totalVertices = 0;
                long totalTriangles = 0;
                for (int yOffset = -1; yOffset <= 1; yOffset++)
                {
                    for (int xOffset = -1; xOffset <= 1; xOffset++)
                    {
                        long regionX = centerRequest.RegionX + xOffset;
                        long regionY = centerRequest.RegionY + yOffset;
                        TerraKitGeneratedMeshData generated = generatedMeshes[meshIndex++];
                        var child = new GameObject(
                            "TerraKit Region (" + regionX + ", " + regionY + ")");
                        Undo.RegisterCreatedObjectUndo(child, "Create TerraKit Region Preview");
                        child.transform.SetParent(gridRoot.transform, false);
                        ApplyMesh(child, generated);
                        child.GetComponent<MeshFilter>().sharedMesh.name =
                            "TerraKit Backend Region " + regionX + " " + regionY;
                        totalVertices += generated.Positions.Length;
                        totalTriangles += generated.Indices.Length / 3;
                    }
                }

                Selection.activeGameObject = gridRoot;
                SceneView.lastActiveSceneView?.FrameSelected();

                string message =
                    "TerraKit backend generated a 3 x 3 region preview.\n\n" +
                    "Graph: " + graph.name + "\n" +
                    "Center Region: (" + centerRequest.RegionX + ", " +
                    centerRequest.RegionY + ")\n" +
                    "Regions: 9\n" +
                    "Total Vertices: " + totalVertices + "\n" +
                    "Total Triangles: " + totalTriangles + "\n" +
                    "Seed: " + centerRequest.Seed + "\n" +
                    "LOD: " + centerRequest.LodLevel;
                Debug.Log("[TerraKit] " + message.Replace("\n", " "), gridRoot);
                EditorUtility.DisplayDialog("TerraKit Backend", message, "OK");
            }
            catch (Exception exception)
            {
                Debug.LogException(exception);
                EditorUtility.DisplayDialog(
                    "TerraKit Backend Grid Generation Failed",
                    exception.Message,
                    "OK");
            }
        }

        internal static bool HasPreviewMesh()
        {
            GameObject preview = FindSceneObject(PreviewObjectName);
            MeshFilter filter = preview == null ? null : preview.GetComponent<MeshFilter>();
            return preview != null && preview.activeInHierarchy &&
                   filter != null && filter.sharedMesh != null;
        }

        internal static bool HasGridPreview()
        {
            GameObject gridRoot = FindSceneObject(GridPreviewObjectName);
            if (gridRoot == null || !gridRoot.activeInHierarchy || gridRoot.transform.childCount == 0)
            {
                return false;
            }

            for (int index = 0; index < gridRoot.transform.childCount; index++)
            {
                MeshFilter filter = gridRoot.transform.GetChild(index).GetComponent<MeshFilter>();
                if (filter == null || filter.sharedMesh == null)
                {
                    return false;
                }
            }

            return true;
        }

        // Saves the nine generated meshes and their parent object as reusable Unity assets.
        internal static void SaveCurrentGridAssetsAndPrefab(string suggestedGraphName)
        {
            try
            {
                GameObject gridRoot = FindSceneObject(GridPreviewObjectName);
                if (!HasGridPreview() || gridRoot == null)
                {
                    EditorUtility.DisplayDialog(
                        "TerraKit Backend",
                        "Generate a 3 x 3 region preview before exporting it.",
                        "OK");
                    return;
                }

                EnsureGeneratedAssetFolder();
                string safeGraphName = string.IsNullOrWhiteSpace(suggestedGraphName)
                    ? "TerraKitBackend"
                    : suggestedGraphName.Trim();
                string prefabPath = EditorUtility.SaveFilePanelInProject(
                    "Save TerraKit 3 x 3 Prefab",
                    safeGraphName + "_3x3",
                    "prefab",
                    "Choose where to save the 3 x 3 Prefab and its Mesh asset folder.",
                    "Assets/TerraKitGenerated");
                if (string.IsNullOrEmpty(prefabPath))
                {
                    return;
                }

                string parentFolder = Path.GetDirectoryName(prefabPath).Replace('\\', '/');
                string prefabName = Path.GetFileNameWithoutExtension(prefabPath);
                string meshFolderName = prefabName + "_Meshes";
                string meshFolderPath = parentFolder + "/" + meshFolderName;

                UnityEngine.Object existingPrefabObject =
                    AssetDatabase.LoadMainAssetAtPath(prefabPath);
                if (existingPrefabObject != null && !(existingPrefabObject is GameObject))
                {
                    throw new InvalidOperationException(
                        "The selected Prefab path is occupied by a non-Prefab asset: " + prefabPath);
                }

                var meshPaths = new List<string>(gridRoot.transform.childCount);
                bool willReplaceExisting = existingPrefabObject != null;
                for (int index = 0; index < gridRoot.transform.childCount; index++)
                {
                    GameObject child = gridRoot.transform.GetChild(index).gameObject;
                    string meshAssetName = MakeSafeAssetName(child.name) + ".asset";
                    string meshPath = meshFolderPath + "/" + meshAssetName;
                    meshPaths.Add(meshPath);

                    UnityEngine.Object existingMeshObject =
                        AssetDatabase.LoadMainAssetAtPath(meshPath);
                    if (existingMeshObject != null && !(existingMeshObject is Mesh))
                    {
                        throw new InvalidOperationException(
                            "A generated Mesh path is occupied by another asset type: " + meshPath);
                    }
                    willReplaceExisting |= existingMeshObject != null;
                }

                if (willReplaceExisting)
                {
                    bool replace = EditorUtility.DisplayDialog(
                        "Update Existing 3 x 3 Export?",
                        "One or more generated assets already exist. Update the nine Mesh assets " +
                        "and Prefab while preserving their existing references?",
                        "Update",
                        "Cancel");
                    if (!replace)
                    {
                        return;
                    }
                }

                if (!AssetDatabase.IsValidFolder(meshFolderPath))
                {
                    string createdFolderGuid = AssetDatabase.CreateFolder(
                        parentFolder,
                        meshFolderName);
                    if (string.IsNullOrEmpty(createdFolderGuid))
                    {
                        throw new InvalidOperationException(
                            "Unity could not create the Mesh asset folder: " + meshFolderPath);
                    }
                }

                for (int index = 0; index < gridRoot.transform.childCount; index++)
                {
                    GameObject child = gridRoot.transform.GetChild(index).gameObject;
                    MeshFilter filter = child.GetComponent<MeshFilter>();
                    Mesh savedMesh = SaveOrUpdateMeshAsset(filter.sharedMesh, meshPaths[index]);
                    Undo.RecordObject(filter, "Assign Exported TerraKit Region Mesh");
                    filter.sharedMesh = savedMesh;
                    EditorUtility.SetDirty(filter);
                    EditorUtility.SetDirty(child);
                }

                AssetDatabase.SaveAssets();
                bool prefabSaved;
                GameObject prefabAsset = PrefabUtility.SaveAsPrefabAsset(
                    gridRoot,
                    prefabPath,
                    out prefabSaved);
                if (!prefabSaved || prefabAsset == null)
                {
                    throw new InvalidOperationException(
                        "Unity could not save the TerraKit grid Prefab: " + prefabPath);
                }

                AssetDatabase.SaveAssets();
                Selection.activeObject = prefabAsset;
                EditorGUIUtility.PingObject(prefabAsset);
                Debug.Log(
                    "[TerraKit] Saved 3 x 3 backend grid Prefab and " +
                    gridRoot.transform.childCount + " Mesh assets: " + prefabPath,
                    prefabAsset);
                EditorUtility.DisplayDialog(
                    "TerraKit Backend",
                    "3 x 3 export saved successfully.\n\nPrefab:\n" + prefabPath +
                    "\n\nMesh folder:\n" + meshFolderPath,
                    "OK");
            }
            catch (Exception exception)
            {
                Debug.LogException(exception);
                EditorUtility.DisplayDialog(
                    "TerraKit 3 x 3 Export Failed",
                    exception.Message,
                    "OK");
            }
        }

        internal static void SaveCurrentPreviewMeshAsset(string suggestedGraphName)
        {
            GameObject preview = FindSceneObject(PreviewObjectName);
            MeshFilter filter = preview == null ? null : preview.GetComponent<MeshFilter>();
            Mesh source = filter == null ? null : filter.sharedMesh;
            if (source == null)
            {
                EditorUtility.DisplayDialog(
                    "TerraKit Backend",
                    "Generate a backend preview before saving its Mesh asset.",
                    "OK");
                return;
            }

            EnsureGeneratedAssetFolder();
            string safeGraphName = string.IsNullOrWhiteSpace(suggestedGraphName)
                ? "TerraKitBackend"
                : suggestedGraphName.Trim();
            string assetPath = EditorUtility.SaveFilePanelInProject(
                "Save TerraKit Backend Mesh",
                safeGraphName + "_Mesh",
                "asset",
                "Choose where to save the generated Unity Mesh asset.",
                "Assets/TerraKitGenerated");
            if (string.IsNullOrEmpty(assetPath))
            {
                return;
            }

            string sourcePath = AssetDatabase.GetAssetPath(source);
            if (sourcePath == assetPath)
            {
                Selection.activeObject = source;
                EditorGUIUtility.PingObject(source);
                EditorUtility.DisplayDialog(
                    "TerraKit Backend",
                    "This preview already uses the selected Mesh asset.",
                    "OK");
                return;
            }

            string assetName = Path.GetFileNameWithoutExtension(assetPath);
            Mesh existing = AssetDatabase.LoadAssetAtPath<Mesh>(assetPath);
            Mesh savedMesh;
            if (existing != null)
            {
                bool replace = EditorUtility.DisplayDialog(
                    "Replace Existing Mesh?",
                    "A Mesh asset already exists at:\n\n" + assetPath +
                    "\n\nReplace its generated mesh data? Existing references will be preserved.",
                    "Replace",
                    "Cancel");
                if (!replace)
                {
                    return;
                }

                EditorUtility.CopySerialized(source, existing);
                existing.name = assetName;
                EditorUtility.SetDirty(existing);
                savedMesh = existing;
            }
            else
            {
                savedMesh = UnityEngine.Object.Instantiate(source);
                savedMesh.name = assetName;
                AssetDatabase.CreateAsset(savedMesh, assetPath);
            }

            Undo.RecordObject(filter, "Assign Saved TerraKit Mesh");
            filter.sharedMesh = savedMesh;
            EditorUtility.SetDirty(filter);
            EditorUtility.SetDirty(preview);
            AssetDatabase.SaveAssets();

            Selection.activeObject = savedMesh;
            EditorGUIUtility.PingObject(savedMesh);
            Debug.Log("[TerraKit] Saved backend Mesh asset: " + assetPath, savedMesh);
            EditorUtility.DisplayDialog(
                "TerraKit Backend",
                "Mesh asset saved successfully.\n\n" + assetPath,
                "OK");
        }

        [MenuItem("Tools/TerraKit/Generate Selected Backend Graph", true)]
        private static bool ValidateGenerateSelectedGraph()
        {
            return Selection.activeObject is TerraKitGraphAsset;
        }

        private static GameObject GetOrCreatePreviewObject()
        {
            GameObject preview = FindSceneObject(PreviewObjectName);
            if (preview != null)
            {
                Undo.RegisterFullObjectHierarchyUndo(preview, "Regenerate TerraKit Backend Preview");
                preview.SetActive(true);
                return preview;
            }

            preview = new GameObject(PreviewObjectName);
            Undo.RegisterCreatedObjectUndo(preview, "Create TerraKit Backend Preview");
            preview.AddComponent<MeshFilter>();
            preview.AddComponent<MeshRenderer>();
            return preview;
        }

        private static GameObject GetOrCreateGridPreviewObject()
        {
            GameObject gridRoot = FindSceneObject(GridPreviewObjectName);
            if (gridRoot == null)
            {
                gridRoot = new GameObject(GridPreviewObjectName);
                Undo.RegisterCreatedObjectUndo(gridRoot, "Create TerraKit Backend Preview Grid");
            }
            else
            {
                Undo.RegisterFullObjectHierarchyUndo(
                    gridRoot,
                    "Regenerate TerraKit Backend Preview Grid");
                gridRoot.SetActive(true);
            }

            gridRoot.transform.position = Vector3.zero;
            gridRoot.transform.rotation = Quaternion.identity;
            gridRoot.transform.localScale = Vector3.one;
            return gridRoot;
        }

        // Converts backend mesh data into a Unity Mesh and assigns it to the preview object.
        private static void ApplyMesh(GameObject preview, TerraKitGeneratedMeshData generated)
        {
            MeshFilter filter = preview.GetComponent<MeshFilter>();
            if (filter == null)
            {
                filter = Undo.AddComponent<MeshFilter>(preview);
            }
            MeshRenderer renderer = preview.GetComponent<MeshRenderer>();
            if (renderer == null)
            {
                renderer = Undo.AddComponent<MeshRenderer>(preview);
            }
            renderer.sharedMaterial = GetRenderPipelineMaterial();

            var mesh = new Mesh { name = "TerraKit Backend Generated Mesh" };
            if (generated.Positions.Length > ushort.MaxValue)
            {
                mesh.indexFormat = IndexFormat.UInt32;
            }
            mesh.vertices = generated.Positions;
            mesh.triangles = generated.Indices;
            if (generated.Normals.Length == generated.Positions.Length)
            {
                mesh.normals = generated.Normals;
            }
            else
            {
                mesh.RecalculateNormals();
            }
            if (generated.Texcoords.Length == generated.Positions.Length)
            {
                mesh.uv = generated.Texcoords;
            }
            mesh.RecalculateBounds();

            Mesh previous = filter.sharedMesh;
            filter.sharedMesh = mesh;
            preview.transform.position = generated.Origin;
            EditorUtility.SetDirty(preview);

            // Remove the previous temporary mesh to avoid leaking editor memory.
            if (previous != null && previous.name == "TerraKit Backend Generated Mesh" &&
                !AssetDatabase.Contains(previous))
            {
                UnityEngine.Object.DestroyImmediate(previous);
            }
        }

        private static Material GetRenderPipelineMaterial()
        {
            RenderPipelineAsset renderPipeline = GraphicsSettings.currentRenderPipeline;
            if (renderPipeline == null)
            {
                renderPipeline = GraphicsSettings.defaultRenderPipeline;
            }

            if (renderPipeline != null && renderPipeline.defaultMaterial != null)
            {
                return renderPipeline.defaultMaterial;
            }

            return AssetDatabase.GetBuiltinExtraResource<Material>("Default-Material.mat");
        }

        private static void ClearGridChildren(GameObject gridRoot)
        {
            for (int index = gridRoot.transform.childCount - 1; index >= 0; index--)
            {
                GameObject child = gridRoot.transform.GetChild(index).gameObject;
                MeshFilter filter = child.GetComponent<MeshFilter>();
                Mesh mesh = filter == null ? null : filter.sharedMesh;
                if (mesh != null && !AssetDatabase.Contains(mesh))
                {
                    Undo.DestroyObjectImmediate(mesh);
                }
                Undo.DestroyObjectImmediate(child);
            }
        }

        private static void SetSceneObjectActive(string objectName, bool active)
        {
            GameObject sceneObject = FindSceneObject(objectName);
            if (sceneObject == null || sceneObject.activeSelf == active)
            {
                return;
            }

            Undo.RecordObject(sceneObject, active ? "Show TerraKit Preview" : "Hide TerraKit Preview");
            sceneObject.SetActive(active);
        }

        private static GameObject FindSceneObject(string objectName)
        {
            GameObject[] objects = Resources.FindObjectsOfTypeAll<GameObject>();
            foreach (GameObject candidate in objects)
            {
                if (candidate.name == objectName &&
                    candidate.scene.IsValid() &&
                    !EditorUtility.IsPersistent(candidate))
                {
                    return candidate;
                }
            }

            return null;
        }

        private static void EnsureGeneratedAssetFolder()
        {
            if (!AssetDatabase.IsValidFolder("Assets/TerraKitGenerated"))
            {
                AssetDatabase.CreateFolder("Assets", "TerraKitGenerated");
            }
        }

        private static Mesh SaveOrUpdateMeshAsset(Mesh source, string assetPath)
        {
            string sourcePath = AssetDatabase.GetAssetPath(source);
            Mesh existing = AssetDatabase.LoadAssetAtPath<Mesh>(assetPath);
            string assetName = Path.GetFileNameWithoutExtension(assetPath);

            if (sourcePath == assetPath && existing != null)
            {
                return existing;
            }
            if (existing != null)
            {
                EditorUtility.CopySerialized(source, existing);
                existing.name = assetName;
                EditorUtility.SetDirty(existing);
                return existing;
            }

            Mesh savedMesh = UnityEngine.Object.Instantiate(source);
            savedMesh.name = assetName;
            AssetDatabase.CreateAsset(savedMesh, assetPath);
            return savedMesh;
        }

        private static string MakeSafeAssetName(string value)
        {
            string result = value
                .Replace("TerraKit Region (", "Region_")
                .Replace(", ", "_")
                .Replace(")", string.Empty)
                .Replace(' ', '_');
            foreach (char invalidCharacter in Path.GetInvalidFileNameChars())
            {
                result = result.Replace(invalidCharacter, '_');
            }
            return result;
        }
    }
}
