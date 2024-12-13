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
}

