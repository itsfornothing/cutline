export interface SampleTemplate {
  id: string;
  name: string;
  description: string;
  aspect: string;
  duration: number;
  timeline: any;
}

export const SAMPLE_TEMPLATES: SampleTemplate[] = [
  {
    id: 'faceless_tiktok',
    name: 'Faceless TikTok / Reel',
    description: '9:16 vertical short with word-level karaoke captions and safe-zone positioning',
    aspect: '9:16',
    duration: 12.0,
    timeline: {
      schema_version: '1.0',
      variables: {
        title: 'Why I Built Cutline',
        speaker: 'Founder',
      },
      outputs: [
        {
          id: 'tt',
          aspect: '9:16',
          resolution: { width: 1080, height: 1920 },
          fps: 30,
          format: 'mp4',
        },
      ],
      assets: {
        bg: {
          type: 'video',
          src: 's3://bucket/minecraft_parkour.mp4',
          duration: 12.0,
          loop: true,
        },
        vo: {
          type: 'audio',
          src: 'https://cdn.cutline.dev/audio/founder_story.mp3',
          duration: 12.0,
        },
      },
      tracks: [
        {
          id: 'v1',
          kind: 'video',
          clips: [
            {
              asset: 'bg',
              start: 0,
              duration: 12.0,
              fit: 'cover',
              keyframes: {
                scale: [
                  [0, 1.0, 'ease_out'],
                  [12, 1.08, 'linear'],
                ],
              },
            },
          ],
        },
        {
          id: 'a1',
          kind: 'audio',
          clips: [{ asset: 'vo', start: 0, duration: 12.0 }],
          mix: { role: 'voice' },
        },
        {
          id: 'c1',
          kind: 'captions',
          source: { align_audio: 'vo', language: 'auto' },
          style: {
            preset: 'karaoke',
            font: 'Inter',
            font_size: 52,
            highlight: '#FFB000',
            safe_zone: 'tiktok',
          },
        },
      ],
      audio: {
        normalize_lufs: -14,
        duck: { under: 'voice', amount_db: -12 },
      },
      provenance: { c2pa: true, ai_generated: true },
    },
  },
  {
    id: 'real_estate_tour',
    name: 'Luxury Real Estate Tour',
    description: '16:9 widescreen tour with pan-and-scan camera moves and brand typography',
    aspect: '16:9',
    duration: 10.0,
    timeline: {
      schema_version: '1.0',
      variables: {
        property_name: 'The Glass Pavilion',
        price: '$4,250,000',
      },
      outputs: [
        {
          id: 'yt',
          aspect: '16:9',
          resolution: { width: 1920, height: 1080 },
          fps: 30,
          format: 'mp4',
        },
      ],
      assets: {
        living_room: {
          type: 'video',
          src: 's3://bucket/living_room_4k.mp4',
          duration: 10.0,
        },
      },
      tracks: [
        {
          id: 'v1',
          kind: 'video',
          clips: [
            {
              asset: 'living_room',
              start: 0,
              duration: 10.0,
              keyframes: {
                scale: [
                  [0, 1.0, 'linear'],
                  [10, 1.06, 'ease_out'],
                ],
                opacity: [
                  [0, 0.0, 'ease_in'],
                  [1, 1.0, 'linear'],
                ],
              },
            },
          ],
        },
        {
          id: 't1',
          kind: 'text',
          clips: [
            {
              start: 1.0,
              duration: 6.0,
              text: {
                text: '{{property_name}} · {{price}}',
                font: 'Geist',
                font_size: 42,
                color: '#FFFFFF',
              },
            },
          ],
        },
      ],
      audio: { normalize_lufs: -14 },
      provenance: { c2pa: false, ai_generated: false },
    },
  },
  {
    id: 'fanout_campaign',
    name: 'Multi-Format Promo (9:16 + 1:1 + 16:9)',
    description: 'Single job multi-format fan-out with smart reframing and C2PA signing',
    aspect: 'Multi',
    duration: 15.0,
    timeline: {
      schema_version: '1.0',
      variables: {
        campaign: 'Summer Launch',
      },
      outputs: [
        {
          id: 'yt',
          aspect: '16:9',
          resolution: { width: 1920, height: 1080 },
          fps: 30,
          format: 'mp4',
        },
        {
          id: 'tt',
          aspect: '9:16',
          resolution: { width: 1080, height: 1920 },
          fps: 30,
          format: 'mp4',
          reframe: 'smart',
        },
        {
          id: 'sq',
          aspect: '1:1',
          resolution: { width: 1080, height: 1080 },
          fps: 30,
          format: 'mp4',
          reframe: 'smart',
        },
      ],
      assets: {
        hero: {
          type: 'video',
          src: 's3://bucket/product_showcase.mp4',
          duration: 15.0,
        },
      },
      tracks: [
        {
          id: 'v1',
          kind: 'video',
          clips: [{ asset: 'hero', start: 0, duration: 15.0 }],
        },
      ],
      audio: { normalize_lufs: -14 },
      provenance: { c2pa: true, ai_generated: true },
    },
  },
];
