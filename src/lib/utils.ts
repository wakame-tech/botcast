import type { ClassValue } from "clsx";
import { clsx } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
	return twMerge(clsx(inputs));
}

/// all keys of K mapped to string
export type MappedString<T, K> = {
	[P in keyof T]: P extends K ? string : T[P];
};
