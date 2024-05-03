/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html", "./src/**/*.{svelte,js,ts,jsx,tsx}", 
    "./node_modules/flowbite-svelte/**/*.{html,js,svelte,ts}"
  ],
  theme: {
		colors: {
			"primary-dark": "#27272a",
      "secondary-dark": "#3f3f46",
      "black": "#09090b",
			"primary-light": "#d4d4d8",
      "secondary-light": "#e4e4e7",
      "white": "white",
			"link": "#539af8",
			"link-visited": "#9268de"
		},
  },
	darkMode: ["selector", '[data-theme="dark"]'],
  corePlugins: {
    preflight: false,
  }
}
