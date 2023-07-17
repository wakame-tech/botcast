import { createTRPCReact } from "@trpc/react-query";
import type { AppRouter } from "../../botcast-api/src";

export const trpc = createTRPCReact<AppRouter>();
