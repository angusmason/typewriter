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
        'background': '#222222',
        'highlight': '#080707',
        'accent': '#888888',
        'fade': '#545354',
        'text': "#EEEEEE",
        'caret': "#EEEEEE",
      },
    },
  },
  plugins: [],
}
