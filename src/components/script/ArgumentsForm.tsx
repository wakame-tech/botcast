import Form from "@rjsf/core";
import type { RJSFSchema } from "@rjsf/utils";
import validator from "@rjsf/validator-ajv8";

interface ArgumentsFormProps {
	schema: RJSFSchema;
	onChange?: (values: Record<string, unknown>) => void;
}

export function ArgumentsForm(props: ArgumentsFormProps) {
	return (
		<Form
			schema={props.schema}
			validator={validator}
			onChange={(e) => props.onChange?.(e.formData as Record<string, unknown>)}
		/>
	);
}
