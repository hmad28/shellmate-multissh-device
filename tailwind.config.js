/** @type {import('tailwindcss').Config} */
export default {
  darkMode: 'class',
  content: ['./index.html', './src/**/*.{js,ts,jsx,tsx}'],
  theme: {
    extend: {
      colors: {
        // Background hierarchy
        bg: {
          DEFAULT: '#0a0a0f', // app background
          sidebar: '#111118',
          panel: '#15151d',
          elevated: '#1c1c26',
        },
        // Borders
        border: {
          DEFAULT: '#26262f',
          subtle: '#1e1e26',
          strong: '#33333f',
        },
        // Text
        fg: {
          DEFAULT: '#e8e8ea',
          muted: '#9a9aa3',
          subtle: '#6b6b75',
        },
        // Accent
        accent: {
          DEFAULT: '#3b82f6',
          hover: '#60a5fa',
          subtle: '#1e3a8a',
        },
        // Status
        status: {
          connected: '#22c55e',
          connecting: '#f59e0b',
          disconnected: '#ef4444',
        },
      },
      fontFamily: {
        mono: [
          'JetBrains Mono',
          'Fira Code',
          'Consolas',
          'Monaco',
          'monospace',
        ],
        sans: [
          'Inter',
          '-apple-system',
          'BlinkMacSystemFont',
          'Segoe UI',
          'system-ui',
          'sans-serif',
        ],
      },
      fontSize: {
        terminal: ['14px', { lineHeight: '1.4' }],
      },
      transitionDuration: {
        DEFAULT: '150ms',
      },
    },
  },
  plugins: [],
};
