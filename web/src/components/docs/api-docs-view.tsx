import React, { useState } from 'react';
import {
  BookOpen,
  Terminal,
  Copy,
  Check,
  ExternalLink,
  ChevronRight,
  ShieldCheck,
  Play,
  Layers,
  Code2,
} from 'lucide-react';

interface EndpointDoc {
  method: 'POST' | 'GET' | 'DELETE';
  path: string;
  title: string;
  description: string;
  curlExample: string;
  tsExample: string;
  pythonExample: string;
  responseExample: string;
}

export const ApiDocsView: React.FC = () => {
  const [activeLang, setActiveLang] = useState<'curl' | 'ts' | 'python'>('curl');
  const [selectedEndpointIdx, setSelectedEndpointIdx] = useState<number>(0);
  const [copied, setCopied] = useState<boolean>(false);

  const endpoints: EndpointDoc[] = [
    {
      method: 'POST',
      path: '/v1/renders',
      title: 'Create Render Job',
      description: 'Submits a JSON timeline for asynchronous GPU rendering. Supports Idempotency-Key header to prevent duplicate execution.',
      curlExample: `curl -X POST https://api.cutline.dev/v1/renders \\
  -H "Authorization: Bearer cl_live_sample" \\
  -H "Content-Type: application/json" \\
  -H "Idempotency-Key: job-2026-10-14-001" \\
  -d '{
    "schema_version": "1.0",
    "outputs": [{ "id": "out", "aspect": "9:16", "resolution": "1080x1920", "fps": 30, "format": "mp4" }],
    "assets": { "bg": { "type": "video", "src": "s3://bucket/vid.mp4", "duration": 10.0 } },
    "tracks": [{ "id": "v1", "kind": "video", "clips": [{ "asset": "bg", "start": 0, "duration": 10.0 }] }]
  }'`,
      tsExample: `import { Cutline } from '@cutline/sdk';

const cutline = new Cutline({ apiKey: process.env.CUTLINE_API_KEY });

const render = await cutline.renders.create({
  timeline: {
    schema_version: '1.0',
    outputs: [{ id: 'out', aspect: '9:16', resolution: { width: 1080, height: 1920 }, fps: 30, format: 'mp4' }],
    assets: { bg: { type: 'video', src: 's3://bucket/vid.mp4', duration: 10.0 } },
    tracks: [{ id: 'v1', kind: 'video', clips: [{ asset: 'bg', start: 0, duration: 10.0 }] }],
  },
  idempotencyKey: 'job-2026-10-14-001',
});

console.log('Render enqueued:', render.id);`,
      pythonExample: `from cutline import Cutline

client = Cutline(api_key="cl_live_sample")

job = client.renders.create(
    timeline={
        "schema_version": "1.0",
        "outputs": [{"id": "out", "aspect": "9:16", "resolution": "1080x1920", "fps": 30, "format": "mp4"}],
        "assets": {"bg": {"type": "video", "src": "s3://bucket/vid.mp4", "duration": 10.0}},
        "tracks": [{"id": "v1", "kind": "video", "clips": [{"asset": "bg", "start": 0, "duration": 10.0}]}],
    },
    idempotency_key="job-2026-10-14-001",
)

print(f"Render enqueued: {job.id}")`,
      responseExample: `{
  "id": "01927c34-8c88-7e23-81a1-8d0739c9f28c",
  "status": "queued",
  "lane": "standard",
  "priority": 0,
  "created_at": "2026-10-14T15:02:12Z"
}`,
    },
    {
      method: 'POST',
      path: '/v1/renders:validate',
      title: 'Dry-Run Quote & Pre-flight Validation',
      description: 'Free verification endpoint. Returns JSON-pointer schema errors, asset reachability checks, duration calculation, and itemized cost quote without queuing a job.',
      curlExample: `curl -X POST https://api.cutline.dev/v1/renders:validate \\
  -H "Authorization: Bearer cl_live_sample" \\
  -H "Content-Type: application/json" \\
  -d @timeline.json`,
      tsExample: `const quote = await cutline.renders.validate(timeline);
console.log('Estimated USD:', quote.estimated_total_usd);`,
      pythonExample: `quote = client.renders.validate(timeline)
print(f"Estimated USD: {quote.estimated_total_usd}")`,
      responseExample: `{
  "valid": true,
  "billed_seconds": 12.0,
  "estimated_total_usd": 0.0180,
  "quote": {
    "base_rate_per_min": 0.08,
    "fanout_cost": 0.0040,
    "captions_cost": 0.0020,
    "provenance_cost": 0.0000
  },
  "warnings": []
}`,
    },
    {
      method: 'POST',
      path: '/v1/batches',
      title: 'Submit Batch Campaign',
      description: 'Upload a template and an array of variable overrides to generate N videos with a single API call, aggregate progress board, and consolidated invoice line.',
      curlExample: `curl -X POST https://api.cutline.dev/v1/batches \\
  -H "Authorization: Bearer cl_live_sample" \\
  -H "Content-Type: application/json" \\
  -d '{
    "template": { ... },
    "items": [{ "name": "Alice" }, { "name": "Bob" }]
  }'`,
      tsExample: `const batch = await cutline.batches.create({
  template,
  items: [{ name: 'Alice' }, { name: 'Bob' }],
});`,
      pythonExample: `batch = client.batches.create(
    template=template,
    items=[{"name": "Alice"}, {"name": "Bob"}],
)`,
      responseExample: `{
  "id": "01927c34-8c88-7e23-81a1-8d0739c9f28c",
  "total_jobs": 2,
  "render_ids": ["0192...", "0192..."],
  "created_at": "2026-10-14T15:02:12Z"
}`,
    },
  ];

  const currentEndpoint = endpoints[selectedEndpointIdx];

  const handleCopyCode = (text: string) => {
    navigator.clipboard.writeText(text);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const getActiveCode = () => {
    switch (activeLang) {
      case 'ts':
        return currentEndpoint.tsExample;
      case 'python':
        return currentEndpoint.pythonExample;
      case 'curl':
      default:
        return currentEndpoint.curlExample;
    }
  };

  return (
    <div className="flex-1 flex h-[calc(100vh-3rem)] overflow-hidden select-none bg-canvas-dark text-xs">
      {/* Pane 1: Endpoints Navigation Sidebar */}
      <aside className="w-64 bg-panel-dark border-r border-white/8 p-3 flex flex-col space-y-3">
        <div className="font-mono text-[11px] text-neutral-400 uppercase tracking-wider px-2">
          API ENDPOINTS (v1)
        </div>

        <div className="space-y-1">
          {endpoints.map((ep, idx) => (
            <button
              key={ep.path}
              onClick={() => setSelectedEndpointIdx(idx)}
              className={`w-full p-2.5 rounded text-left font-mono transition-colors flex items-center justify-between ${
                selectedEndpointIdx === idx
                  ? 'bg-amber-safelight/15 text-amber-safelight border border-amber-safelight/30 font-semibold'
                  : 'text-neutral-300 hover:bg-white/4 border border-transparent'
              }`}
            >
              <div>
                <span className="text-[10px] font-bold text-amber-400 mr-2">{ep.method}</span>
                <span className="text-xs">{ep.path}</span>
              </div>
              <ChevronRight className="w-3.5 h-3.5 opacity-40" />
            </button>
          ))}
        </div>
      </aside>

      {/* Pane 2: Documentation Description & OpenAPI Schema */}
      <div className="flex-1 overflow-y-auto p-6 space-y-6 max-w-2xl">
        <div className="space-y-2">
          <div className="flex items-center gap-2">
            <span className="px-2 py-0.5 rounded font-mono font-bold text-xs bg-amber-safelight text-neutral-950">
              {currentEndpoint.method}
            </span>
            <span className="font-mono text-sm font-bold text-neutral-100">
              {currentEndpoint.path}
            </span>
          </div>
          <h2 className="text-base font-bold text-neutral-100">{currentEndpoint.title}</h2>
          <p className="text-neutral-400 leading-relaxed">{currentEndpoint.description}</p>
        </div>

        {/* Security & Headers Card */}
        <div className="p-4 rounded bg-panel-dark border border-white/8 space-y-2 font-mono text-xs">
          <span className="text-neutral-400 font-semibold text-[11px] uppercase">
            HEADERS & AUTHENTICATION
          </span>
          <div className="text-neutral-300 space-y-1">
            <div>
              <span className="text-amber-safelight">Authorization:</span> Bearer &lt;API_KEY&gt;
            </div>
            <div>
              <span className="text-amber-safelight">Idempotency-Key:</span> &lt;UUID / custom key&gt; (Optional)
            </div>
            <div>
              <span className="text-amber-safelight">Content-Type:</span> application/json
            </div>
          </div>
        </div>

        {/* Response Body Spec */}
        <div className="space-y-2 font-mono">
          <span className="text-neutral-400 font-semibold text-[11px] uppercase">
            RESPONSE (application/json)
          </span>
          <pre className="p-4 rounded bg-panel-dark border border-white/8 text-neutral-200 text-[11px] overflow-x-auto">
            {currentEndpoint.responseExample}
          </pre>
        </div>
      </div>

      {/* Pane 3: Code Snippets & Language Selector */}
      <div className="w-96 bg-panel-dark border-l border-white/8 flex flex-col overflow-hidden">
        {/* Language Tabs */}
        <div className="h-10 border-b border-white/8 px-4 flex items-center justify-between bg-panel-dark/50">
          <div className="flex items-center gap-1 font-mono text-xs">
            {(['curl', 'ts', 'python'] as const).map((lang) => (
              <button
                key={lang}
                onClick={() => setActiveLang(lang)}
                className={`px-2.5 py-1 rounded capitalize font-medium transition-colors ${
                  activeLang === lang
                    ? 'bg-white/10 text-neutral-100 font-bold'
                    : 'text-neutral-400 hover:text-neutral-200'
                }`}
              >
                {lang === 'ts' ? 'TypeScript' : lang}
              </button>
            ))}
          </div>

          <button
            onClick={() => handleCopyCode(getActiveCode())}
            className="p-1.5 text-neutral-400 hover:text-neutral-200 transition-colors"
            title="Copy code"
          >
            {copied ? <Check className="w-3.5 h-3.5 text-emerald-400" /> : <Copy className="w-3.5 h-3.5" />}
          </button>
        </div>

        {/* Code Content */}
        <div className="flex-1 p-4 bg-canvas-dark overflow-x-auto font-mono text-[11px] text-neutral-200 leading-relaxed">
          <pre>{getActiveCode()}</pre>
        </div>
      </div>
    </div>
  );
};
