const CACHE_NAME = 'spis-v1';

self.addEventListener('install', (event) => {
  self.skipWaiting();
});

self.addEventListener('activate', (event) => {
  event.waitUntil(clients.claim());
});

self.addEventListener('fetch', () => {
  // Present to satisfy PWA install requirements, but deliberately does not
  // call respondWith: proxying through the worker breaks streaming responses
  // like /hx/sse. Requests fall through to the network.
});
