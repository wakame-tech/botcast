import { ScriptForm } from "@/components/script/ScriptForm";
import type { ScriptInput } from "@/lib/api_client";
import { $cms, recordToScript, toRecordData } from "@/lib/cms_client";
import { createLazyFileRoute } from "@tanstack/react-router";

export const Route = createLazyFileRoute("/scripts/$scriptId_/edit")({
	component: EditScript,
});

export function EditScript() {
	const { scriptId } = Route.useParams();
	const navigate = Route.useNavigate();
	const { data: scriptRecord } = $cms.useQuery(
		"get",
		"/records/{collectionId}/{recordId}",
		{ params: { path: { collectionId: "scripts", recordId: scriptId } } },
	);
	const updateScript = $cms.useMutation(
		"put",
		"/records/{collectionId}/{recordId}",
	);

	const handleSubmit = async (values: ScriptInput) => {
		await updateScript.mutateAsync({
			params: { path: { collectionId: "scripts", recordId: scriptId } },
			body: {
				data: toRecordData({
					title: values.title,
					description: values.description,
					template: values.template,
				}),
			},
		});
		navigate({ to: "/scripts/$scriptId", params: { scriptId } });
	};

	if (!scriptRecord) {
		return null;
	}

	const script = recordToScript(scriptRecord);

	return (
		<>
			<ScriptForm
				values={{
					title: script.title,
					description: script.description,
					template: script.template,
				}}
				onSubmit={handleSubmit}
			/>
		</>
	);
}
