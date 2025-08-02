import { ScriptForm } from "@/components/script/ScriptForm";
import { $api, type ScriptInput } from "@/lib/api_client";
import { createLazyFileRoute } from "@tanstack/react-router";

export const Route = createLazyFileRoute("/scripts/$scriptId/edit")({
	component: EditScript,
});

export function EditScript() {
	const { scriptId } = Route.useParams();
	const navigate = Route.useNavigate();
	const getScript = $api.useQuery("get", "/scripts/{scriptId}", {
		params: { path: { scriptId } },
	});
	const updateScript = $api.useMutation("put", "/scripts/{scriptId}");

	const handleSubmit = async (values: ScriptInput) => {
		await updateScript.mutateAsync({
			params: {
				path: { scriptId },
			},
			body: {
				title: values.title,
				description: values.description,
				template: JSON.parse(values.template),
			},
		});
		navigate({ to: "/scripts/$scriptId", params: { scriptId } });
	};

	if (!getScript.data) {
		return null;
	}

	const script = getScript.data;

	return (
		<>
			<ScriptForm
				values={{
					title: script.title,
					description: script.description,
					template: JSON.stringify(script.template, null, 4),
				}}
				onSubmit={handleSubmit}
			/>
		</>
	);
}
