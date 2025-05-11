import { $api } from "@/lib/api_client";
import { Link, createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/scripts/")({
	component: Scripts,
});

export function Scripts() {
	const getScripts = $api.useQuery("get", "/scripts");

	if (!getScripts.data) {
		return null;
	}

	const scripts = getScripts.data;

	return (
		<>
			<Link to="/scripts/new">New Script</Link>

			<ul>
				{scripts.map((script) => (
					<li key={script.id}>
						<Link to="/scripts/$scriptId" params={{ scriptId: script.id }}>
							{script.title}
						</Link>
					</li>
				))}
			</ul>
		</>
	);
}
