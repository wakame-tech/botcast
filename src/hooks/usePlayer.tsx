import { useRef, useState } from "react";
import ReactPlayer from "react-player";

export const usePlayer = () => {
	const ref = useRef<ReactPlayer>();
	const [seconds, setSeconds] = useState(0);
	const [isPlaying, setPlaying] = useState(false);

	const seek = (seconds: number) => {
		if (ref.current) {
			ref.current.seekTo(seconds, "seconds");
			setPlaying(true);
		}
	};

	const play = () => {
		setPlaying((playing) => !playing);
	};

	const render = (url: string) => {
		return (
			<ReactPlayer
				url={url}
				playing={isPlaying}
				config={{
					file: {
						forceAudio: true,
					},
				}}
				width="100%"
				height="100%"
				ref={(player) => {
					if (player) {
						ref.current = player;
					}
				}}
				progressInterval={300}
				onProgress={({ playedSeconds }) => {
					setSeconds(playedSeconds);
				}}
			/>
		);
	};

	return {
		isPlaying,
		play,
		seconds,
		seek,
		render,
	};
};
