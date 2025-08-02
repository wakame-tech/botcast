import { MailList } from "@/components/mail/MailList";
import { $api } from "@/lib/api_client";
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
	const getMails = $api.useQuery("get", "/corners/{cornerId}/mails", {
		params: { path: { cornerId } },
	});
	const mails = getMails.data ?? [];

	if (!corner) {
		return <div>not found</div>;
	}

	return (
		<div>
			<h1>{corner.title}</h1>
			<p>{corner.description}</p>

			<MailList mails={mails} />
		</div>
	);
}
