import { Button } from "@/components/ui/button";
import {
	Form,
	FormControl,
	FormField,
	FormItem,
	FormLabel,
} from "@/components/ui/form";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { useEvaluateScript } from "@/hooks/useEvaluateScript";
import { zodResolver } from "@hookform/resolvers/zod";
import { useForm } from "react-hook-form";
import { z } from "zod";

interface ScriptFormProps {
	disabled?: boolean;
	values?: ScriptEditFormValues;
	onSubmit: (values: ScriptEditFormValues) => void;
	onEvaluate?: (values: ScriptEditFormValues) => void;
}

export const MANUSCRIPT_TEMPLATE = JSON.stringify(
	{
		$let: {
			num: {
				$eval: "str(len(get(self).episodes) + 1)",
			},
		},
		in: {
			title: "第${num}話",
			sections: [
				{
					speaker: "urn:voicevox:zunda_normal",
					text: "こんにちは",
					type: "Serif",
				},
			],
		},
	},
	null,
	4,
);

export const SCRIPT_TEMPLATE = JSON.stringify(
	{
		$eval: "1+1",
	},
	null,
	4,
);

const scriptEditFormSchema = z.object({
	title: z.string(),
	template: z.string(),
});

export type ScriptEditFormValues = z.infer<typeof scriptEditFormSchema>;

export function ScriptForm(props: ScriptFormProps) {
	const { evaluate, running, taskResult } = useEvaluateScript();
	const form = useForm<z.infer<typeof scriptEditFormSchema>>({
		resolver: zodResolver(scriptEditFormSchema),
		defaultValues: props.values ?? {
			title: "新しいスクリプト",
			template: SCRIPT_TEMPLATE,
		},
	});

	return (
		<Form {...form}>
			<form onSubmit={form.handleSubmit(props.onSubmit)} className="space-y-8">
				<FormField
					control={form.control}
					name="title"
					render={({ field }) => (
						<FormItem>
							<FormLabel>タイトル</FormLabel>
							<FormControl>
								<Input placeholder="title" {...field} />
							</FormControl>
						</FormItem>
					)}
				/>

				<FormField
					control={form.control}
					name="template"
					render={({ field }) => (
						<FormItem>
							<FormLabel>スクリプト</FormLabel>
							<FormControl>
								<Textarea rows={10} placeholder="template" {...field} />
							</FormControl>
						</FormItem>
					)}
				/>
				<Button disabled={props.disabled} type="submit">
					{props.values ? "更新" : "作成"}
				</Button>
				<Button
					type="button"
					disabled={running}
					onClick={() => evaluate(form.getValues().template)}
				>
					実行
				</Button>
			</form>

			<h2>実行結果</h2>
			<Textarea
				rows={10}
				value={JSON.stringify(taskResult, null, 4)}
				readOnly
			/>
		</Form>
	);
}
