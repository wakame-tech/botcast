import createFetchClient from "openapi-fetch";
import createClient from "openapi-react-query";
import type { paths } from "./cms_api";

const fetchClient = createFetchClient<paths>({
	baseUrl: import.meta.env.VITE_CMS_API_URL ?? "http://localhost:3002",
});

export const $cms = createClient(fetchClient);
