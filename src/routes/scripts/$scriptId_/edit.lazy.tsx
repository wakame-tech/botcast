import { ScriptForm } from "@/components/script/ScriptForm";
import type { ScriptEditFormValues } from "@/components/script/ScriptForm";
import { trpc } from "@/trpc.ts";
import { createLazyFileRoute } from "@tanstack/react-router";

export const Route = createLazyFileRoute("/scripts/$scriptId/edit")({
	component: EditScript,
});

export function EditScript() {
	const { scriptId } = Route.useParams();
	const navigate = Route.useNavigate();
	const getScript = trpc.script.useQuery({ id: scriptId });
	const updateScript = trpc.updateScript.useMutation();

	const handleSubmit = async (values: ScriptEditFormValues) => {
		await updateScript.mutateAsync({
			id: scriptId,
			title: values.title,
			template: values.template,
		});
		navigate({ to: "/scripts/$scriptId", params: { scriptId } });
	};

	if (!getScript.data) {
		return null;
	}

	const script = getScript.data.script;
	const template = JSON.stringify(script.template, null, 4);

	return (
		<>
			<ScriptForm
				values={{
					title: script.title,
					template,
				}}
				onSubmit={handleSubmit}
			/>
		</>
	);
}
