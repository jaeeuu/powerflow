import tailwindcssForm from '@tailwindcss/forms'
import tailwindcssAnimate from 'tailwindcss-animate'

const withOpacity = (variable: `--${string}`) => ({ opacityValue }: { opacityValue?: string }) => {
  if (opacityValue === undefined) {
    return `hsl(var(${variable}))`
  }

  return `hsl(var(${variable}) / ${opacityValue})`
}

const semanticColors = {
  background: withOpacity('--background'),
  foreground: withOpacity('--foreground'),
  primary: {
    DEFAULT: withOpacity('--primary'),
    foreground: withOpacity('--primary-foreground'),
  },
  secondary: {
    DEFAULT: withOpacity('--secondary'),
    foreground: withOpacity('--secondary-foreground'),
  },
  destructive: {
    DEFAULT: withOpacity('--destructive'),
    foreground: withOpacity('--destructive-foreground'),
  },
  muted: {
    DEFAULT: withOpacity('--muted'),
    foreground: withOpacity('--muted-foreground'),
  },
  accent: {
    DEFAULT: withOpacity('--accent'),
    foreground: withOpacity('--accent-foreground'),
  },
  popover: {
    DEFAULT: withOpacity('--popover'),
    foreground: withOpacity('--popover-foreground'),
  },
  card: {
    DEFAULT: withOpacity('--card'),
    foreground: withOpacity('--card-foreground'),
  },
} as const

const colorPalette = {
  border: {
    DEFAULT: withOpacity('--border'),
  },
  input: {
    DEFAULT: withOpacity('--input'),
  },
  ring: {
    DEFAULT: withOpacity('--ring'),
  },
  ...semanticColors,
} as const

/** @type {import('tailwindcss').Config} */
export default {
  darkMode: 'class',
  content: [
    './pages/**/*.{ts,tsx,vue}',
    './components/**/*.{ts,tsx,vue}',
    './app/**/*.{ts,tsx,vue}',
    './src/**/*.{ts,tsx,vue}',
  ],
  theme: {
    container: {
      center: true,
      padding: '2rem',
      screens: {
        '2xl': '1400px',
      },
    },
    extend: {
      colors: colorPalette,
      backgroundColor: semanticColors,
      textColor: semanticColors,
      borderColor: {
        DEFAULT: withOpacity('--border'),
        border: withOpacity('--border'),
        input: withOpacity('--input'),
        primary: withOpacity('--primary'),
        secondary: withOpacity('--secondary'),
        destructive: withOpacity('--destructive'),
        accent: withOpacity('--accent'),
        muted: withOpacity('--muted'),
      },
      ringColor: {
        DEFAULT: withOpacity('--ring'),
        ring: withOpacity('--ring'),
      },
      ringOffsetColor: {
        background: withOpacity('--background'),
      },
      borderRadius: {
        xl: 'calc(var(--radius) + 4px)',
        lg: 'var(--radius)',
        md: 'calc(var(--radius) - 2px)',
        sm: 'calc(var(--radius) - 4px)',
      },
      keyframes: {
        'accordion-down': {
          from: { height: 0 },
          to: { height: 'var(--radix-accordion-content-height)' },
        },
        'accordion-up': {
          from: { height: 'var(--radix-accordion-content-height)' },
          to: { height: 0 },
        },
        'collapsible-down': {
          from: { height: 0 },
          to: { height: 'var(--radix-collapsible-content-height)' },
        },
        'collapsible-up': {
          from: { height: 'var(--radix-collapsible-content-height)' },
          to: { height: 0 },
        },
      },
      animation: {
        'accordion-down': 'accordion-down 0.2s ease-out',
        'accordion-up': 'accordion-up 0.2s ease-out',
        'collapsible-down': 'collapsible-down 0.2s ease-in-out',
        'collapsible-up': 'collapsible-up 0.2s ease-in-out',
      },
    },
  },
  plugins: [tailwindcssForm, tailwindcssAnimate],
}
