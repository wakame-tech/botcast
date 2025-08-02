import { JsonSchemaForm } from "@/components/JsonSchemaForm";
import { Button } from "@/components/ui/button";
import { $api } from "@/lib/api_client";
import { Link, createLazyFileRoute, useNavigate } from "@tanstack/react-router";
import { useState } from "react";

export const Route = createLazyFileRoute("/scripts/$scriptId")({
	component: Script,
});

export function Script() {
	const navigate = useNavigate();
	const { scriptId } = Route.useParams();
	const getScript = $api.useQuery("get", "/scripts/{scriptId}", {
		params: { path: { scriptId } },
	});
	const [_, setParameters] = useState<Record<string, unknown>>({});
	const deleteScript = $api.useMutation("delete", "/scripts/{scriptId}");

	if (!getScript.data) {
		return null;
	}

	const handleDelete = async () => {
		await deleteScript.mutateAsync({ params: { path: { scriptId } } });
		navigate({ to: "/scripts" });
	};

	const script = getScript.data;

	return (
		<>
			<h1>{script.title}</h1>

			<p>{script.description}</p>

			<Link to="/scripts/$scriptId/edit" params={{ scriptId }}>
				<Button>edit</Button>
			</Link>
			<Button onClick={handleDelete}>delete</Button>

			<pre className="p-2 text-sm bg-gray-1">
				<code>{JSON.stringify(script.template, null, 4)}</code>
			</pre>

			{Object.keys(script.arguments).length !== 0 && (
				<JsonSchemaForm
					schema={script.arguments}
					onChange={(e) => setParameters(e)}
				/>
			)}
		</>
	);
}
