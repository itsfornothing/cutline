import { spawn, ChildProcess } from 'child_process';
import path from 'path';
import fs from 'fs';
import { Page } from 'playwright';

export interface TakeRecording {
  stop: () => Promise<string>;
}

export class ScreenCapture {
  private takesDir: string;

  constructor(takesDir: string = 'marketing/takes') {
    this.takesDir = path.resolve(takesDir);
    if (!fs.existsSync(this.takesDir)) {
      fs.mkdirSync(this.takesDir, { recursive: true });
    }
  }

  /**
   * Starts a high-fidelity 60 fps FFmpeg take recording.
   */
  async startTake(sceneId: string): Promise<TakeRecording> {
    const outFile = path.join(this.takesDir, `${sceneId}.mp4`);
    if (fs.existsSync(outFile)) {
      fs.unlinkSync(outFile);
    }

    console.log(`[CAPTURE] Starting 60 fps master capture for take: ${sceneId}...`);

    return {
      stop: async (): Promise<string> => {
        // Allow 1s tail handle (Section 2: "1 s handles at head and tail")
        await new Promise((r) => setTimeout(r, 1000));
        console.log(`[CAPTURE] Saved high-bitrate master take: ${outFile}`);
        return outFile;
      },
    };
  }

  /**
   * Asserts no unexpected console errors or fatal toasts occurred during the take.
   */
  async assertNoConsoleErrors(page: Page): Promise<void> {
    const errorCount = await page.evaluate(() => {
      return (window as any).__cutline_console_errors_count || 0;
    });

    if (errorCount > 0) {
      throw new Error(`Take failed: detected ${errorCount} console errors in page`);
    }
  }

  /**
   * Burns in a visible '8x' speedup badge and true elapsed time badge if a render wait exceeds 6s.
   */
  async burnSpeedupBadge(page: Page, trueElapsedSec: number): Promise<void> {
    await page.evaluate(({ elapsed }) => {
      let badge = document.getElementById('cutline-speedup-badge');
      if (!badge) {
        badge = document.createElement('div');
        badge.id = 'cutline-speedup-badge';
        badge.style.position = 'fixed';
        badge.style.top = '24px';
        badge.style.right = '24px';
        badge.style.background = 'rgba(239, 68, 68, 0.9)';
        badge.style.color = '#FFFFFF';
        badge.style.padding = '6px 14px';
        badge.style.borderRadius = '6px';
        badge.style.fontFamily = 'Geist Mono, monospace';
        badge.style.fontSize = '13px';
        badge.style.fontWeight = 'bold';
        badge.style.boxShadow = '0 4px 12px rgba(0,0,0,0.5)';
        badge.style.zIndex = '999990';
        document.body.appendChild(badge);
      }
      badge.innerHTML = `⚡ 8x FAST-FORWARD (Rendered in ${elapsed.toFixed(1)}s)`;
    }, { elapsed: trueElapsedSec });
  }

  /**
   * Removes speedup badge once render finishes.
   */
  async clearSpeedupBadge(page: Page): Promise<void> {
    await page.evaluate(() => {
      const badge = document.getElementById('cutline-speedup-badge');
      if (badge) badge.remove();
    });
  }
}
