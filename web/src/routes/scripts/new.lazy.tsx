import {
	ScriptForm,
	type ScriptFormValues,
	toScriptData,
} from "@/components/script/ScriptForm";
import { $cms, CMS_TENANT, toRecordData } from "@/lib/cms_client";
import { createLazyFileRoute, useNavigate } from "@tanstack/react-router";

export const Route = createLazyFileRoute("/scripts/new")({
	component: NewScript,
});

export function NewScript() {
	const navigate = useNavigate();
	const newScript = $cms.useMutation(
		"post",
		"/t/{tenantId}/records/{collectionId}",
	);

	const handleSubmit = async (values: ScriptFormValues) => {
		await newScript.mutateAsync({
			params: { path: { tenantId: CMS_TENANT, collectionId: "scripts" } },
			body: {
				data: toRecordData({
					...toScriptData(values),
				}),
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
