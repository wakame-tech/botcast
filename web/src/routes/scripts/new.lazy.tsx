import { ScriptForm } from "@/components/script/ScriptForm";
import { $api } from "@/lib/api_client";
import type { ScriptInput } from "@/lib/api_client";
import { $cms, toRecordData } from "@/lib/cms_client";
import { createLazyFileRoute, useNavigate } from "@tanstack/react-router";

export const Route = createLazyFileRoute("/scripts/new")({
	component: NewScript,
});

export function NewScript() {
	const navigate = useNavigate();
	const { data: me } = $api.useQuery("get", "/me");
	const newScript = $cms.useMutation("post", "/records/{collectionId}");

	const handleSubmit = async (values: ScriptInput) => {
		await newScript.mutateAsync({
			params: { path: { collectionId: "scripts" } },
			body: {
				data: toRecordData({
					title: values.title,
					description: values.description,
					template: values.template,
					arguments: {},
					user_id: me?.id ?? "",
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
