import { Cutline, CutlineError } from '../dist/index.js';
import assert from 'node:assert';
import test from 'node:test';

test('Cutline SDK Client Suite', async (t) => {
  await t.test('Client initializes with API key and custom baseUrl', () => {
    const client = new Cutline({
      apiKey: 'cl_test_12345',
      baseUrl: 'http://localhost:8080/v1/',
    });
    assert.strictEqual(client['apiKey'], 'cl_test_12345');
    assert.strictEqual(client['baseUrl'], 'http://localhost:8080/v1');
  });

  await t.test('renders.create sends authorization and idempotency headers', async () => {
    let capturedUrl = '';
    let capturedHeaders: Record<string, string> = {};
    let capturedBody = '';

    const mockFetch = async (input: RequestInfo | URL, init?: RequestInit): Promise<Response> => {
      capturedUrl = input.toString();
      capturedHeaders = init?.headers as Record<string, string>;
      capturedBody = init?.body as string;

      return {
        ok: true,
        status: 201,
        json: async () => ({
          id: '01927c34-8c88-7e23-81a1-8d0739c9f28c',
          status: 'queued',
          lane: 'standard',
          progress: 0,
        }),
      } as unknown as Response;
    };

    const client = new Cutline({
      apiKey: 'cl_test_secret',
      fetch: mockFetch,
    });

    const timeline = {
      schema_version: '1.0',
      outputs: [
        {
          id: 'out',
          aspect: '16:9',
          resolution: { width: 1920, height: 1080 },
          fps: 30,
          format: 'mp4' as const,
        },
      ],
      tracks: [
        {
          id: 'v1',
          kind: 'video' as const,
          clips: [{ asset: 'bg', start: 0, duration: 10 }],
        },
      ],
    };

    const job = await client.renders.create(timeline, {
      idempotencyKey: 'idem-key-777',
    });

    assert.strictEqual(capturedUrl, 'https://api.cutline.dev/v1/renders');
    assert.strictEqual(capturedHeaders['Authorization'], 'Bearer cl_test_secret');
    assert.strictEqual(capturedHeaders['Idempotency-Key'], 'idem-key-777');
    assert.strictEqual(job.id, '01927c34-8c88-7e23-81a1-8d0739c9f28c');
    assert.strictEqual(job.status, 'queued');
  });

  await t.test('renders.validate performs dry-run quote', async () => {
    const mockFetch = async (): Promise<Response> => {
      return {
        ok: true,
        status: 200,
        json: async () => ({
          valid: true,
          estimated_duration_seconds: 12.0,
          estimated_total_usd: 0.016,
          quote: { billed_seconds: 12.0, estimated_total_usd: 0.016 },
          warnings: [],
        }),
      } as unknown as Response;
    };

    const client = new Cutline({ fetch: mockFetch });
    const res = await client.renders.validate({
      schema_version: '1.0',
      outputs: [],
      tracks: [],
    });

    assert.strictEqual(res.valid, true);
    assert.strictEqual(res.estimated_total_usd, 0.016);
  });

  await t.test('CutlineError wraps RFC 9457 Problem Details on failure', async () => {
    const mockFetch = async (): Promise<Response> => {
      return {
        ok: false,
        status: 400,
        json: async () => ({
          type: 'https://api.cutline.dev/errors/validation-failed',
          title: 'Validation Failed',
          code: 'INVALID_TIMELINE',
          detail: 'Output resolution must be greater than 0',
        }),
      } as unknown as Response;
    };

    const client = new Cutline({ fetch: mockFetch });

    await assert.rejects(
      async () => {
        await client.renders.create({
          schema_version: '1.0',
          outputs: [],
          tracks: [],
        });
      },
      (err: any) => {
        assert.ok(err instanceof CutlineError);
        assert.strictEqual(err.status, 400);
        assert.strictEqual(err.code, 'INVALID_TIMELINE');
        assert.strictEqual(err.message, 'Output resolution must be greater than 0');
        return true;
      }
    );
  });
});
