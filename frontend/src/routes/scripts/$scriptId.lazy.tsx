import { JsonSchemaForm } from "@/components/JsonSchemaForm";
import { Button } from "@/components/ui/button";
import { $cms, recordToScript } from "@/lib/cms_client";
import { Link, createLazyFileRoute, useNavigate } from "@tanstack/react-router";
import { useState } from "react";

export const Route = createLazyFileRoute("/scripts/$scriptId")({
	component: Script,
});

export function Script() {
	const navigate = useNavigate();
	const { scriptId } = Route.useParams();
	const { data: scriptRecord } = $cms.useQuery(
		"get",
		"/records/{collectionId}/{recordId}",
		{ params: { path: { collectionId: "scripts", recordId: scriptId } } },
	);
	const [_, setParameters] = useState<Record<string, unknown>>({});
	const deleteScript = $cms.useMutation(
		"delete",
		"/records/{collectionId}/{recordId}",
	);

	if (!scriptRecord) {
		return null;
	}

	const handleDelete = async () => {
		await deleteScript.mutateAsync({
			params: { path: { collectionId: "scripts", recordId: scriptId } },
		});
		navigate({ to: "/scripts" });
	};

	const script = recordToScript(scriptRecord);

	return (
		<>
			<h1>{script.title}</h1>

			<p>{script.description}</p>

			<Link to="/scripts/$scriptId/edit" params={{ scriptId }}>
				<Button>edit</Button>
			</Link>
			<Button onClick={handleDelete}>delete</Button>

			<pre className="p-2 text-sm bg-gray-1">
				<code>{script.template}</code>
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
