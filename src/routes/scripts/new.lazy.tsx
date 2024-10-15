import { ScriptForm } from "@/components/script/ScriptForm";
import type { ScriptEditFormValues } from "@/components/script/ScriptForm";
import { trpc } from "@/trpc.ts";
import { createLazyFileRoute, useNavigate } from "@tanstack/react-router";

export const Route = createLazyFileRoute("/scripts/new")({
	component: NewScript,
});

export function NewScript() {
	const navigate = useNavigate();
	const newScript = trpc.newScript.useMutation();

	const handleSubmit = async (values: ScriptEditFormValues) => {
		const { script } = await newScript.mutateAsync({
			...values,
		});
		navigate({ to: "/scripts/$scriptId", params: { scriptId: script.id } });
	};

	return (
		<div>
			<h1>New Script</h1>
			<ScriptForm onSubmit={handleSubmit} />
		</div>
	);
}
