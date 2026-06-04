import { createContext, useContext, useState, useRef, useCallback, type ReactNode } from "react";
import type { ContentItem } from "@/types/source";

interface PlayerState {
  current: ContentItem | null;
  isPlaying: boolean;
  currentTime: number;
  duration: number;
  volume: number;
}

interface PlayerContextValue extends PlayerState {
  play: (item: ContentItem) => void;
  pause: () => void;
  resume: () => void;
  stop: () => void;
  seek: (time: number) => void;
  setVolume: (v: number) => void;
}

const PlayerContext = createContext<PlayerContextValue | null>(null);

export function usePlayer(): PlayerContextValue {
  const ctx = useContext(PlayerContext);
  if (!ctx) throw new Error("usePlayer must be used within PlayerProvider");
  return ctx;
}

export function PlayerProvider({ children }: { children: ReactNode }) {
  const audioRef = useRef<HTMLAudioElement | null>(null);
  const [current, setCurrent] = useState<ContentItem | null>(null);
  const [isPlaying, setIsPlaying] = useState(false);
  const [currentTime, setCurrentTime] = useState(0);
  const [duration, setDuration] = useState(0);
  const [volume, setVolumeState] = useState(0.8);

  const getAudio = useCallback(() => {
    if (!audioRef.current) {
      audioRef.current = new Audio();
      audioRef.current.volume = volume;

      audioRef.current.addEventListener("timeupdate", () => {
        setCurrentTime(audioRef.current?.currentTime ?? 0);
      });
      audioRef.current.addEventListener("loadedmetadata", () => {
        setDuration(audioRef.current?.duration ?? 0);
      });
      audioRef.current.addEventListener("ended", () => {
        setIsPlaying(false);
      });
      audioRef.current.addEventListener("pause", () => {
        setIsPlaying(false);
      });
      audioRef.current.addEventListener("play", () => {
        setIsPlaying(true);
      });
    }
    return audioRef.current;
  }, [volume]);

  const play = useCallback((item: ContentItem) => {
    if (!item.audioUrl) return;
    const audio = getAudio();

    // If clicking on the same track, toggle pause/resume
    if (current?.audioUrl === item.audioUrl) {
      if (isPlaying) {
        audio.pause();
      } else {
        audio.play().catch(() => {});
      }
      return;
    }

    audio.src = item.audioUrl;
    audio.currentTime = 0;
    setCurrent(item);
    setCurrentTime(0);
    setDuration(item.duration ?? 0);
    audio.play().catch(() => {});
  }, [current, isPlaying, getAudio]);

  const pause = useCallback(() => {
    audioRef.current?.pause();
  }, []);

  const resume = useCallback(() => {
    audioRef.current?.play().catch(() => {});
  }, []);

  const stop = useCallback(() => {
    if (audioRef.current) {
      audioRef.current.pause();
      audioRef.current.currentTime = 0;
    }
    setCurrent(null);
    setIsPlaying(false);
    setCurrentTime(0);
    setDuration(0);
  }, []);

  const seek = useCallback((time: number) => {
    if (audioRef.current) {
      audioRef.current.currentTime = time;
    }
  }, []);

  const setVolume = useCallback((v: number) => {
    setVolumeState(v);
    if (audioRef.current) {
      audioRef.current.volume = v;
    }
  }, []);

  return (
    <PlayerContext.Provider
      value={{ current, isPlaying, currentTime, duration, volume, play, pause, resume, stop, seek, setVolume }}
    >
      {children}
    </PlayerContext.Provider>
  );
}
