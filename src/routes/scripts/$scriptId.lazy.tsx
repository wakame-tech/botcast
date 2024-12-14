import { ArgumentsForm } from "@/components/script/ArgumentsForm";
import { Button } from "@/components/ui/button";
import { Textarea } from "@/components/ui/textarea";
import { useEvaluateScript } from "@/hooks/useEvaluateScript";
import { trpc } from "@/trpc.ts";
import { Link, createLazyFileRoute, useNavigate } from "@tanstack/react-router";
import { useState } from "react";

export const Route = createLazyFileRoute("/scripts/$scriptId")({
	component: Script,
});

export function Script() {
	const navigate = useNavigate();
	const { scriptId } = Route.useParams();
	const getScript = trpc.script.useQuery({ id: scriptId });
	const [parameters, setParameters] = useState<Record<string, unknown>>({});
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

			{Object.keys(script.arguments).length !== 0 && (
				<ArgumentsForm
					schema={script.arguments}
					onChange={(e) => setParameters(e)}
				/>
			)}

			<Button
				type="button"
				disabled={running}
				onClick={() => evaluate(script.template, parameters)}
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
