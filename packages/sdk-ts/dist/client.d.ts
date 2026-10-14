import { Timeline, RenderJob, ValidationResponse, BatchRecord, BatchProgressResponse, BrandKit, TemplateRecord, CiRunReport, RenderLane } from './types.js';
export interface CutlineClientOptions {
    apiKey?: string;
    baseUrl?: string;
    fetch?: typeof globalThis.fetch;
}
export declare class CutlineError extends Error {
    status?: number | undefined;
    code?: string | undefined;
    details?: any | undefined;
    constructor(message: string, status?: number | undefined, code?: string | undefined, details?: any | undefined);
}
export declare class Cutline {
    private apiKey;
    private baseUrl;
    private customFetch;
    renders: RendersResource;
    batches: BatchesResource;
    brandKits: BrandKitsResource;
    templates: TemplatesResource;
    usage: UsageResource;
    constructor(options?: CutlineClientOptions);
    request<T>(path: string, options?: RequestInit): Promise<T>;
}
export declare class RendersResource {
    private client;
    constructor(client: Cutline);
    create(timeline: Timeline, options?: {
        idempotencyKey?: string;
    }): Promise<RenderJob>;
    get(id: string): Promise<RenderJob>;
    cancel(id: string): Promise<RenderJob>;
    validate(timeline: Timeline): Promise<ValidationResponse>;
}
export declare class BatchesResource {
    private client;
    constructor(client: Cutline);
    create(template: Timeline, items: Array<Record<string, any>>, options?: {
        lane?: RenderLane;
        priority?: number;
    }): Promise<BatchRecord>;
    get(id: string): Promise<BatchProgressResponse>;
}
export declare class BrandKitsResource {
    private client;
    constructor(client: Cutline);
    create(kit: {
        brand_id: string;
        name: string;
        fonts?: Record<string, string>;
        colors?: Record<string, string>;
        logo_url?: string;
        intro_bumper_url?: string;
        outro_bumper_url?: string;
    }): Promise<BrandKit>;
    get(id: string): Promise<BrandKit>;
}
export declare class TemplatesResource {
    private client;
    constructor(client: Cutline);
    create(template: {
        id: string;
        name: string;
        timeline: Timeline;
    }): Promise<TemplateRecord>;
    get(id: string): Promise<TemplateRecord>;
    pinGolden(id: string, timestamps: number[]): Promise<TemplateRecord>;
    runCi(id: string): Promise<CiRunReport>;
}
export declare class UsageResource {
    private client;
    constructor(client: Cutline);
    get(): Promise<any[]>;
}
//# sourceMappingURL=client.d.ts.map