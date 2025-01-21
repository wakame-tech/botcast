import { MailList } from "@/components/mail/MailList";
import { trpc } from "@/trpc";
import { createLazyFileRoute } from "@tanstack/react-router";

export const Route = createLazyFileRoute("/corners/$cornerId")({
	component: Corner,
});

export default function Corner() {
	const { cornerId } = Route.useParams();
	const getCorner = trpc.corner.useQuery({ id: cornerId });
	const corner = getCorner.data?.corner;
	const getMails = trpc.mails.useQuery({ cornerId });
	const mails = getMails.data?.mails ?? [];

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
