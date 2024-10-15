import { trpc } from "@/trpc.ts";
import { Link, createLazyFileRoute } from "@tanstack/react-router";

export const Route = createLazyFileRoute("/scripts/$scriptId")({
	component: Script,
});

export function Script() {
	const { scriptId } = Route.useParams();
	const getScript = trpc.script.useQuery({ id: scriptId });

	if (!getScript.data) {
		return null;
	}

	const script = getScript.data.script;

	return (
		<>
			<h1>{script.title}</h1>
			{/* @ts-ignore: */}
			<p>{JSON.stringify(script.template)}</p>
			<Link to="/scripts/$scriptId/edit" params={{ scriptId }}>
				edit
			</Link>
		</>
	);
}
