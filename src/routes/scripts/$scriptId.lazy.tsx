import { SchemaForm } from "@/components/script/SchemaForm";
import { Button } from "@/components/ui/button";
import { Textarea } from "@/components/ui/textarea";
import { useEvaluateScript } from "@/hooks/useEvaluateScript";
import { trpc } from "@/trpc.ts";
import { Link, createLazyFileRoute, useNavigate } from "@tanstack/react-router";

export const Route = createLazyFileRoute("/scripts/$scriptId")({
	component: Script,
});

export function Script() {
	const navigate = useNavigate();
	const { scriptId } = Route.useParams();
	const getScript = trpc.script.useQuery({ id: scriptId });
	const { evaluate, running, taskResult } = useEvaluateScript();
	const deleteScript = trpc.deleteScript.useMutation();

	if (!getScript.data) {
		return null;
	}

	const handleDelete = async () => {
		await deleteScript.mutateAsync({ id: scriptId });
		navigate({ to: "/scripts" });
	};

	const script = getScript.data.script;

	return (
		<>
			<h1>{script.title}</h1>

			<Link to="/scripts/$scriptId/edit" params={{ scriptId }}>
				<Button>edit</Button>
			</Link>
			<Button onClick={handleDelete}>delete</Button>

			<pre className="p-2 text-sm bg-gray-1">
				<code>{JSON.stringify(script.template, null, 4)}</code>
			</pre>

			{script.template.parameters && (
				<SchemaForm schema={script.template.parameters} />
			)}

			<Button
				type="button"
				disabled={running}
				onClick={() => evaluate(script.template)}
			>
				実行
			</Button>

			<h2>実行結果</h2>
			<Textarea
				rows={10}
				value={JSON.stringify(taskResult, null, 4)}
				readOnly
			/>
		</>
	);
}
