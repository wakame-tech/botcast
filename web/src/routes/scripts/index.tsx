import { $cms, recordToScript } from "@/lib/cms_client";
import { Link, createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/scripts/")({
	component: Scripts,
});

export function Scripts() {
	const { data: records } = $cms.useQuery("get", "/records/{collectionId}", {
		params: { path: { collectionId: "scripts" } },
	});
	const scripts = (records ?? []).map(recordToScript);

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
