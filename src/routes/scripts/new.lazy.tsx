import { ScriptForm } from "@/components/script/ScriptForm";
import { $api, type ScriptInput } from "@/lib/api_client";
import { createLazyFileRoute, useNavigate } from "@tanstack/react-router";

export const Route = createLazyFileRoute("/scripts/new")({
	component: NewScript,
});

export function NewScript() {
	const navigate = useNavigate();
	const newScript = $api.useMutation("post", "/scripts");

	const handleSubmit = async (values: ScriptInput) => {
		await newScript.mutateAsync({
			body: {
				title: values.title,
				description: values.description,
				template: JSON.parse(values.template),
				arguments: {},
			},
		});
		navigate({ to: "/scripts" });
	};

	return (
		<div>
			<h1>New Script</h1>
			<ScriptForm onSubmit={handleSubmit} />
		</div>
	);
}
