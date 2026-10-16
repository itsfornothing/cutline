import fs from 'fs';
import path from 'path';

export interface ClaimStatus {
  claimId: string;
  claimName: string;
  status: 'BACKED' | 'UNBACKED' | 'RETRACTED';
}

/**
 * Checks whether a given claim ID is officially marked as BACKED in docs/claims.md.
 */
export function claimIsBacked(claimId: string): boolean {
  const claimsPath = path.resolve('docs/claims.md');
  if (!fs.existsSync(claimsPath)) {
    console.error(`Claims register not found at ${claimsPath}`);
    return false;
  }

  const content = fs.readFileSync(claimsPath, 'utf8');
  const lines = content.split('\n');

  for (const line of lines) {
    if (line.includes(`\`${claimId}\``)) {
      if (line.includes('**BACKED**')) {
        return true;
      } else {
        return false;
      }
    }
  }

  return false;
}

/**
 * Logs a skipped scene to marketing/skipped-scenes.md if not already recorded.
 */
export function logSkip(sceneId: string, claimId: string, reason: string): void {
  const skipLogPath = path.resolve('marketing/skipped-scenes.md');
  const logEntry = `\n| **${sceneId}** | Auto-detected | \`${claimId}\` | **GATED** | ${reason} |`;
  if (fs.existsSync(skipLogPath)) {
    const existing = fs.readFileSync(skipLogPath, 'utf8');
    if (!existing.includes(`**${sceneId}**`)) {
      fs.appendFileSync(skipLogPath, logEntry);
    }
  }
  console.log(`[GATE RULE] Scene ${sceneId} with claim '${claimId}' is GATED/SKIPPED: ${reason}`);
}
