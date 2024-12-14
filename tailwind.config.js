/** @type {import('tailwindcss').Config} */
export default {
  content: [
    './src/**/*.{html,js,svelte,ts}', // Include all SvelteKit source files
    './src/routes/**/*.{svelte,ts}', // Include SvelteKit route files
  ],
  theme: {
    extend: {},
  },
  plugins: [require('@tailwindcss/typography')],
  darkMode: 'class', // or 'media', depending on your setup
  theme: {
    extend: {
      colors: {
        indigo: {
          300: '#a5b4fc', // Ensure this matches Tailwind's default colors
          600: '#4f46e5', // Ensure this matches Tailwind's default colors
        },
      },
    },
  },
}

