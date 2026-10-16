import { Page } from 'playwright';

export interface Point {
  x: number;
  y: number;
}

export class SyntheticCursor {
  private page: Page;
  private currentPos: Point = { x: 720, y: 450 };

  constructor(page: Page) {
    this.page = page;
  }

  /**
   * Injects the synthetic cursor and keystroke overlay elements into the DOM.
   */
  async init(): Promise<void> {
    await this.page.evaluate(() => {
      if (document.getElementById('cutline-synthetic-cursor')) return;

      // Cursor element
      const cursor = document.createElement('div');
      cursor.id = 'cutline-synthetic-cursor';
      cursor.innerHTML = `
        <svg width="24" height="24" viewBox="0 0 24 24" fill="none" style="filter: drop-shadow(0 2px 5px rgba(0,0,0,0.4));">
          <path d="M5.5 3.2L18.8 12.3C19.7 12.9 19.4 14.3 18.3 14.5L13.1 15.3L16.4 20.8C16.8 21.5 16.5 22.4 15.7 22.8L13.8 23.7C13.1 24.1 12.2 23.8 11.8 23.1L8.5 17.5L5.1 20.9C4.3 21.6 3 21.1 3 20V4.2C3 3.1 4.3 2.4 5.5 3.2Z" fill="#FFFFFF" stroke="#0A0A0A" stroke-width="1.5" stroke-linejoin="round"/>
        </svg>
        <div id="cutline-cursor-ripple" style="position: absolute; width: 32px; height: 32px; border-radius: 50%; border: 2px solid #F59E0B; top: -4px; left: -4px; opacity: 0; transform: scale(0.5); transition: transform 0.25s ease-out, opacity 0.25s ease-out; pointer-events: none;"></div>
      `;
      cursor.style.position = 'fixed';
      cursor.style.left = '720px';
      cursor.style.top = '450px';
      cursor.style.zIndex = '999999';
      cursor.style.pointerEvents = 'none';
      cursor.style.transition = 'transform 0.05s ease';
      document.body.appendChild(cursor);

      // Keystroke overlay badge
      const keyBadge = document.createElement('div');
      keyBadge.id = 'cutline-keystroke-overlay';
      keyBadge.style.position = 'fixed';
      keyBadge.style.bottom = '32px';
      keyBadge.style.left = '50%';
      keyBadge.style.transform = 'translateX(-50%) translateY(20px)';
      keyBadge.style.background = 'rgba(15, 23, 42, 0.92)';
      keyBadge.style.backdropFilter = 'blur(12px)';
      keyBadge.style.border = '1px solid rgba(255, 255, 255, 0.15)';
      keyBadge.style.borderRadius = '8px';
      keyBadge.style.padding = '8px 16px';
      keyBadge.style.fontFamily = 'Geist Mono, monospace';
      keyBadge.style.fontSize = '14px';
      keyBadge.style.fontWeight = 'bold';
      keyBadge.style.color = '#F59E0B';
      keyBadge.style.boxShadow = '0 10px 25px -5px rgba(0, 0, 0, 0.5)';
      keyBadge.style.opacity = '0';
      keyBadge.style.transition = 'opacity 0.2s ease, transform 0.2s ease';
      keyBadge.style.zIndex = '999998';
      keyBadge.style.pointerEvents = 'none';
      document.body.appendChild(keyBadge);
    });
  }

  /**
   * Smoothly moves the cursor along a cubic Bezier eased trajectory.
   */
  async moveTo(target: Point, durationMs: number = 500): Promise<void> {
    const start = { ...this.currentPos };
    const steps = Math.max(15, Math.floor(durationMs / 16)); // ~60 fps steps

    // Randomize control points for natural curved hand motion
    const dx = target.x - start.x;
    const dy = target.y - start.y;
    const cp1 = { x: start.x + dx * 0.25 - dy * 0.1, y: start.y + dy * 0.25 + dx * 0.1 };
    const cp2 = { x: start.x + dx * 0.75 + dy * 0.05, y: start.y + dy * 0.75 - dx * 0.05 };

    for (let i = 1; i <= steps; i++) {
      const t = i / steps;
      // Cubic Bezier formula: B(t) = (1-t)^3 P0 + 3(1-t)^2 t P1 + 3(1-t) t^2 P2 + t^3 P3
      const u = 1 - t;
      const x = u * u * u * start.x + 3 * u * u * t * cp1.x + 3 * u * t * t * cp2.x + t * t * t * target.x;
      const y = u * u * u * start.y + 3 * u * u * t * cp1.y + 3 * u * t * t * cp2.y + t * t * t * target.y;

      await this.page.evaluate(({ curX, curY }) => {
        const el = document.getElementById('cutline-synthetic-cursor');
        if (el) {
          el.style.left = `${curX}px`;
          el.style.top = `${curY}px`;
        }
      }, { curX: x, curY: y });

      await new Promise((r) => setTimeout(r, Math.floor(durationMs / steps)));
    }

    this.currentPos = { ...target };
  }

  /**
   * Performs a click with realistic 120ms hover pause and click ripple animation.
   */
  async click(target?: Point): Promise<void> {
    if (target) {
      await this.moveTo(target, 450);
    }

    // 120 ms hover pause before clicking (rule in Section 2)
    await new Promise((r) => setTimeout(r, 120));

    // Animate click ripple
    await this.page.evaluate(() => {
      const ripple = document.getElementById('cutline-cursor-ripple');
      if (ripple) {
        ripple.style.transform = 'scale(1.4)';
        ripple.style.opacity = '1';
        setTimeout(() => {
          ripple.style.transform = 'scale(2.2)';
          ripple.style.opacity = '0';
        }, 150);
      }
    });

    // Real browser click at current coordinates
    await this.page.mouse.click(this.currentPos.x, this.currentPos.y);
    await new Promise((r) => setTimeout(r, 150));
  }

  /**
   * Types text with human character jitter (45–70 ms/char).
   */
  async typeText(selector: string, text: string): Promise<void> {
    await this.page.focus(selector);
    for (const ch of text) {
      await this.page.keyboard.type(ch);
      const jitter = 45 + Math.floor(Math.random() * 25);
      await new Promise((r) => setTimeout(r, jitter));
    }
  }

  /**
   * Pastes a long block of text with a short typed prefix first.
   */
  async pasteText(selector: string, prefix: string, fullBlock: string): Promise<void> {
    await this.typeText(selector, prefix);
    await new Promise((r) => setTimeout(r, 180));
    // Paste remainder
    await this.page.evaluate(({ sel, content }) => {
      const input = document.querySelector(sel) as HTMLInputElement | HTMLTextAreaElement;
      if (input) {
        input.value = content;
        input.dispatchEvent(new Event('input', { bubbles: true }));
        input.dispatchEvent(new Event('change', { bubbles: true }));
      }
    }, { sel: selector, content: fullBlock });
  }

  /**
   * Shows a keystroke overlay banner for shortcuts (e.g. ⌘K, J, K, L, Shift+→).
   */
  async showKeystroke(label: string, durationMs: number = 800): Promise<void> {
    await this.page.evaluate(({ text }) => {
      const badge = document.getElementById('cutline-keystroke-overlay');
      if (badge) {
        badge.innerText = text;
        badge.style.opacity = '1';
        badge.style.transform = 'translateX(-50%) translateY(0)';
      }
    }, { text: label });

    await new Promise((r) => setTimeout(r, durationMs));

    await this.page.evaluate(() => {
      const badge = document.getElementById('cutline-keystroke-overlay');
      if (badge) {
        badge.style.opacity = '0';
        badge.style.transform = 'translateX(-50%) translateY(20px)';
      }
    });
  }
}
