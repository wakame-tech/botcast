import { createLazyFileRoute } from '@tanstack/react-router'
import { trpc } from "../trpc.ts";

export const Route = createLazyFileRoute('/')({
    component: Index,
})

function Episodes() {
    const episodeQuery = trpc.episodes.useQuery();

    return (
        <div>
            {JSON.stringify(episodeQuery.data?.episodes)}
        </div>
    );
}

function Tasks() {
    const taskQuery = trpc.tasks.useQuery();

    return (
        <div>
            {JSON.stringify(taskQuery.data?.tasks)}
        </div>
    );
}

function Index() {
    return (
        <div className="p-2">
            <h3>Welcome Home!</h3>
            <Episodes />
            <Tasks />
        </div>
    )
}
