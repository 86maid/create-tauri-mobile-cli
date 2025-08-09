const { invoke } = window.__TAURI__.core;
const { listen, emit } = window.__TAURI__.event;

const term = new Terminal();
const fitAddon = new FitAddon.FitAddon();

term.loadAddon(fitAddon);
term.open(document.getElementById("terminal"));
fitAddon.fit();

window.addEventListener("resize", () => {
  fitAddon.fit();
});

let buffer = "";

term.onData(async (c) => {
  switch (c) {
    case "\r":
    case "\n":
      term.write("\r\n");
      emit("cli-read", buffer);
      buffer = "";
      break;
    case "\u007f":
      if (buffer.length > 0) {
        buffer = buffer.slice(0, -1);
        term.write("\b \b");
      }
      break;
    default:
      buffer += c;
      term.write(c);
  }
});

listen("cli-write", (e) => {
  console.log(JSON.stringify(e.payload));
  term.write(e.payload.replace(/\n/g, "\r\n"));
});


listen("cli-cmd", (e) => {
  switch (e.payload) {
    case "clear":
      term.clear();
      break;
  }
});

await invoke("main");