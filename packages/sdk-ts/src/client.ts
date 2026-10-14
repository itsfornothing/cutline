import {
  Timeline,
  RenderJob,
  ValidationResponse,
  BatchRecord,
  BatchProgressResponse,
  BrandKit,
  TemplateRecord,
  CiRunReport,
  RenderLane,
} from './types.js';

export interface CutlineClientOptions {
  apiKey?: string;
  baseUrl?: string;
  fetch?: typeof globalThis.fetch;
}

export class CutlineError extends Error {
  constructor(
    message: string,
    public status?: number,
    public code?: string,
    public details?: any
  ) {
    super(message);
    this.name = 'CutlineError';
  }
}

export class Cutline {
  private apiKey: string;
  private baseUrl: string;
  private customFetch: typeof globalThis.fetch;

  public renders: RendersResource;
  public batches: BatchesResource;
  public brandKits: BrandKitsResource;
  public templates: TemplatesResource;
  public usage: UsageResource;

  constructor(options: CutlineClientOptions = {}) {
    const envKey = (globalThis as any).process?.env?.CUTLINE_API_KEY;
    this.apiKey = options.apiKey || (typeof envKey === 'string' ? envKey : '');
    this.baseUrl = (options.baseUrl || 'https://api.cutline.dev').replace(/\/+$/, '');
    this.customFetch = options.fetch || globalThis.fetch;

    this.renders = new RendersResource(this);
    this.batches = new BatchesResource(this);
    this.brandKits = new BrandKitsResource(this);
    this.templates = new TemplatesResource(this);
    this.usage = new UsageResource(this);
  }

  async request<T>(path: string, options: RequestInit = {}): Promise<T> {
    const url = `${this.baseUrl}${path}`;
    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
      Accept: 'application/json, application/problem+json',
      ...(options.headers as Record<string, string>),
    };

    if (this.apiKey) {
      headers['Authorization'] = `Bearer ${this.apiKey}`;
    }

    const res = await this.customFetch(url, { ...options, headers });

    if (!res.ok) {
      let errorBody: any = null;
      try {
        errorBody = await res.json();
      } catch {
        errorBody = { message: await res.text() };
      }
      throw new CutlineError(
        errorBody?.detail || errorBody?.title || errorBody?.message || `Request failed with status ${res.status}`,
        res.status,
        errorBody?.code,
        errorBody
      );
    }

    return res.json() as Promise<T>;
  }
}

export class RendersResource {
  constructor(private client: Cutline) {}

  async create(
    timeline: Timeline,
    options: { idempotencyKey?: string } = {}
  ): Promise<RenderJob> {
    const headers: Record<string, string> = {};
    if (options.idempotencyKey) {
      headers['Idempotency-Key'] = options.idempotencyKey;
    }
    return this.client.request<RenderJob>('/v1/renders', {
      method: 'POST',
      headers,
      body: JSON.stringify(timeline),
    });
  }

  async get(id: string): Promise<RenderJob> {
    return this.client.request<RenderJob>(`/v1/renders/${id}`, {
      method: 'GET',
    });
  }

  async cancel(id: string): Promise<RenderJob> {
    return this.client.request<RenderJob>(`/v1/renders/${id}`, {
      method: 'DELETE',
    });
  }

  async validate(timeline: Timeline): Promise<ValidationResponse> {
    return this.client.request<ValidationResponse>('/v1/renders:validate', {
      method: 'POST',
      body: JSON.stringify(timeline),
    });
  }
}

export class BatchesResource {
  constructor(private client: Cutline) {}

  async create(
    template: Timeline,
    items: Array<Record<string, any>>,
    options: { lane?: RenderLane; priority?: number } = {}
  ): Promise<BatchRecord> {
    return this.client.request<BatchRecord>('/v1/batches', {
      method: 'POST',
      body: JSON.stringify({
        template,
        items,
        lane: options.lane || 'standard',
        priority: options.priority || 0,
      }),
    });
  }

  async get(id: string): Promise<BatchProgressResponse> {
    return this.client.request<BatchProgressResponse>(`/v1/batches/${id}`, {
      method: 'GET',
    });
  }
}

export class BrandKitsResource {
  constructor(private client: Cutline) {}

  async create(kit: {
    brand_id: string;
    name: string;
    fonts?: Record<string, string>;
    colors?: Record<string, string>;
    logo_url?: string;
    intro_bumper_url?: string;
    outro_bumper_url?: string;
  }): Promise<BrandKit> {
    return this.client.request<BrandKit>('/v1/brand-kits', {
      method: 'POST',
      body: JSON.stringify(kit),
    });
  }

  async get(id: string): Promise<BrandKit> {
    return this.client.request<BrandKit>(`/v1/brand-kits/${id}`, {
      method: 'GET',
    });
  }
}

export class TemplatesResource {
  constructor(private client: Cutline) {}

  async create(template: {
    id: string;
    name: string;
    timeline: Timeline;
  }): Promise<TemplateRecord> {
    return this.client.request<TemplateRecord>('/v1/templates', {
      method: 'POST',
      body: JSON.stringify(template),
    });
  }

  async get(id: string): Promise<TemplateRecord> {
    return this.client.request<TemplateRecord>(`/v1/templates/${id}`, {
      method: 'GET',
    });
  }

  async pinGolden(id: string, timestamps: number[]): Promise<TemplateRecord> {
    return this.client.request<TemplateRecord>(`/v1/templates/${id}/golden`, {
      method: 'POST',
      body: JSON.stringify({ timestamps }),
    });
  }

  async runCi(id: string): Promise<CiRunReport> {
    return this.client.request<CiRunReport>(`/v1/templates/${id}/ci-run`, {
      method: 'POST',
    });
  }
}

export class UsageResource {
  constructor(private client: Cutline) {}

  async get(): Promise<any[]> {
    return this.client.request<any[]>('/v1/usage', {
      method: 'GET',
    });
  }
}
