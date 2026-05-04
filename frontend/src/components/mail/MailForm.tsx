import { JsonSchemaForm } from "@/components/JsonSchemaForm";
import { type MailInput, MailInputSchema } from "@/lib/api_client";
import { zodResolver } from "@hookform/resolvers/zod";
import { useForm } from "react-hook-form";

interface MailFormProps {
	bodySchema: Record<string, unknown>;
	onSubmit: (values: MailInput) => void;
}

export function MailForm(props: MailFormProps) {
	const form = useForm<MailInput>({
		resolver: zodResolver(MailInputSchema),
		defaultValues: {
			body: {},
		} satisfies MailInput,
	});

	const onSubmit = (values: MailInput) => {
		form.reset();
		props.onSubmit(values);
	};

	return (
		<>
			<JsonSchemaForm
				schema={props.bodySchema}
				onSubmit={(values) => onSubmit({ body: values })}
			/>
		</>
	);
}
