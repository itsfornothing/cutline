/**
 * SMPTE Timecode formatting and parsing utilities.
 */

export function secondsToSMPTE(seconds: number, fps: number = 30): string {
  const totalFrames = Math.max(0, Math.floor(seconds * fps));
  const f = totalFrames % fps;
  const totalSeconds = Math.floor(seconds);
  const s = totalSeconds % 60;
  const totalMinutes = Math.floor(totalSeconds / 60);
  const m = totalMinutes % 60;
  const h = Math.floor(totalMinutes / 60);

  const pad = (n: number, width: number = 2) => n.toString().padStart(width, '0');
  return `${pad(h)}:${pad(m)}:${pad(s)}:${pad(f)}`;
}

export function smpteToSeconds(smpte: string, fps: number = 30): number {
  const parts = smpte.split(':').map((p) => parseInt(p, 10) || 0);
  if (parts.length < 4) return 0;
  const [h, m, s, f] = parts;
  const totalFrames = ((h * 60 + m) * 60 + s) * fps + f;
  return totalFrames / fps;
}

export function formatDuration(seconds: number): string {
  return `${seconds.toFixed(2)}s`;
}

export function formatBilledCost(seconds: number, ratePerMin: number = 0.08): string {
  const billedSeconds = Math.ceil(seconds * 10) / 10;
  const cost = (billedSeconds * (ratePerMin / 60));
  return `$${cost.toFixed(4)}`;
}
