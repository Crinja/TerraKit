using System.Collections.Generic;
using System.Globalization;

namespace TerraKit
{
    /// <summary>
    /// Defines a node type. Runtime, compiler, and editor can all read this without depending on GraphView.
    /// </summary>
    public interface ITerraKitNodeDefinition
    {
        string TypeId { get; }
        string DisplayName { get; }
        string Category { get; }
        string Description { get; }
        IReadOnlyList<TerraKitPortDefinition> Inputs { get; }
        IReadOnlyList<TerraKitPortDefinition> Outputs { get; }
        IReadOnlyList<TerraKitParameterDefinition> Parameters { get; }
    }

    public sealed class TerraKitNodeDefinition : ITerraKitNodeDefinition
    {
        public string TypeId { get; private set; }
        public string DisplayName { get; private set; }
        public string Category { get; private set; }
        public string Description { get; private set; }
        public IReadOnlyList<TerraKitPortDefinition> Inputs { get; private set; }
        public IReadOnlyList<TerraKitPortDefinition> Outputs { get; private set; }
        public IReadOnlyList<TerraKitParameterDefinition> Parameters { get; private set; }

        public TerraKitNodeDefinition(
            string typeId,
            string displayName,
            string category,
            IReadOnlyList<TerraKitPortDefinition> inputs,
            IReadOnlyList<TerraKitPortDefinition> outputs,
            IReadOnlyList<TerraKitParameterDefinition> parameters,
            string description = "")
        {
            TypeId = typeId;
            DisplayName = displayName;
            Category = category;
            Description = description ?? string.Empty;
            Inputs = inputs;
            Outputs = outputs;
            Parameters = parameters;
        }
    }

    public sealed class TerraKitPortDefinition
    {
        public string Id { get; private set; }
        public string DisplayName { get; private set; }
        public TerraKitPortType Type { get; private set; }
        public bool IsRequired { get; private set; }
        public bool AllowMultipleConnections { get; private set; }

        public TerraKitPortDefinition(
            string id,
            string displayName,
            TerraKitPortType type,
            bool isRequired,
            bool allowMultipleConnections)
        {
            Id = id;
            DisplayName = displayName;
            Type = type;
            IsRequired = isRequired;
            AllowMultipleConnections = allowMultipleConnections;
        }
    }

    public sealed class TerraKitParameterDefinition
    {
        public string Key { get; private set; }
        public string DisplayName { get; private set; }
        public TerraKitParameterType Type { get; private set; }
        public string DefaultValue { get; private set; }
        public double? MinimumValue { get; private set; }
        public bool MinimumInclusive { get; private set; }
        public double? MaximumValue { get; private set; }
        public bool MaximumInclusive { get; private set; }
        public string Description { get; private set; }
        public string ValidValueHint { get; private set; }

        public TerraKitParameterDefinition(
            string key,
            string displayName,
            TerraKitParameterType type,
            string defaultValue,
            double? minimumValue = null,
            bool minimumInclusive = true,
            string description = "",
            double? maximumValue = null,
            bool maximumInclusive = true)
        {
            Key = key;
            DisplayName = displayName;
            Type = type;
            DefaultValue = defaultValue;
            MinimumValue = minimumValue;
            MinimumInclusive = minimumInclusive;
            MaximumValue = maximumValue;
            MaximumInclusive = maximumInclusive;
            Description = description ?? string.Empty;
            ValidValueHint = BuildValidValueHint(
                type,
                minimumValue,
                minimumInclusive,
                maximumValue,
                maximumInclusive);
        }

        private static string BuildValidValueHint(
            TerraKitParameterType type,
            double? minimumValue,
            bool minimumInclusive,
            double? maximumValue,
            bool maximumInclusive)
        {
            if (type == TerraKitParameterType.Boolean)
            {
                return "True or False";
            }

            if (type == TerraKitParameterType.String)
            {
                return "Text";
            }

            string valueType = type == TerraKitParameterType.Integer ? "Integer" : "Number";
            if (!minimumValue.HasValue && !maximumValue.HasValue)
            {
                return "Any " + valueType.ToLowerInvariant();
            }

            string hint = valueType;
            if (minimumValue.HasValue)
            {
                string minimumText = minimumValue.Value.ToString(
                    "0.################",
                    CultureInfo.InvariantCulture);
                hint += (minimumInclusive ? " at least " : " greater than ") + minimumText;
            }

            if (maximumValue.HasValue)
            {
                string maximumText = maximumValue.Value.ToString(
                    "0.################",
                    CultureInfo.InvariantCulture);
                hint += (maximumInclusive ? " and at most " : " and less than ") + maximumText;
            }

            return hint;
        }
    }
}
