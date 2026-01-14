const CACHE_NAME = 'spis-v1';

self.addEventListener('install', (event) => {
  self.skipWaiting();
});

self.addEventListener('activate', (event) => {
  event.waitUntil(clients.claim());
});

self.addEventListener('fetch', (event) => {
  // Simple pass-through to satisfy PWA requirements.
  // In the future, we can add caching logic here.
  event.respondWith(fetch(event.request));
});
