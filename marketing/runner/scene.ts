import { Page } from 'playwright';
import { claimIsBacked, logSkip } from './claims';
import { ScreenCapture, TakeRecording } from './capture';
import { SyntheticCursor } from './cursor';

export interface SceneContext {
  page: Page;
  cursor: SyntheticCursor;
  capture: ScreenCapture;
}

export type SceneFunction = (ctx: SceneContext) => Promise<void>;

export interface SceneDefinition {
  id: string;
  name: string;
  claimId: string;
  targetDurationSec: number;
  voiceover: string;
  onScreenText: string;
  fn: SceneFunction;
}

/**
 * Executes an individual scene take guarded by the claims gate rule.
 */
export async function executeScene(
  ctx: SceneContext,
  def: SceneDefinition
): Promise<string | null> {
  console.log(`\n========================================================`);
  console.log(`[SCENE] Executing ${def.id}: ${def.name}`);
  console.log(`[SCENE] Claim Check: '${def.claimId}'`);
  console.log(`========================================================`);

  // Gate rule: Claim must be BACKED in docs/claims.md
  if (!claimIsBacked(def.claimId)) {
    logSkip(def.id, def.claimId, `Claim '${def.claimId}' is not backed by verifiable physical evidence in claims register.`);
    return null;
  }

  // 1s head handle
  await new Promise((r) => setTimeout(r, 1000));
  const take = await ctx.capture.startTake(def.id);

  try {
    await def.fn(ctx);
    await ctx.capture.assertNoConsoleErrors(ctx.page);
  } catch (err: any) {
    console.error(`[SCENE ERROR] Scene ${def.id} encountered an error:`, err);
    throw err;
  } finally {
    return await take.stop();
  }
}
