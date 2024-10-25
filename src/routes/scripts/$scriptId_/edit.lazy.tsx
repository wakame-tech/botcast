import { ScriptForm } from "@/components/script/ScriptForm";
import type { ScriptEditFormValues } from "@/components/script/ScriptForm";
import { Textarea } from "@/components/ui/textarea";
import { useScript } from "@/hooks/useScript";
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
	const { evaluate, running } = useScript(scriptId);

	const handleSubmit = async (values: ScriptEditFormValues) => {
		await updateScript.mutateAsync({
			id: scriptId,
			title: values.title,
			template: values.template,
		});
		navigate({ to: "/scripts/$scriptId", params: { scriptId } });
	};

	const handleEvaluate = async (values: ScriptEditFormValues) => {
		if (!getScript.data) {
			return;
		}
		await evaluate(values.template);
	};

	if (!getScript.data) {
		return null;
	}

	const script = getScript.data.script;
	// @ts-ignore
	const template = JSON.stringify(script.template, null, 4);

	return (
		<>
			<ScriptForm
				disabled={running}
				values={{
					title: script.title,
					template,
				}}
				onSubmit={handleSubmit}
				onEvaluate={handleEvaluate}
			/>
			{/* @ts-ignore */}
			<Textarea
				rows={10}
				value={JSON.stringify(script.result, null, 4)}
				readOnly
			/>
		</>
	);
}
