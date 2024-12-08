import { usePlayer } from "@/hooks/usePlayer";
import { formatMmss } from "./EpisodeList";

export function EpisodePlayer() {
	const { state, playOrPause, render } = usePlayer();
	const icon = state.isPlaying ? "i-solar:pause-bold" : "i-solar:play-bold";
	const iconColor = state.episode ? "text-white" : "text-gray-300";

	return (
		<div className="fixed bottom-0 left-0 h-16 w-full bg-teal-500">
			<div className="flex items-center px-4 gap-4 text-white">
				<button
					className={`${icon} ${iconColor} w-2em h-2em`}
					disabled={!state.episode}
					type="button"
					onClick={() => playOrPause()}
				/>
				<p className="grow text-xl align-middle font-bold">
					{state.episode?.title ?? "---"}
				</p>
				<p>
					{formatMmss(state.seconds)} /{" "}
					{formatMmss(state.episode?.duration_sec ?? null)}
				</p>
			</div>
			{render(state.episode?.audio_url ?? null)}
		</div>
	);
}
