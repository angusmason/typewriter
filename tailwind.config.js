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
        // 'background': '#29272B',
        // 'highlight': '#080809',
        // 'accent': '#CA695C',
        // 'fade': '#655F6A',
        'background': '#1B1A1A',
        'highlight': '#080809',
        'accent': '#CA695C',
        'fade': '#655F6A',
        'text': "#F7F7F7",
      },
    },
  },
  plugins: [],
}
