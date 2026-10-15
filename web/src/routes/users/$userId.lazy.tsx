import { createLazyFileRoute } from "@tanstack/react-router";

export const Route = createLazyFileRoute("/users/$userId")({
	component: User,
});

function User() {
	const { userId } = Route.useParams();

	return (
		<div className="flex flex-col gap-6 p-4">
			<h2 className="text-xl font-bold">ユーザー</h2>
			<p className="text-sm text-gray-500">{userId}</p>
		</div>
	);
}
