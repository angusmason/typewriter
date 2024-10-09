/** @type {import('tailwindcss').Config} */
module.exports = {
    content: [
      "index.html",
      "src/**/*.rs",
    ],
    theme: {
      extend: {
        fontFamily: {
          'mono': ["Mono"],
        },
        colors: {
            'brown': '#29272B',
          },
      },
    },
    plugins: [],
  }
  