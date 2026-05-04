import Form from "@rjsf/core";
import type { RJSFSchema } from "@rjsf/utils";
import validator from "@rjsf/validator-ajv8";

interface JsonSchemaFormProps {
	schema: RJSFSchema;
	onChange?: (values: Record<string, unknown>) => void;
	onSubmit?: (values: Record<string, unknown>) => void;
}

const buildUiSchema = (
	schema: Record<string, unknown>,
): Record<string, { "ui:widget": string }> => {
	const properties = schema.properties as Record<
		string,
		Record<string, unknown>
	>;
	if (typeof properties !== "object") {
		return {};
	}
	const uiSchemas: Record<string, { "ui:widget": string }> = {};
	for (const [k, v] of Object.entries(properties)) {
		if (v.uiType === "textarea") {
			uiSchemas[k] = { "ui:widget": "textarea" };
		}
	}
	return uiSchemas;
};

export function JsonSchemaForm(props: JsonSchemaFormProps) {
	const uiSchema = buildUiSchema(props.schema);
	return (
		<Form
			schema={props.schema}
			uiSchema={uiSchema}
			validator={validator}
			onChange={(e) => props.onChange?.(e.formData as Record<string, unknown>)}
			onSubmit={(e) => props.onSubmit?.(e.formData as Record<string, unknown>)}
		/>
	);
}
