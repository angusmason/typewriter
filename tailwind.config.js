/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    "index.html",
    "src/**/*.rs",
  ],
  theme: {
    extend: {
      fontFamily: {
        'sans': ["Mono"],
      },
      colors: {
        'brown': '#29272B',
        'darkbrown': '#080809'
      },
    },
  },
  plugins: [],
}
