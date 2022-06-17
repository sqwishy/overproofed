import { render } from 'solid-js/web';
import { createSignal, createEffect } from 'solid-js';

import { Bread } from './Bread';
import init, * as wasm from '../public/p/overproofed_wasm.js';

const [history, setHistory] = createSignal({ state: {}, url: new URL(window.location.toString()) });
window.addEventListener('popstate', ({ state }) => setHistory({ state, url: new URL(window.location.toString()) }));

const [title, setTitle] = createSignal(document.title);

createEffect(() => (document.title = title()));

(async () => {
    await init();

    wasm.set_console_panic_hook();

    render(() => <Bread history={history} title={setTitle} />, document.body);
})();
