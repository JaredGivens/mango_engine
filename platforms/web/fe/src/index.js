import { WASI, File, OpenFile, PreopenDirectory } from "@bjorn3/browser_wasi_shim";

async function main() {
  let args = [];
  let env = ["FOO=bar"];
  let fds = [
    new OpenFile(new File([])), // stdin
    new OpenFile(new File([])), // stdout
    new OpenFile(new File([])), // stderr
    new PreopenDirectory(".", {
      "example.c": new File(new TextEncoder("utf-8").encode(`#include "a"`)),
      "hello.rs": new File(new TextEncoder("utf-8").encode(`fn main() { println!("Hello World!"); }`)),
    }),
  ];
  let wasi = new WASI(args, env, fds);

  let wasm = await WebAssembly.compileStreaming(fetch("build/iced_mg.wasm"));
  let inst = await WebAssembly.instantiate(wasm, {
    "wasi_snapshot_preview1": wasi.wasiImport,
  });
  wasi.start(inst);
}
main()
