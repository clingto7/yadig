import { usePlayer } from "@/lib/player-context";
import { Play, Pause, X, Volume2, VolumeX } from "lucide-react";
import { useState } from "react";

function formatTime(seconds: number): string {
  if (!seconds || !isFinite(seconds)) return "0:00";
  const m = Math.floor(seconds / 60);
  const s = Math.floor(seconds % 60);
  return `${m}:${s.toString().padStart(2, "0")}`;
}

export function AudioPlayer() {
  const { current, isPlaying, currentTime, duration, volume, pause, resume, stop, seek, setVolume } = usePlayer();
  const [prevVolume, setPrevVolume] = useState(0.8);

  if (!current) return null;

  const progress = duration > 0 ? (currentTime / duration) * 100 : 0;

  function handleSeek(e: React.MouseEvent<HTMLDivElement>) {
    const rect = e.currentTarget.getBoundingClientRect();
    const pct = (e.clientX - rect.left) / rect.width;
    seek(pct * duration);
  }

  function toggleMute() {
    if (volume > 0) {
      setPrevVolume(volume);
      setVolume(0);
    } else {
      setVolume(prevVolume || 0.8);
    }
  }

  return (
    <div className="fixed bottom-0 left-0 right-0 z-50 border-t border-border bg-card/95 backdrop-blur supports-[backdrop-filter]:bg-card/80">
      {/* Progress bar (clickable) */}
      <div
        className="relative h-1 w-full cursor-pointer bg-secondary group"
        onClick={handleSeek}
      >
        <div
          className="absolute left-0 top-0 h-full bg-primary transition-[width] duration-100"
          style={{ width: `${progress}%` }}
        />
        <div
          className="absolute top-1/2 h-3 w-3 -translate-y-1/2 rounded-full bg-primary opacity-0 transition-opacity group-hover:opacity-100"
          style={{ left: `calc(${progress}% - 6px)` }}
        />
      </div>

      <div className="flex items-center gap-4 px-4 py-2">
        {/* Cover image */}
        {current.imageUrl ? (
          <img
            src={current.imageUrl}
            alt={current.title}
            className="h-10 w-10 flex-shrink-0 rounded object-cover bg-secondary"
          />
        ) : (
          <div className="flex h-10 w-10 flex-shrink-0 items-center justify-center rounded bg-secondary">
            <Play className="h-4 w-4 text-muted-foreground" />
          </div>
        )}

        {/* Track info */}
        <div className="min-w-0 flex-1">
          <p className="truncate text-sm font-medium">{current.title}</p>
          <p className="truncate text-xs text-muted-foreground">
            {current.author ?? current.sourceId}
          </p>
        </div>

        {/* Time */}
        <span className="text-xs text-muted-foreground tabular-nums">
          {formatTime(currentTime)} / {formatTime(duration || (current.duration ?? 0))}
        </span>

        {/* Play/Pause */}
        <button
          onClick={() => isPlaying ? pause() : resume()}
          className="flex h-8 w-8 items-center justify-center rounded-full bg-primary text-primary-foreground hover:bg-primary/90"
        >
          {isPlaying ? <Pause className="h-4 w-4" /> : <Play className="h-4 w-4 ml-0.5" />}
        </button>

        {/* Volume */}
        <button
          onClick={toggleMute}
          className="text-muted-foreground hover:text-foreground"
        >
          {volume === 0 ? <VolumeX className="h-4 w-4" /> : <Volume2 className="h-4 w-4" />}
        </button>
        <input
          type="range"
          min={0}
          max={1}
          step={0.01}
          value={volume}
          onChange={(e) => setVolume(parseFloat(e.target.value))}
          className="w-16 accent-primary"
        />

        {/* Close */}
        <button
          onClick={stop}
          className="text-muted-foreground hover:text-foreground"
          title="Stop"
        >
          <X className="h-4 w-4" />
        </button>
      </div>
    </div>
  );
}
