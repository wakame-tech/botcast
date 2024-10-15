import { trpc } from "@/trpc.ts";
import { useEffect, useState } from "react";

export const useScript = (scriptId: string) => {
    const [taskId, setTaskId] = useState<string | null>(null);
    const getScript = trpc.script.useQuery({ id: scriptId });
    const _updateScript = trpc.updateScript.useMutation();
    const addTask = trpc.addTask.useMutation();
    const getTask = trpc.task.useQuery(
        // biome-ignore lint/style/noNonNullAssertion: conditional fetch
        { id: taskId! },
        { enabled: taskId !== null },
    );

    useEffect(() => {
        const id = setInterval(async () => {
            if (!taskId) {
                return;
            }
            const res = await getTask.refetch();
            if (res.error || !res.data?.task) {
                console.error(res.error);
                return;
            }
            const task = res.data?.task;
            console.log(task);
            if (task.status === "COMPLETED" || task.status === "FAILED") {
                setTaskId(null);
                getScript.refetch();
                clearInterval(id);
            }
        }, 3000);
        return () => clearInterval(id);
    }, [taskId, getTask, getScript]);

    const updateScript = async (title: string, template: string) => {
        await _updateScript.mutateAsync({
            id: scriptId,
            title,
            template,
        });
    };

    const evaluate = async (template: string) => {
        if (!getScript.data) {
            return;
        }
        await updateScript(getScript.data.script.title, template);
        const { task } = await addTask.mutateAsync({
            type: "evaluateScript",
            scriptId,
        });
        setTaskId(task.id);
    };

    return {
        taskId,
        evaluate,
        running: taskId !== null,
    };
};

type Section = {
    type: "Serif";
    speaker: string;
    text: string;
};

export interface Manuscript {
    title: string;
    sections: Section[];
}
