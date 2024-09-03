import { createLazyFileRoute } from '@tanstack/react-router'
import { trpc } from '../trpc'

export const Route = createLazyFileRoute('/')({
    component: Index,
})

function Index() {
    const testQuery = trpc.testGetUserId.useQuery()


    return (
        <div className="p-2">
            <h3>userId: {testQuery.data?.userId}</h3>
        </div>
    )
}
