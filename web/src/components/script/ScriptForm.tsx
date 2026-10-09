import { Button } from "@/components/ui/button";
import {
	Form,
	FormControl,
	FormField,
	FormItem,
	FormLabel,
	FormMessage,
} from "@/components/ui/form";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { ScriptInputSchema } from "@/lib/api_client";
import { zodResolver } from "@hookform/resolvers/zod";
import { useForm } from "react-hook-form";
import { z } from "zod";

/** JSON Schema として解釈できるオブジェクトの JSON 文字列か */
const isJsonObject = (text: string): boolean => {
	try {
		const v = JSON.parse(text);
		return typeof v === "object" && v !== null && !Array.isArray(v);
	} catch {
		return false;
	}
};

const ScriptFormSchema = ScriptInputSchema.extend({
	/** スクリプトが受け取る引数の JSON Schema (JSON 文字列) */
	arguments: z.string().refine(isJsonObject, {
		message: "JSON のオブジェクトとして不正です",
	}),
});

export type ScriptFormValues = z.infer<typeof ScriptFormSchema>;

/** フォームの値を CMS に保存する `data` の形に変換する */
export function toScriptData(values: ScriptFormValues) {
	return {
		title: values.title,
		description: values.description,
		template: values.template,
		arguments: JSON.parse(values.arguments) as Record<string, unknown>,
	};
}

interface ScriptFormProps {
	disabled?: boolean;
	values?: ScriptFormValues;
	onSubmit: (values: ScriptFormValues) => void;
}

export function ScriptForm(props: ScriptFormProps) {
	const form = useForm<ScriptFormValues>({
		resolver: zodResolver(ScriptFormSchema),
		defaultValues:
			props.values ??
			({
				title: "NewScript",
				description: "",
				template: "",
				arguments: "{}",
			} satisfies ScriptFormValues),
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
					name="description"
					render={({ field }) => (
						<FormItem>
							<FormLabel>説明</FormLabel>
							<FormControl>
								<Input
									placeholder="description"
									{...field}
									value={field.value ?? undefined}
								/>
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

				<FormField
					control={form.control}
					name="arguments"
					render={({ field }) => (
						<FormItem>
							<FormLabel>引数 (JSON Schema)</FormLabel>
							<FormControl>
								<Textarea
									rows={8}
									className="font-mono"
									placeholder='{"type":"object","properties":{}}'
									{...field}
								/>
							</FormControl>
							<FormMessage />
						</FormItem>
					)}
				/>
				<Button disabled={props.disabled} type="submit">
					{props.values ? "更新" : "作成"}
				</Button>
			</form>
		</Form>
	);
}
