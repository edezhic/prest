// TypeScript shenanigans 
/// <reference lib="WebWorker" />
const { addEventListener, clients, location } = self as unknown as ServiceWorkerGlobalScope;

// import WASM module initializer and generated bindings 
// lib.js and lib.d.ts will be generated and updated in this folder right before bundling
import init, { serve } from "./lib.js";
// holding SW installation completion until wasm module is initialized
addEventListener('install', event => event.waitUntil(init('/sw.wasm')));
// force all tabs/windows to use SW for requests
addEventListener("activate", event => event.waitUntil(clients.claim()));
// using rust service to handle fetch event
addEventListener('fetch', event => serve(location.host, event));