import {
	ScriptForm,
	type ScriptFormValues,
	toScriptData,
} from "@/components/script/ScriptForm";
import { useSession } from "@/hooks/useSession";
import { $cms, toRecordData } from "@/lib/cms_client";
import { createLazyFileRoute, useNavigate } from "@tanstack/react-router";

export const Route = createLazyFileRoute("/scripts/new")({
	component: NewScript,
});

export function NewScript() {
	const navigate = useNavigate();
	const { userId } = useSession();
	const newScript = $cms.useMutation("post", "/records/{collectionId}");

	const handleSubmit = async (values: ScriptFormValues) => {
		await newScript.mutateAsync({
			params: { path: { collectionId: "scripts" } },
			body: {
				data: toRecordData({
					...toScriptData(values),
					user_id: userId ?? "",
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
