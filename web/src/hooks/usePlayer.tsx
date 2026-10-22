import type { Episode } from "@/lib/cms_client";
import { atom, useAtom } from "jotai";
import ReactPlayer from "react-player";

type PlayerState = {
	episode: Episode | null;
	seconds: number;
	isPlaying: boolean;
	/** 音声の読み込みが終わったら移動する再生位置 (別のエピソードに切り替えて途中から再生するとき) */
	pendingSeek: number | null;
};

const playerEpisodeAtom = atom<PlayerState>({
	episode: null,
	seconds: 0,
	isPlaying: false,
	pendingSeek: null,
});

// プレイヤーは EpisodePlayer が 1 つだけ描画する。どこから usePlayer を呼んでもそれを操作できるようにモジュールで持つ
const playerRef: { current: ReactPlayer | null } = { current: null };

export const usePlayer = () => {
	const [state, setState] = useAtom(playerEpisodeAtom);

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

	/** episode の seconds 秒から再生する。別のエピソードなら切り替えて、読み込みが終わってから移動する */
	const playFrom = (episode: Episode, seconds: number) => {
		if (state.episode?.id === episode.id && playerRef.current) {
			playerRef.current.seekTo(seconds, "seconds");
			setState((state) => ({ ...state, seconds, isPlaying: true }));
			return;
		}
		setState((state) => ({
			...state,
			episode,
			seconds,
			isPlaying: true,
			pendingSeek: seconds,
		}));
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
						playerRef.current = e;
					}}
					progressInterval={300}
					onReady={(player) => {
						if (state.pendingSeek !== null) {
							player.seekTo(state.pendingSeek, "seconds");
							setState((state) => ({ ...state, pendingSeek: null }));
						}
					}}
					onProgress={({ playedSeconds }) => {
						setState((state) => ({
							...state,
							seconds: playedSeconds,
						}));
					}}
				/>
			</>
		);
	};

	return {
		state,
		setEpisode,
		playOrPause,
		playFrom,
		render,
	};
};
