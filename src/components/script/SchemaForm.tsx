import Form from '@rjsf/core';
import type { RJSFSchema } from '@rjsf/utils';
import validator from '@rjsf/validator-ajv8';

interface SchemaFormProps {
    schema: RJSFSchema,
    onSubmit: (values: unknown) => void,
}

export function SchemaForm(props: SchemaFormProps) {
    return (
        <Form
            schema={props.schema}
            validator={validator}
            onSubmit={props.onSubmit}
        />
    )
}
