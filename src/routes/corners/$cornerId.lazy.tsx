import { createLazyFileRoute } from "@tanstack/react-router";

export const Route = createLazyFileRoute("/corners/$cornerId")({
	component: () => <div>Hello /corners/$cornerId!</div>,
});
