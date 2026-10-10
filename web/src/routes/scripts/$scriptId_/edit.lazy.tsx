import {
	ScriptForm,
	type ScriptFormValues,
	toScriptData,
} from "@/components/script/ScriptForm";
import {
	$cms,
	CMS_TENANT,
	recordToScript,
	toRecordData,
} from "@/lib/cms_client";
import { createLazyFileRoute } from "@tanstack/react-router";

export const Route = createLazyFileRoute("/scripts/$scriptId/edit")({
	component: EditScript,
});

export function EditScript() {
	const { scriptId } = Route.useParams();
	const navigate = Route.useNavigate();
	const { data: scriptRecord } = $cms.useQuery(
		"get",
		"/t/{tenantId}/records/{collectionId}/{recordId}",
		{
			params: {
				path: {
					tenantId: CMS_TENANT,
					collectionId: "scripts",
					recordId: scriptId,
				},
			},
		},
	);
	const updateScript = $cms.useMutation(
		"put",
		"/t/{tenantId}/records/{collectionId}/{recordId}",
	);

	const handleSubmit = async (values: ScriptFormValues) => {
		// user_id など、フォームに無い既存の data を保ったまま更新する
		const current = (scriptRecord?.data ?? {}) as Record<string, unknown>;
		await updateScript.mutateAsync({
			params: {
				path: {
					tenantId: CMS_TENANT,
					collectionId: "scripts",
					recordId: scriptId,
				},
			},
			body: {
				data: toRecordData({ ...current, ...toScriptData(values) }),
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
					arguments: JSON.stringify(script.arguments, null, 2),
				}}
				onSubmit={handleSubmit}
			/>
		</>
	);
}
