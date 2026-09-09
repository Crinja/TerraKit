using System;
using System.IO;
using System.Linq;
using UnityEditor;
using UnityEditor.UIElements;
using UnityEngine;
using UnityEngine.UIElements;

namespace TerraKit.Editor
{
    public sealed class TerraKitGraphWindow : EditorWindow
    {
        private TerraKitGraphAsset _graphAsset;
        private TerraKitGraphView _graphView;
        private VisualElement _inspector;
        private TwoPaneSplitView _bodySplitView;
        private VisualElement _statusPanel;
        private Label _statusIcon;
        private Label _statusType;
        private Label _statusTime;
        private Label _statusSummary;
        private VisualElement _statusDetails;
        private ObjectField _graphField;
        private Label _saveStateLabel;
        
        private Button _saveButton;
        private Button _validateButton;
        private Button _compileButton;
        private Button _frameAllButton;
        private Button _focusButton;
        private Button _resetZoomButton;
        private TerraKitNodeView _selectedNode;

        private enum StatusType
        {
            Info,
            Success,
            Warning,
            Error
        }

        [MenuItem("Tools/TerraKit/Graph Editor")]
        public static void Open()
        {
            var window = GetWindow<TerraKitGraphWindow>();
            window.titleContent = new GUIContent("TerraKit Graph");
            window.minSize = new Vector2(1100, 650);
        }

        private void CreateGUI()
        {
            rootVisualElement.Clear();
            rootVisualElement.AddToClassList("terrakit-root");

            LoadStylesheet();
            BuildToolbar();
            BuildMainLayout();
            BuildConsole();

            ShowEmptyInspector();
        }

        private void BuildToolbar()
        {
            var toolbar = new Toolbar();
            toolbar.AddToClassList("terrakit-toolbar");

            var title = new Label("TerraKit Pipeline Graph");
            title.AddToClassList("terrakit-toolbar-title");
            toolbar.Add(title);

            _graphField = new ObjectField
            {
                objectType = typeof(TerraKitGraphAsset),
                allowSceneObjects = false,
                value = _graphAsset
            };
            _graphField.AddToClassList("terrakit-graph-field");
            _graphField.RegisterValueChangedCallback(evt =>
            {
                _graphAsset = evt.newValue as TerraKitGraphAsset;
                LoadGraphIntoView();
            });
            toolbar.Add(_graphField);

            _saveStateLabel = new Label();
            _saveStateLabel.AddToClassList("terrakit-save-state");
            toolbar.Add(_saveStateLabel);

            toolbar.Add(new Button(CreateGraphAsset) { text = "New Graph" });

            _saveButton = new Button(SaveGraph) { text = "Save" };
            _validateButton = new Button(ValidateGraph) { text = "Validate" };
            _compileButton = new Button(CompileGraph) { text = "Compile" };

            toolbar.Add(_saveButton);
            toolbar.Add(_validateButton);
            toolbar.Add(_compileButton);

            var backendDemoButton = new Button(CreateBackendDemoGraph)
            {
                text = "Backend Demo",
                tooltip = "Create a new backend-executable demo graph without modifying the current graph."
            };
            toolbar.Add(backendDemoButton);

            _frameAllButton = new Button(FrameAllNodes) { text = "Frame All" };
            toolbar.Add(_frameAllButton);

            _focusButton = new Button(FocusSelectedNode)
            {
                text = "Focus",
                tooltip = "Fit the selected node in the canvas. Shortcut: F"
            };
            toolbar.Add(_focusButton);

            _resetZoomButton = new Button(ResetZoom)
            {
                text = "100%",
                tooltip = "Reset zoom to 100% while keeping the current centre. Shortcut: 1"
            };
            toolbar.Add(_resetZoomButton);

            toolbar.Add(new Button(ShowHelp) { text = "Help" });

            rootVisualElement.Add(toolbar);
        }

        private void BuildMainLayout()
        {
            _bodySplitView = new TwoPaneSplitView(
                1,
                210,
                TwoPaneSplitViewOrientation.Vertical)
            {
                viewDataKey = "terrakit-body-split"
            };
            _bodySplitView.AddToClassList("terrakit-body-split");

            var main = new TwoPaneSplitView(
                1,
                360,
                TwoPaneSplitViewOrientation.Horizontal)
            {
                viewDataKey = "terrakit-inspector-split"
            };
            main.AddToClassList("terrakit-main");

            _graphView = new TerraKitGraphView();
            _graphView.StretchToParentSize();
            _graphView.NodeSelected += HandleNodeSelected;
            _graphView.GraphChanged += HandleGraphChanged;

            var graphContainer = new VisualElement();
            graphContainer.AddToClassList("terrakit-graph-container");
            graphContainer.Add(_graphView);

            graphContainer.Add(BuildNavigationHint());

            _inspector = new ScrollView();
            _inspector.AddToClassList("terrakit-inspector");

            main.Add(graphContainer);
            main.Add(_inspector);
            _bodySplitView.Add(main);
            rootVisualElement.Add(_bodySplitView);

            LoadGraphIntoView();
        }

        private static VisualElement BuildNavigationHint()
        {
            var panel = new VisualElement();
            panel.AddToClassList("terrakit-navigation-hint");
            panel.pickingMode = PickingMode.Ignore;

            var heading = new Label("CANVAS CONTROLS");
            heading.AddToClassList("terrakit-navigation-heading");
            panel.Add(heading);

            string alternatePan = Application.platform == RuntimePlatform.OSXEditor
                ? "Middle-drag or Option + left-drag"
                : "Middle-drag or Alt + left-drag";

            AddNavigationHintRow(panel, "Move canvas", alternatePan);
            AddNavigationHintRow(panel, "Zoom", "Mouse wheel");
            AddNavigationHintRow(panel, "Focus node", "Select a node, then press F");
            AddNavigationHintRow(panel, "Actual size", "Press 1 for 100%");
            AddNavigationHintRow(panel, "Overview", "Use Frame All");

            return panel;
        }

        private static void AddNavigationHintRow(VisualElement panel, string action, string control)
        {
            var row = new VisualElement();
            row.AddToClassList("terrakit-navigation-row");

            var actionLabel = new Label(action);
            actionLabel.AddToClassList("terrakit-navigation-action");

            var controlLabel = new Label(control);
            controlLabel.AddToClassList("terrakit-navigation-control");

            row.Add(actionLabel);
            row.Add(controlLabel);
            panel.Add(row);
        }

        private void BuildConsole()
        {
            _statusPanel = new VisualElement();
            _statusPanel.AddToClassList("terrakit-status-panel");

            var header = new VisualElement();
            header.AddToClassList("terrakit-status-header");

            _statusIcon = new Label();
            _statusIcon.AddToClassList("terrakit-status-icon");

            _statusType = new Label();
            _statusType.AddToClassList("terrakit-status-type");

            _statusTime = new Label();
            _statusTime.AddToClassList("terrakit-status-time");

            var clearButton = new Button(ClearStatus) { text = "Clear" };
            clearButton.AddToClassList("terrakit-status-clear");

            header.Add(_statusIcon);
            header.Add(_statusType);
            header.Add(_statusTime);
            header.Add(clearButton);

            var scrollView = new ScrollView(ScrollViewMode.Vertical);
            scrollView.AddToClassList("terrakit-status-scroll");

            _statusSummary = new Label();
            _statusSummary.AddToClassList("terrakit-status-summary");

            _statusDetails = new VisualElement();
            _statusDetails.AddToClassList("terrakit-status-details");

            scrollView.Add(_statusSummary);
            scrollView.Add(_statusDetails);

            _statusPanel.Add(header);
            _statusPanel.Add(scrollView);

            if (_bodySplitView != null)
            {
                _bodySplitView.Add(_statusPanel);
            }
            else
            {
                rootVisualElement.Add(_statusPanel);
            }

            ClearStatus();
        }

        private void LoadStylesheet()
        {
            var stylePath = "Assets/TerraKit/Editor/Styling/TerraKitGraph.uss";
            var stylesheet = AssetDatabase.LoadAssetAtPath<StyleSheet>(stylePath);
            if (stylesheet != null)
            {
                rootVisualElement.styleSheets.Add(stylesheet);
            }
        }

        private void CreateGraphAsset()
        {
            string path = EditorUtility.SaveFilePanelInProject(
                "Create TerraKit Graph",
                "TerraKitGenerationGraph",
                "asset",
                "Choose where to save the TerraKit graph asset.");

            if (string.IsNullOrEmpty(path))
            {
                return;
            }

            var asset = CreateInstance<TerraKitGraphAsset>();
            AssetDatabase.CreateAsset(asset, path);
            AssetDatabase.SaveAssets();
            AssetDatabase.Refresh();

            _graphAsset = asset;
            _graphField.value = asset;
            LoadGraphIntoView();
            SetStatus(
                StatusType.Success,
                "Graph created successfully.",
                "Asset: " + path);
        }

        private void LoadGraphIntoView()
        {
            if (_graphView != null)
            {
                _graphView.Load(_graphAsset);
            }

            _selectedNode = null;
            ShowEmptyInspector();
            UpdateGraphControls();
            RefreshSaveStateFromAsset();
            RefreshGraphFieldBranding();
        }

        private void RefreshGraphFieldBranding()
        {
            if (_graphField == null)
            {
                return;
            }

            _graphField.schedule.Execute(() =>
            {
                VisualElement objectDisplay =
                    _graphField.Q<VisualElement>(
                        className: ObjectField.objectUssClassName);
                Label objectLabel =
                    objectDisplay == null
                        ? null
                        : objectDisplay.Q<Label>();
                if (objectLabel == null)
                {
                    return;
                }

                objectLabel.text = _graphAsset == null
                    ? "None (TerraKit Graph Asset)"
                    : _graphAsset.name + " (TerraKit Graph Asset)";
            });
        }

        private void HandleNodeSelected(TerraKitNodeView node)
        {
            _selectedNode = node;
            ShowNodeInspector(node);
            UpdateGraphControls();
        }

        private void HandleGraphChanged()
        {
            SetSavedAt(DateTime.Now);
            SetStatus(
                StatusType.Info,
                "Graph changed and saved automatically.",
                "Validate or compile to check the updated pipeline state.");
        }

        private void RefreshSaveStateFromAsset()
        {
            if (_saveStateLabel == null)
            {
                return;
            }

            if (_graphAsset == null)
            {
                _saveStateLabel.style.display = DisplayStyle.None;
                return;
            }

            string assetPath = AssetDatabase.GetAssetPath(_graphAsset);
            if (!string.IsNullOrEmpty(assetPath))
            {
                string projectRoot = Path.GetFullPath(Path.Combine(Application.dataPath, ".."));
                string fullPath = Path.Combine(projectRoot, assetPath);
                if (File.Exists(fullPath))
                {
                    SetSavedAt(File.GetLastWriteTime(fullPath));
                    return;
                }
            }

            SetSavedAt(DateTime.Now);
        }

        private void SetSavedAt(DateTime savedAt)
        {
            if (_saveStateLabel == null || _graphAsset == null)
            {
                return;
            }

            _saveStateLabel.text = "Saved " + savedAt.ToString("HH:mm:ss");
            _saveStateLabel.tooltip =
                "Changes are saved automatically.\nLast saved: " +
                savedAt.ToString("yyyy-MM-dd HH:mm:ss");
            _saveStateLabel.style.display = DisplayStyle.Flex;
        }

        private void SaveGraph()
        {
            if (_graphAsset == null)
            {
                SetStatus(
                    StatusType.Error,
                    "Graph could not be saved.",
                    "No graph is selected. Create or assign a TerraKit graph first.");
                return;
            }

            if (_graphView != null)
            {
                _graphView.SyncNodeLayoutsToAsset();
            }

            EditorUtility.SetDirty(_graphAsset);
            AssetDatabase.SaveAssets();
            SetSavedAt(DateTime.Now);
            SetStatus(
                StatusType.Success,
                "Graph saved successfully.",
                "Asset: " + AssetDatabase.GetAssetPath(_graphAsset));
        }

        private void ValidateGraph()
        {
            if (_graphAsset == null)
            {
                SetStatus(StatusType.Error, "Validation could not start.", "No graph is selected.");
                return;
            }

            var result = new TerraKitGraphCompiler().Compile(_graphAsset);
            DisplayCompileResult("Validation", result);
        }

        private void CompileGraph()
        {
            if (_graphAsset == null)
            {
                SetStatus(StatusType.Error, "Compilation could not start.", "No graph is selected.");
                return;
            }

            var result = new TerraKitGraphCompiler().Compile(_graphAsset);
            DisplayCompileResult("Compile", result);
        }

        private void DisplayCompileResult(string operation, TerraKitGraphCompileResult result)
        {
            _graphView.SetValidationErrors(result.ErrorIssues);

            if (!result.Success)
            {
                SetStatus(
                    StatusType.Error,
                    operation + " failed with " + result.Errors.Count + " error(s).");
            }
            else if (result.Warnings.Count > 0)
            {
                SetStatus(
                    StatusType.Warning,
                    operation + " passed with " + result.Warnings.Count + " warning(s).");
            }
            else
            {
                SetStatus(
                    StatusType.Success,
                    operation + " passed. " + result.ExecutionOrder.Count + " node(s) are ready.");
            }

            bool showCompileSummary = operation == "Compile" && result.Success;
            PopulateCompileDetails(result, showCompileSummary);
        }

        private void PopulateCompileDetails(
            TerraKitGraphCompileResult result,
            bool showCompileSummary)
        {
            _statusDetails.Clear();

            foreach (var issue in result.ErrorIssues)
            {
                var row = new VisualElement();
                row.AddToClassList("terrakit-status-error-row");

                var message = new Label("Error: " + issue.Message);
                message.AddToClassList("terrakit-status-detail-message");
                row.Add(message);

                if (!string.IsNullOrEmpty(issue.NodeId))
                {
                    string nodeId = issue.NodeId;
                    var goToButton = new Button(() => GoToNode(nodeId)) { text = "Go to" };
                    goToButton.AddToClassList("terrakit-status-go-to");
                    row.Add(goToButton);
                }

                _statusDetails.Add(row);
            }

            foreach (var warning in result.Warnings)
            {
                AddStatusDetailLabel("Warning: " + warning);
            }

            if (result.Success)
            {
                if (showCompileSummary)
                {
                    AddCompileSummary(result);
                }

                var executionHeading = new Label("Execution order");
                executionHeading.AddToClassList("terrakit-compile-section-heading");
                _statusDetails.Add(executionHeading);

                for (int i = 0; i < result.ExecutionOrder.Count; i++)
                {
                    AddStatusDetailLabel((i + 1) + ". " + result.ExecutionOrder[i].displayName);
                }
            }

            _statusDetails.style.display = _statusDetails.childCount == 0
                ? DisplayStyle.None
                : DisplayStyle.Flex;
        }

        private void AddCompileSummary(TerraKitGraphCompileResult result)
        {
            int inputCount = 0;
            int outputCount = 0;

            foreach (var node in _graphAsset.nodes)
            {
                ITerraKitNodeDefinition definition;
                if (!TerraKitNodeRegistry.TryGet(node.typeId, out definition))
                {
                    continue;
                }

                if (definition.Category == "Input")
                {
                    inputCount++;
                }
                else if (definition.Category == "Output")
                {
                    outputCount++;
                }
            }

            int processorCount = _graphAsset.nodes.Count - inputCount - outputCount;

            var summary = new VisualElement();
            summary.AddToClassList("terrakit-compile-summary");

            var heading = new Label("Compile summary");
            heading.AddToClassList("terrakit-compile-section-heading");
            summary.Add(heading);

            var metrics = new VisualElement();
            metrics.AddToClassList("terrakit-compile-metrics");
            AddCompileMetric(metrics, "Nodes", _graphAsset.nodes.Count);
            AddCompileMetric(metrics, "Connections", _graphAsset.edges.Count);
            AddCompileMetric(metrics, "Inputs", inputCount);
            AddCompileMetric(metrics, "Processors", processorCount);
            AddCompileMetric(metrics, "Outputs", outputCount);
            AddCompileMetric(metrics, "Execution steps", result.ExecutionOrder.Count);
            summary.Add(metrics);

            _statusDetails.Add(summary);
        }

        private static void AddCompileMetric(VisualElement parent, string name, int value)
        {
            var metric = new VisualElement();
            metric.AddToClassList("terrakit-compile-metric");

            var valueLabel = new Label(value.ToString());
            valueLabel.AddToClassList("terrakit-compile-metric-value");

            var nameLabel = new Label(name);
            nameLabel.AddToClassList("terrakit-compile-metric-name");

            metric.Add(valueLabel);
            metric.Add(nameLabel);
            parent.Add(metric);
        }

        private void GoToNode(string nodeId)
        {
            if (_graphView != null)
            {
                _graphView.GoToNode(nodeId);
            }
        }

        private void AddStatusDetailLabel(string text)
        {
            var label = new Label(text);
            label.AddToClassList("terrakit-status-detail-message");
            _statusDetails.Add(label);
        }

        private void CreateBackendDemoGraph()
        {
            const string demoFolder = "Assets/TerraKitGraphs";
            if (!AssetDatabase.IsValidFolder(demoFolder))
            {
                AssetDatabase.CreateFolder("Assets", "TerraKitGraphs");
            }

            string assetPath = AssetDatabase.GenerateUniqueAssetPath(
                demoFolder + "/TerraKitBackendDemo.asset");

            var demoAsset = CreateInstance<TerraKitGraphAsset>();
            AssetDatabase.CreateAsset(demoAsset, assetPath);

            _graphAsset = demoAsset;
            _graphField.SetValueWithoutNotify(_graphAsset);

            var flat = CreateNode("terrakit.height.flat", new Vector2(50, 160));
            var noise = CreateNode("terrakit.height.noise", new Vector2(390, 130));
            var mesh = CreateNode("terrakit.mesh.height_field", new Vector2(730, 160));

            SetDemoParameter(noise, "frequency", "0.08");
            SetDemoParameter(noise, "amplitude", "10");

            AddEdge(flat, "height", noise, "source");
            AddEdge(noise, "height", mesh, "height");

            TerraKitGraphEditorUtil.MarkDirty(_graphAsset);
            AssetDatabase.SaveAssets();

            LoadGraphIntoView();
            CompileGraph();

            Selection.activeObject = _graphAsset;
            EditorGUIUtility.PingObject(_graphAsset);
        }

        private static void SetDemoParameter(
            TerraKitNodeData node,
            string parameterKey,
            string value)
        {
            if (node == null)
            {
                return;
            }

            var parameter = node.parameters.FirstOrDefault(
                item => item.key == parameterKey);
            if (parameter != null)
            {
                parameter.value = value;
            }
        }

        private TerraKitNodeData CreateNode(string typeId, Vector2 position)
        {
            ITerraKitNodeDefinition definition;
            if (!TerraKitNodeRegistry.TryGet(typeId, out definition))
            {
                Debug.LogError("Unknown TerraKit node type: " + typeId);
                return null;
            }

            var node = TerraKitGraphEditorUtil.CreateNodeData(definition, position);
            _graphAsset.nodes.Add(node);
            return node;
        }

        private void AddEdge(TerraKitNodeData outputNode, string outputPort, TerraKitNodeData inputNode, string inputPort)
        {
            if (outputNode == null || inputNode == null)
            {
                return;
            }

            _graphAsset.edges.Add(new TerraKitEdgeData
            {
                id = System.Guid.NewGuid().ToString("N"),
                outputNodeId = outputNode.id,
                outputPortId = outputPort,
                inputNodeId = inputNode.id,
                inputPortId = inputPort
            });
        }

        private void ShowEmptyInspector()
        {
            if (_inspector == null)
            {
                return;
            }

            _inspector.Clear();
            _inspector.Add(new Label("Inspector") { name = "InspectorTitle" });

            var guidance = new Label(
                "Select a node to edit its parameters. Right-click the canvas to add nodes.");
            guidance.AddToClassList("terrakit-inspector-text");
            _inspector.Add(guidance);
        }

        private void ShowNodeInspector(TerraKitNodeView node)
        {
            if (_inspector == null)
            {
                return;
            }

            _inspector.Clear();

            if (node == null)
            {
                ShowEmptyInspector();
                return;
            }

            _inspector.Add(new Label(node.Data.displayName) { name = "InspectorTitle" });

            var typeLabel = new Label(node.Definition.Category + " / " + node.Definition.TypeId);
            typeLabel.AddToClassList("terrakit-inspector-text");
            _inspector.Add(typeLabel);

            var idLabel = new Label("Node ID: " + node.Data.id);
            idLabel.AddToClassList("terrakit-inspector-text");
            _inspector.Add(idLabel);

            if (node.Definition.Parameters.Count == 0)
            {
                var emptyMessage = new Label("This node has no editable parameters.");
                emptyMessage.AddToClassList("terrakit-inspector-text");
                _inspector.Add(emptyMessage);
                return;
            }

            _inspector.Add(new Label("Parameters") { name = "InspectorSection" });

            foreach (var parameterDefinition in node.Definition.Parameters)
            {
                var parameter = node.Data.parameters.FirstOrDefault(p => p.key == parameterDefinition.Key);
                if (parameter == null)
                {
                    continue;
                }

                AddParameterField(node, parameterDefinition, parameter);
            }
        }

        private void AddParameterField(
            TerraKitNodeView node,
            TerraKitParameterDefinition definition,
            TerraKitParameterData parameter)
        {
            var fieldContainer = new VisualElement();
            fieldContainer.AddToClassList("terrakit-parameter-field");

            VisualElement inputField;
            Action<string> setInputValueWithoutNotify;
            Button resetButton = null;

            switch (parameter.type)
            {
                case TerraKitParameterType.Integer:
                    var integerField = new TextField(parameter.displayName) { value = parameter.value };
                    integerField.RegisterValueChangedCallback(evt =>
                    {
                        UpdateParameter(node, parameter, evt.newValue);
                        RefreshParameterValidation(definition, parameter, fieldContainer);
                        resetButton?.SetEnabled(parameter.value != definition.DefaultValue);
                    });
                    inputField = integerField;
                    setInputValueWithoutNotify = value => integerField.SetValueWithoutNotify(value);
                    break;

                case TerraKitParameterType.Float:
                    var floatField = new TextField(parameter.displayName) { value = parameter.value };
                    floatField.RegisterValueChangedCallback(evt =>
                    {
                        UpdateParameter(node, parameter, evt.newValue);
                        RefreshParameterValidation(definition, parameter, fieldContainer);
                        resetButton?.SetEnabled(parameter.value != definition.DefaultValue);
                    });
                    inputField = floatField;
                    setInputValueWithoutNotify = value => floatField.SetValueWithoutNotify(value);
                    break;

                case TerraKitParameterType.Boolean:
                    bool boolValue;
                    bool.TryParse(parameter.value, out boolValue);
                    var boolField = new Toggle(parameter.displayName) { value = boolValue };
                    boolField.RegisterValueChangedCallback(evt =>
                    {
                        UpdateParameter(node, parameter, evt.newValue.ToString());
                        RefreshParameterValidation(definition, parameter, fieldContainer);
                        resetButton?.SetEnabled(parameter.value != definition.DefaultValue);
                    });
                    inputField = boolField;
                    setInputValueWithoutNotify = value =>
                    {
                        bool parsedValue;
                        bool.TryParse(value, out parsedValue);
                        boolField.SetValueWithoutNotify(parsedValue);
                    };
                    break;

                default:
                    var textField = new TextField(parameter.displayName) { value = parameter.value };
                    textField.RegisterValueChangedCallback(evt =>
                    {
                        UpdateParameter(node, parameter, evt.newValue);
                        RefreshParameterValidation(definition, parameter, fieldContainer);
                        resetButton?.SetEnabled(parameter.value != definition.DefaultValue);
                    });
                    inputField = textField;
                    setInputValueWithoutNotify = value => textField.SetValueWithoutNotify(value);
                    break;
            }

            inputField.AddToClassList("terrakit-parameter-input");
            inputField.tooltip = definition.Description +
                                 "\nValid value: " + definition.ValidValueHint +
                                 "\nDefault: " + definition.DefaultValue;
            fieldContainer.Add(inputField);

            if (!string.IsNullOrEmpty(definition.Description))
            {
                var descriptionLabel = new Label(definition.Description);
                descriptionLabel.AddToClassList("terrakit-parameter-description");
                fieldContainer.Add(descriptionLabel);
            }

            var metadataRow = new VisualElement();
            metadataRow.AddToClassList("terrakit-parameter-metadata-row");

            var valueHint = new Label(
                "Valid: " + definition.ValidValueHint +
                "  •  Default: " + definition.DefaultValue);
            valueHint.AddToClassList("terrakit-parameter-value-hint");
            metadataRow.Add(valueHint);

            resetButton = new Button(() =>
            {
                setInputValueWithoutNotify(definition.DefaultValue);
                UpdateParameter(node, parameter, definition.DefaultValue);
                RefreshParameterValidation(definition, parameter, fieldContainer);
                resetButton.SetEnabled(false);
            })
            {
                text = "Reset",
                tooltip = "Restore the default value: " + definition.DefaultValue
            };
            resetButton.AddToClassList("terrakit-parameter-reset");
            resetButton.SetEnabled(parameter.value != definition.DefaultValue);
            metadataRow.Add(resetButton);
            fieldContainer.Add(metadataRow);

            var errorLabel = new Label();
            errorLabel.AddToClassList("terrakit-parameter-error");
            fieldContainer.Add(errorLabel);

            _inspector.Add(fieldContainer);
            RefreshParameterValidation(definition, parameter, fieldContainer);
        }

        private static void RefreshParameterValidation(
            TerraKitParameterDefinition definition,
            TerraKitParameterData parameter,
            VisualElement fieldContainer)
        {
            var errorLabel = fieldContainer.Q<Label>(className: "terrakit-parameter-error");
            var issues = TerraKitParameterValidator.Validate(definition, parameter);

            if (issues.Count == 0)
            {
                fieldContainer.RemoveFromClassList("terrakit-parameter-invalid");
                errorLabel.text = string.Empty;
                errorLabel.style.display = DisplayStyle.None;
                return;
            }

            fieldContainer.AddToClassList("terrakit-parameter-invalid");
            errorLabel.text = string.Join("\n", issues.Select(issue => issue.Message));
            errorLabel.style.display = DisplayStyle.Flex;
        }

        private void UpdateParameter(TerraKitNodeView node, TerraKitParameterData parameter, string value)
        {
            if (_graphAsset == null)
            {
                return;
            }

            Undo.RecordObject(_graphAsset, "Edit TerraKit Parameter");
            parameter.value = value;
            node.RefreshParameterSummary();
            TerraKitGraphEditorUtil.MarkDirty(_graphAsset);
            SetSavedAt(DateTime.Now);
        }

        private void UpdateGraphControls()
        {
            bool hasGraph = _graphAsset != null;

            _saveButton.SetEnabled(hasGraph);
            _validateButton.SetEnabled(hasGraph);
            _compileButton.SetEnabled(hasGraph);
            _frameAllButton.SetEnabled(hasGraph);
            _focusButton.SetEnabled(hasGraph && _selectedNode != null);
            _resetZoomButton.SetEnabled(hasGraph);
        }

        private void FrameAllNodes()
        {
            if (_graphView == null)
            {
                return;
            }

            _graphView.FrameAll();
            SetStatus(
                StatusType.Info,
                "All nodes were fitted to the current view.",
                "You can continue editing, validating, or compiling the graph.");
        }

        private void FocusSelectedNode()
        {
            if (_graphView == null || !_graphView.FocusSelection())
            {
                return;
            }

            SetStatus(
                StatusType.Info,
                "The selected node was fitted to the current view.",
                "Use the mouse wheel to zoom, or press 1 to return to 100%.");
        }

        private void ResetZoom()
        {
            if (_graphView == null)
            {
                return;
            }

            _graphView.ResetZoomTo100Percent();
            SetStatus(
                StatusType.Info,
                "Canvas zoom was reset to 100%.",
                "The current canvas centre was preserved.");
        }

        private void ShowHelp()
        {
            SetStatus(
                StatusType.Info,
                "TerraKit Graph help",
                "How to use TerraKit Graph:\n" +
                "1. Create or select a graph asset.\n" +
                "2. Right-click the canvas to add nodes.\n" +
                "3. Connect compatible input and output ports.\n" +
                "4. Select a node to edit its parameters.\n" +
                "5. Save, Validate, and Compile the graph.\n" +
                "6. Use Frame All to centre all nodes."
            );
        }

        private void ClearStatus()
        {
            SetStatus(
                StatusType.Info,
                "Ready.",
                _graphAsset == null
                    ? "Select or create a TerraKit graph to begin."
                    : "Select a node, edit its parameters, or choose an action from the toolbar.");
        }

        private void SetStatus(StatusType type, string summary, string details = "")
        {
            if (_statusPanel == null)
            {
                return;
            }

            _statusPanel.RemoveFromClassList("terrakit-status-info");
            _statusPanel.RemoveFromClassList("terrakit-status-success");
            _statusPanel.RemoveFromClassList("terrakit-status-warning");
            _statusPanel.RemoveFromClassList("terrakit-status-error");

            string icon;
            string typeName;
            string styleClass;

            switch (type)
            {
                case StatusType.Success:
                    icon = "✓";
                    typeName = "SUCCESS";
                    styleClass = "terrakit-status-success";
                    break;
                case StatusType.Warning:
                    icon = "⚠";
                    typeName = "WARNING";
                    styleClass = "terrakit-status-warning";
                    break;
                case StatusType.Error:
                    icon = "✕";
                    typeName = "ERROR";
                    styleClass = "terrakit-status-error";
                    break;
                default:
                    icon = "ⓘ";
                    typeName = "INFO";
                    styleClass = "terrakit-status-info";
                    break;
            }

            _statusPanel.AddToClassList(styleClass);
            _statusIcon.text = icon;
            _statusType.text = typeName;
            _statusTime.text = System.DateTime.Now.ToString("HH:mm:ss");
            _statusSummary.text = summary;
            _statusDetails.Clear();
            if (!string.IsNullOrEmpty(details))
            {
                AddStatusDetailLabel(details);
            }
            _statusDetails.style.display = string.IsNullOrEmpty(details)
                ? DisplayStyle.None
                : DisplayStyle.Flex;
        }
    }
}
