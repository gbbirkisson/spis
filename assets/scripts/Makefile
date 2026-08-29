HTMX_VERSION:=4.0.0
HYPERSCRIPT_VERSION:=0.9.93

all: htmx-${HTMX_VERSION}.js htmx-sse-${HTMX_VERSION}.js hyperscript-${HYPERSCRIPT_VERSION}.js

htmx-${HTMX_VERSION}.js:
	wget https://cdn.jsdelivr.net/npm/htmx.org@${HTMX_VERSION}/dist/htmx.min.js -O htmx-${HTMX_VERSION}.js

htmx-sse-${HTMX_VERSION}.js:
	wget https://cdn.jsdelivr.net/npm/htmx.org@${HTMX_VERSION}/dist/ext/hx-sse.min.js -O htmx-sse-${HTMX_VERSION}.js

hyperscript-${HYPERSCRIPT_VERSION}.js:
	wget https://cdn.jsdelivr.net/npm/hyperscript.org@${HYPERSCRIPT_VERSION}/dist/_hyperscript.min.js -O hyperscript-${HYPERSCRIPT_VERSION}.js
