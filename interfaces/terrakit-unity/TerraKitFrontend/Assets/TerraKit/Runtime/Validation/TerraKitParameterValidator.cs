using System.Collections.Generic;
using System.Globalization;

namespace TerraKit
{
    public sealed class TerraKitParameterValidationIssue
    {
        public string ParameterKey { get; private set; }
        public string Message { get; private set; }

        public TerraKitParameterValidationIssue(string parameterKey, string message)
        {
            ParameterKey = parameterKey;
            Message = message;
        }
    }

    public static class TerraKitParameterValidator
    {
        public static IReadOnlyList<TerraKitParameterValidationIssue> Validate(
            TerraKitParameterDefinition definition,
            TerraKitParameterData parameter)
        {
            var issues = new List<TerraKitParameterValidationIssue>();

            if (definition == null)
            {
                issues.Add(new TerraKitParameterValidationIssue(
                    parameter != null ? parameter.key : string.Empty,
                    "Parameter definition is missing."));
                return issues;
            }

            if (parameter == null)
            {
                issues.Add(new TerraKitParameterValidationIssue(
                    definition.Key,
                    "Parameter value is missing."));
                return issues;
            }

            double numericValue;
            if (!TryParseNumericValue(definition.Type, parameter.value, out numericValue))
            {
                if (definition.Type == TerraKitParameterType.Integer ||
                    definition.Type == TerraKitParameterType.Float)
                {
                    issues.Add(new TerraKitParameterValidationIssue(
                        definition.Key,
                        "Value is not a valid " + GetNumericTypeName(definition.Type) +
                        ". Current value: " + FormatCurrentValue(parameter.value) + "."));
                }

                return issues;
            }

            if (definition.MinimumValue.HasValue)
            {
                double minimum = definition.MinimumValue.Value;
                bool belowMinimum = definition.MinimumInclusive
                    ? numericValue < minimum
                    : numericValue <= minimum;

                if (belowMinimum)
                {
                    string minimumText = FormatNumber(minimum);
                    string currentValueText = FormatNumber(numericValue);
                    string message;

                    if (definition.MinimumInclusive && minimum == 0)
                    {
                        message = "Value cannot be negative. Current value: " + currentValueText + ".";
                    }
                    else if (definition.MinimumInclusive)
                    {
                        message = "Value must be at least " + minimumText +
                                  ". Current value: " + currentValueText + ".";
                    }
                    else
                    {
                        message = "Value must be greater than " + minimumText +
                                  ". Current value: " + currentValueText + ".";
                    }

                    issues.Add(new TerraKitParameterValidationIssue(definition.Key, message));
                    return issues;
                }
            }

            if (definition.MaximumValue.HasValue)
            {
                double maximum = definition.MaximumValue.Value;
                bool aboveMaximum = definition.MaximumInclusive
                    ? numericValue > maximum
                    : numericValue >= maximum;

                if (aboveMaximum)
                {
                    string message = definition.MaximumInclusive
                        ? "Value must be at most " + FormatNumber(maximum) +
                          ". Current value: " + FormatNumber(numericValue) + "."
                        : "Value must be less than " + FormatNumber(maximum) +
                          ". Current value: " + FormatNumber(numericValue) + ".";
                    issues.Add(new TerraKitParameterValidationIssue(definition.Key, message));
                }
            }

            return issues;
        }

        private static bool TryParseNumericValue(
            TerraKitParameterType type,
            string value,
            out double numericValue)
        {
            numericValue = 0;

            if (type == TerraKitParameterType.Integer)
            {
                int integerValue;
                if (!int.TryParse(
                        value,
                        NumberStyles.Integer,
                        CultureInfo.InvariantCulture,
                        out integerValue))
                {
                    return false;
                }

                numericValue = integerValue;
                return true;
            }

            if (type == TerraKitParameterType.Float)
            {
                float floatValue;
                if (!float.TryParse(
                        value,
                        NumberStyles.Float,
                        CultureInfo.InvariantCulture,
                        out floatValue) ||
                    float.IsNaN(floatValue) ||
                    float.IsInfinity(floatValue))
                {
                    return false;
                }

                numericValue = floatValue;
                return true;
            }

            return false;
        }

        private static string GetNumericTypeName(TerraKitParameterType type)
        {
            return type == TerraKitParameterType.Integer ? "integer" : "number";
        }

        private static string FormatNumber(double value)
        {
            return value.ToString("0.################", CultureInfo.InvariantCulture);
        }

        private static string FormatCurrentValue(string value)
        {
            return string.IsNullOrEmpty(value) ? "<empty>" : value;
        }
    }
}
