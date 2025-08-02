import type { Episode } from "@/lib/api_client";
import { atom, useAtom } from "jotai";
import { useRef } from "react";
import ReactPlayer from "react-player";

type PlayerState = {
	episode: Episode | null;
	seconds: number;
	isPlaying: boolean;
};

const playerEpisodeAtom = atom<PlayerState>({
	episode: null,
	seconds: 0,
	isPlaying: false,
});

export const usePlayer = () => {
	const [state, setState] = useAtom(playerEpisodeAtom);
	const ref = useRef<ReactPlayer>();

	const setEpisode = (episode: Episode) => {
		setState((state) => ({
			...state,
			episode,
		}));
	};

	const playOrPause = () => {
		setState((state) => ({
			...state,
			isPlaying: !state.isPlaying,
		}));
	};

	const seekTo = (seconds: number) => {
		console.log(ref.current);
		ref.current?.seekTo(seconds, "seconds");
	};

	const render = (url: string | null) => {
		return (
			<>
				<ReactPlayer
					url={url ?? ""}
					playing={state.isPlaying}
					config={{
						file: {
							forceAudio: true,
						},
					}}
					width="100%"
					height="100%"
					ref={(e) => {
						if (e) {
							ref.current = e;
						}
					}}
					progressInterval={300}
					onProgress={({ playedSeconds }) => {
						setState({
							...state,
							seconds: playedSeconds,
						});
					}}
				/>
			</>
		);
	};

	return {
		state,
		setEpisode,
		playOrPause,
		seekTo,
		render,
	};
};
