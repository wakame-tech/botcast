import { Button } from "@/components/ui/button";
import { trpc } from "@/trpc.ts";
import { Link, createLazyFileRoute, useNavigate } from "@tanstack/react-router";

export const Route = createLazyFileRoute("/scripts/$scriptId")({
	component: Script,
});

export function Script() {
	const navigate = useNavigate();
	const { scriptId } = Route.useParams();
	const getScript = trpc.script.useQuery({ id: scriptId });
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
			<pre className="p-2 text-lg bg-gray-1">
				<code>
					{/* @ts-ignore: */}
					{JSON.stringify(script.template, null, 4)}
				</code>
			</pre>
			<Link to="/scripts/$scriptId/edit" params={{ scriptId }}>
				<Button>edit</Button>
			</Link>
			<Button onClick={handleDelete}>delete</Button>
		</>
	);
}
