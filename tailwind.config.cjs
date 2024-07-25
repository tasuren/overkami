const colors = require("tailwindcss/colors");

/**@type {import("tailwindcss").Config} */
module.exports = {
    darkMode: ["class", '[data-kb-theme="dark"]'],
    content: ["./src/**/*.{ts,tsx}"],
    plugins: [],

    darkMode: ["selector", '[data-theme="dark"]'],
    theme: {
        extend: {
            colors: {
                dark: colors.zinc[800],
                light: colors.zinc[200]
            }
        }
    }
};
