import { main, initSync } from "./wasm/game.js"
import * as wbg from "./wasm/game_bg.js"

await WebAssembly
    .instantiateStreaming(fetch("./wasm/game_bg.wasm"), { wbg: wbg })
    .then((res_obj) => {
      initSync({ module: res_obj.module, memory: res_obj.instance.exports.memory })
    })
    .catch(console.error);
