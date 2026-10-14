export class CutlineError extends Error {
    status;
    code;
    details;
    constructor(message, status, code, details) {
        super(message);
        this.status = status;
        this.code = code;
        this.details = details;
        this.name = 'CutlineError';
    }
}
export class Cutline {
    apiKey;
    baseUrl;
    customFetch;
    renders;
    batches;
    brandKits;
    templates;
    usage;
    constructor(options = {}) {
        const envKey = globalThis.process?.env?.CUTLINE_API_KEY;
        this.apiKey = options.apiKey || (typeof envKey === 'string' ? envKey : '');
        this.baseUrl = (options.baseUrl || 'https://api.cutline.dev').replace(/\/+$/, '');
        this.customFetch = options.fetch || globalThis.fetch;
        this.renders = new RendersResource(this);
        this.batches = new BatchesResource(this);
        this.brandKits = new BrandKitsResource(this);
        this.templates = new TemplatesResource(this);
        this.usage = new UsageResource(this);
    }
    async request(path, options = {}) {
        const url = `${this.baseUrl}${path}`;
        const headers = {
            'Content-Type': 'application/json',
            Accept: 'application/json, application/problem+json',
            ...options.headers,
        };
        if (this.apiKey) {
            headers['Authorization'] = `Bearer ${this.apiKey}`;
        }
        const res = await this.customFetch(url, { ...options, headers });
        if (!res.ok) {
            let errorBody = null;
            try {
                errorBody = await res.json();
            }
            catch {
                errorBody = { message: await res.text() };
            }
            throw new CutlineError(errorBody?.detail || errorBody?.title || errorBody?.message || `Request failed with status ${res.status}`, res.status, errorBody?.code, errorBody);
        }
        return res.json();
    }
}
export class RendersResource {
    client;
    constructor(client) {
        this.client = client;
    }
    async create(timeline, options = {}) {
        const headers = {};
        if (options.idempotencyKey) {
            headers['Idempotency-Key'] = options.idempotencyKey;
        }
        return this.client.request('/v1/renders', {
            method: 'POST',
            headers,
            body: JSON.stringify(timeline),
        });
    }
    async get(id) {
        return this.client.request(`/v1/renders/${id}`, {
            method: 'GET',
        });
    }
    async cancel(id) {
        return this.client.request(`/v1/renders/${id}`, {
            method: 'DELETE',
        });
    }
    async validate(timeline) {
        return this.client.request('/v1/renders:validate', {
            method: 'POST',
            body: JSON.stringify(timeline),
        });
    }
}
export class BatchesResource {
    client;
    constructor(client) {
        this.client = client;
    }
    async create(template, items, options = {}) {
        return this.client.request('/v1/batches', {
            method: 'POST',
            body: JSON.stringify({
                template,
                items,
                lane: options.lane || 'standard',
                priority: options.priority || 0,
            }),
        });
    }
    async get(id) {
        return this.client.request(`/v1/batches/${id}`, {
            method: 'GET',
        });
    }
}
export class BrandKitsResource {
    client;
    constructor(client) {
        this.client = client;
    }
    async create(kit) {
        return this.client.request('/v1/brand-kits', {
            method: 'POST',
            body: JSON.stringify(kit),
        });
    }
    async get(id) {
        return this.client.request(`/v1/brand-kits/${id}`, {
            method: 'GET',
        });
    }
}
export class TemplatesResource {
    client;
    constructor(client) {
        this.client = client;
    }
    async create(template) {
        return this.client.request('/v1/templates', {
            method: 'POST',
            body: JSON.stringify(template),
        });
    }
    async get(id) {
        return this.client.request(`/v1/templates/${id}`, {
            method: 'GET',
        });
    }
    async pinGolden(id, timestamps) {
        return this.client.request(`/v1/templates/${id}/golden`, {
            method: 'POST',
            body: JSON.stringify({ timestamps }),
        });
    }
    async runCi(id) {
        return this.client.request(`/v1/templates/${id}/ci-run`, {
            method: 'POST',
        });
    }
}
export class UsageResource {
    client;
    constructor(client) {
        this.client = client;
    }
    async get() {
        return this.client.request('/v1/usage', {
            method: 'GET',
        });
    }
}
//# sourceMappingURL=client.js.map