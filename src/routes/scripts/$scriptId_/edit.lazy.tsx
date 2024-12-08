import { ScriptForm } from "@/components/script/ScriptForm";
import { trpc } from "@/trpc.ts";
import type { ScriptInput } from "@/trpc.ts";
import { createLazyFileRoute } from "@tanstack/react-router";

export const Route = createLazyFileRoute("/scripts/$scriptId/edit")({
	component: EditScript,
});

export function EditScript() {
	const { scriptId } = Route.useParams();
	const navigate = Route.useNavigate();
	const getScript = trpc.script.useQuery({ id: scriptId });
	const updateScript = trpc.updateScript.useMutation();

	const handleSubmit = async (values: ScriptInput) => {
		await updateScript.mutateAsync({
			id: scriptId,
			title: values.title,
			description: values.description,
			template: values.template,
		});
		navigate({ to: "/scripts/$scriptId", params: { scriptId } });
	};

	if (!getScript.data) {
		return null;
	}

	const script = getScript.data.script;

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
