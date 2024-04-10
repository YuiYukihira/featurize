/** @type {import('tailwindcss').Config} */
const colors = require("tailwindcss/colors");

module.exports = {
  content: ["templates/**/*.{html,js}", "public/**/*.{html,js}"],
  theme: {
    extend: {
      colors: {
        transparent: "transparent",
        current: "currentColor",
        black: colors.black,
        white: colors.white,
        gray: colors.slate,
        red: colors.red,
        orange: colors.orange,
        yellow: colors.yellow,
        green: colors.green,
        blue: colors.sky,
        purple: colors.violet,
        pink: {
          "50": "#fcf3f9",
          "100": "#fae9f4",
          "200": "#f8d2ea",
          "300": "#f3afd8",
          "400": "#ea7cbd",
          "500": "#e054a1",
          "600": "#cd3582",
          "700": "#b12567",
          "800": "#932156",
          "900": "#7b204a",
          "950": "#4a0d29",
        },
      },
    },
  },
  plugins: [],
};
