import { Card, CardContent, CardTitle } from "@/components/ui/card";
import { Link, createLazyFileRoute } from "@tanstack/react-router";

export const Route = createLazyFileRoute("/users/$userId")({
	component: User,
});

function User() {
	return (
		<>
			<Card>
				<CardTitle>デバッグ</CardTitle>
				<CardContent>
					<Link to="/tasks">タスク</Link>
					<Link to="/scripts">スクリプト</Link>
				</CardContent>
			</Card>
		</>
	);
}
