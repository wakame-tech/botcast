import { Task } from "@prisma/client"

interface TaskListProps {
    tasks: Task[]
}

export function TaskList(props: TaskListProps) {
    const textStyles = `px-4 py-2 font-medium text-gray-900`

    return (
        <>
            <table className="overflow-x-auto divide-y-2 divide-gray-200">
                <thead>
                    <tr>
                        <th className={textStyles}>ID</th>
                        <th className={textStyles}>Status</th>
                    </tr>
                </thead>
                <tbody className="divide-y-2 divide-gray-200">
                    {props.tasks.map((task) => (
                        <tr key={task.id}>
                            <td className={textStyles}>{task.id}</td>
                            <td className={textStyles}>{task.status}</td>
                        </tr>
                    ))}
                </tbody>
            </table>
        </>
    )
}
