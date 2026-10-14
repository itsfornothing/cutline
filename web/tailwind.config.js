/** @type {import('tailwindcss').Config} */
export default {
  content: ['./index.html', './src/**/*.{js,ts,jsx,tsx}'],
  darkMode: 'class',
  theme: {
    extend: {
      colors: {
        canvas: {
          dark: 'oklch(0.16 0.008 70)',
          light: 'oklch(0.97 0.005 70)',
        },
        panel: {
          dark: 'oklch(0.20 0.008 70)',
          light: 'oklch(0.94 0.005 70)',
        },
        surface: {
          dark: 'oklch(0.24 0.008 70)',
          light: 'oklch(0.90 0.005 70)',
        },
        amber: {
          safelight: 'oklch(0.82 0.16 80)',
          dim: 'oklch(0.65 0.14 80)',
          highlight: 'oklch(0.88 0.18 80)',
        },
        border: {
          subtle: 'rgba(255, 255, 255, 0.08)',
          strong: 'rgba(255, 255, 255, 0.15)',
          lightSubtle: 'rgba(0, 0, 0, 0.08)',
        },
      },
      fontFamily: {
        sans: [
          'Geist',
          'Inter Tight',
          '-apple-system',
          'BlinkMacSystemFont',
          '"Segoe UI"',
          'Roboto',
          'sans-serif',
        ],
        mono: [
          '"JetBrains Mono"',
          '"Berkeley Mono"',
          'ui-monospace',
          'SFMono-Regular',
          'Menlo',
          'Monaco',
          'Consolas',
          'monospace',
        ],
      },
      transitionTimingFunction: {
        cutline: 'cubic-bezier(0.2, 0.8, 0.2, 1)',
      },
      transitionDuration: {
        fast: '120ms',
        panel: '200ms',
      },
      boxShadow: {
        focus: '0 0 0 2px oklch(0.82 0.16 80)',
      },
    },
  },
  plugins: [],
};
