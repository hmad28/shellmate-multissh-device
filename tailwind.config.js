/** @type {import('tailwindcss').Config} */
export default {
  darkMode: 'class',
  content: ['./index.html', './src/**/*.{js,ts,jsx,tsx}'],
  theme: {
    extend: {
      colors: {
        // Read from CSS variables — set by ThemeStore on theme apply.
        bg: {
          DEFAULT: 'var(--color-bg)',
          sidebar: 'var(--color-bg-sidebar)',
          panel: 'var(--color-bg-panel)',
          elevated: 'var(--color-bg-elevated)',
        },
        border: {
          DEFAULT: 'var(--color-border)',
          subtle: 'var(--color-border-subtle)',
          strong: 'var(--color-border-strong)',
        },
        fg: {
          DEFAULT: 'var(--color-fg)',
          muted: 'var(--color-fg-muted)',
          subtle: 'var(--color-fg-subtle)',
        },
        accent: {
          DEFAULT: 'var(--color-accent)',
          hover: 'var(--color-accent-hover)',
          subtle: 'var(--color-accent-subtle)',
        },
        status: {
          connected: 'var(--color-status-connected)',
          connecting: 'var(--color-status-connecting)',
          disconnected: 'var(--color-status-disconnected)',
        },
      },
      fontFamily: {
        mono: ['var(--font-mono)'],
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
