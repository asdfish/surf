import { main, initSync } from "./game/game.js"
import * as game from "./game/game_bg.js"

await WebAssembly
    .instantiateStreaming(fetch("./game/game_bg.wasm"), { wbg: game })
    .then((res_obj) => initSync({ module: res_obj.module, memory: res_obj.instance.exports.memory }))
    .then(() => main())
    .catch(console.error);

