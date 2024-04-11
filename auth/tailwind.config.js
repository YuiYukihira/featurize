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
        purple: {
          "50": "#fcf6fd",
          "100": "#f8edfa",
          "200": "#f1d9f5",
          "300": "#e8bbec",
          "400": "#d992e0",
          "500": "#c667ce",
          "600": "#ab48b1",
          "700": "#8f3992",
          "800": "#763078",
          "900": "#632c63",
          "950": "#4b164b",
        },
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
