import { App } from "@/App.tsx";
import { createRootRoute } from "@tanstack/react-router";

export const Route = createRootRoute({
	component: App,
});
