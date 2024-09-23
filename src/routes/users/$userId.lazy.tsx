import { createLazyFileRoute } from "@tanstack/react-router";
import { trpc } from "../../trpc";

export const Route = createLazyFileRoute("/users/$userId")({
	component: User,
});

function User() {
	const getMe = trpc.me.useQuery();
	if (getMe.error) {
		return <div>Error: {getMe.error.message}</div>;
	}
	if (getMe.isLoading || !getMe.data) {
		return <div>Loading...</div>;
	}
	const user = getMe.data.user;

	return (
		<div>
			<p>name: {user.name}</p>
			<p>email: {user.email}</p>
		</div>
	);
}
