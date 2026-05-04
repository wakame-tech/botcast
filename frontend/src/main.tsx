import { RouterProvider, createRouter } from "@tanstack/react-router";
import React from "react";
import ReactDOM from "react-dom/client";
import "virtual:uno.css";

import { extend } from "dayjs";
import duration from "dayjs/plugin/duration";

extend(duration);

import { routeTree } from "./routeTree.gen";
const router = createRouter({ routeTree });

declare module "@tanstack/react-router" {
	interface Register {
		router: typeof router;
	}
}

ReactDOM.createRoot(
	// biome-ignore lint:
	document.getElementById("root")!,
).render(
	<React.StrictMode>
		<RouterProvider router={router} />
	</React.StrictMode>,
);
