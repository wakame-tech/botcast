import { MailList } from "@/components/mail/MailList";
import { $api } from "@/lib/api_client";
import { $cms } from "@/lib/cms_client";
import { createLazyFileRoute } from "@tanstack/react-router";

export const Route = createLazyFileRoute("/corners/$cornerId")({
	component: Corner,
});

export default function Corner() {
	const { cornerId } = Route.useParams();
	const getCorner = $api.useQuery("get", "/corners/{cornerId}", {
		params: { path: { cornerId } },
	});
	const corner = getCorner.data;
	const collectionId = corner?.cms_collection_id ?? "";

	const getRecords = $cms.useQuery(
		"get",
		"/records/{collectionId}",
		{
			params: { path: { collectionId: collectionId } },
		},
		{ enabled: !!collectionId },
	);
	const records = (getRecords.data ?? []) as Record<string, unknown>[];

	if (!corner) {
		return <div>not found</div>;
	}

	return (
		<div>
			<h1>{corner.title}</h1>
			<p>{corner.description}</p>

			<MailList records={records} />
		</div>
	);
}
