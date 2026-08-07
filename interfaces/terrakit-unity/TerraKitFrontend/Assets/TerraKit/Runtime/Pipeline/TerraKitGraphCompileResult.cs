using System.Collections.Generic;

namespace TerraKit
{
    public sealed class TerraKitGraphCompileIssue
    {
        public string Message { get; private set; }
        public string NodeId { get; private set; }

        public TerraKitGraphCompileIssue(string message, string nodeId = null)
        {
            Message = message;
            NodeId = nodeId;
        }
    }

    public sealed class TerraKitGraphCompileResult
    {
        public bool Success
        {
            get { return Errors.Count == 0; }
        }

        public readonly List<string> Errors = new List<string>();
        public readonly List<TerraKitGraphCompileIssue> ErrorIssues =
            new List<TerraKitGraphCompileIssue>();
        public readonly List<string> Warnings = new List<string>();
        public readonly List<TerraKitNodeData> ExecutionOrder = new List<TerraKitNodeData>();

        public void AddError(string message, string nodeId = null)
        {
            Errors.Add(message);
            ErrorIssues.Add(new TerraKitGraphCompileIssue(message, nodeId));
        }
    }
}
