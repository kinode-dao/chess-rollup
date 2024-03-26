# sequencer_ui

The UI for the sequencer package. This UI is available at /sequencer:rollup:goldfinger.os

# Developing

To get started, install the dependencies with `npm install`.

Next, start your local kinode in the manner you are accustomed, and install the other processes in this repo.

Then, run the development server with `npm run dev`.

Our UI uses [Tailwind](https://tailwindcss.com/) for styling. 
If you're changing CSS, kick off the Tailwind compiler with `npx tailwindcss -i ./src/input.css -o ./src/index.css --watch`.

Finally, open the UI in your browser at http://localhost:5173. 

